//! Relevant structure of an mp4 file
//!
//! ```md
//! ftyp
//! mdat
//! moov
//! ├─ mvhd
//! ├─ trak
//! │  ├─ tkhd
//! │  ├─ tref
//! │  │  └─ chap
//! │  └─ mdia
//! │     ├─ mdhd
//! │     ├─ hdlr
//! │     └─ minf
//! │        ├─ dinf
//! │        │  └─ dref
//! │        │     └─ url
//! │        ├─ gmhd
//! │        │  ├─ gmin
//! │        │  └─ text
//! │        └─ stbl
//! │           ├─ stsd
//! │           │  ├─ mp4a
//! │           │  │  └─ esds
//! │           │  └─ text
//! │           ├─ stts
//! │           ├─ stsc
//! │           ├─ stsz
//! │           ├─ stco
//! │           └─ co64
//! └─ udta
//!    ├─ chpl
//!    └─ meta
//!       ├─ hdlr
//!       └─ ilst
//!          ├─ **** (any fourcc)
//!          │  └─ data
//!          └─ ---- (freeform fourcc)
//!             ├─ mean
//!             ├─ name
//!             └─ data
//! ```

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::{AudioInfo, Chapter, ErrorKind, Tag, Userdata};

use head::*;
use ident::*;
use util::*;

use change::*;
use chap::*;
use chpl::*;
use co64::*;
use dinf::*;
use dref::*;
use ftyp::*;
use gmhd::*;
use gmin::*;
use hdlr::*;
use ilst::*;
use mdat::*;
use mdhd::*;
use mdia::*;
use meta::*;
use minf::*;
use moov::*;
use mp4a::*;
use mvhd::*;
use state::*;
use stbl::*;
use stco::*;
use stsc::*;
use stsd::*;
use stsz::*;
use stts::*;
use text::*;
use tkhd::*;
use trak::*;
use tref::*;
use udta::*;
use url::*;

pub use data::Data;
pub use metaitem::MetaItem;

/// A module for working with identifiers.
pub mod ident;

#[macro_use]
mod util;
mod head;

mod change;
mod chap;
mod chpl;
mod co64;
mod data;
mod dinf;
mod dref;
mod ftyp;
mod gmhd;
mod gmin;
mod hdlr;
mod ilst;
mod mdat;
mod mdhd;
mod mdia;
mod meta;
mod metaitem;
mod minf;
mod moov;
mod mp4a;
mod mvhd;
mod state;
mod stbl;
mod stco;
mod stsc;
mod stsd;
mod stsz;
mod stts;
mod text;
mod tkhd;
mod trak;
mod tref;
mod udta;
mod url;

trait Atom: Sized {
    const FOURCC: Fourcc;
}

trait ParseAtom: Atom {
    fn parse(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        match Self::parse_atom(reader, cfg, size) {
            Err(mut e) => {
                let mut d = e.description.into_owned();
                insert_str(&mut d, "Error parsing ", Self::FOURCC);
                e.description = d.into();
                Err(e)
            }
            a => a,
        }
    }

    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self>;
}

trait WriteAtom: Atom {
    fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self.write_atom(writer) {
            Err(mut e) => {
                let mut d = e.description.into_owned();
                insert_str(&mut d, "Error writing ", Self::FOURCC);
                e.description = d.into();
                Err(e)
            }
            a => a,
        }
    }

    fn write_head(&self, writer: &mut impl Write) -> crate::Result<()> {
        let head = Head::from(self.size(), Self::FOURCC);
        write_head(writer, head)
    }

    fn len(&self) -> u64 {
        self.size().len()
    }

    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()>;

    fn size(&self) -> Size;
}

trait CollectChanges {
    /// Recursively collect changes and return the length difference when applied.
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64;
}

impl<T: CollectChanges> CollectChanges for Option<T> {
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.as_ref().map_or(0, |a| a.collect_changes(insert_pos, level, changes))
    }
}

trait SimpleCollectChanges: WriteAtom {
    fn state(&self) -> &State;

    /// Add changes, if any, and return the length difference when applied.
    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64;

    fn atom_ref(&self) -> AtomRef<'_>;
}

impl<T: SimpleCollectChanges> CollectChanges for T {
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        match &self.state() {
            State::Existing(b) => {
                let len_diff = self.existing(level + 1, b, changes);
                if len_diff != 0 {
                    changes.push(Change::UpdateLen(UpdateAtomLen {
                        bounds: b,
                        fourcc: Self::FOURCC,
                        len_diff,
                    }));
                }
                len_diff
            }
            State::Remove(b) => {
                changes.push(Change::Remove(RemoveAtom { bounds: b, level: level + 1 }));
                -(b.len() as i64)
            }
            State::Replace(b) => {
                let r = ReplaceAtom { bounds: b, atom: self.atom_ref(), level: level + 1 };
                let len_diff = r.len_diff();
                changes.push(Change::Replace(r));
                len_diff
            }
            State::Insert => {
                changes.push(Change::Insert(InsertAtom {
                    pos: insert_pos,
                    atom: self.atom_ref(),
                    level: level + 1,
                }));
                self.len() as i64
            }
        }
    }
}

fn insert_str(description: &mut String, msg: &str, fourcc: Fourcc) {
    description.reserve(msg.len() + 6);
    description.insert_str(0, ": ");
    fourcc.iter().rev().for_each(|c| {
        description.insert(0, char::from(*c));
    });
    description.insert_str(0, msg);
}

trait LenOrZero {
    fn len_or_zero(&self) -> u64;
}

impl<T: WriteAtom> LenOrZero for Option<T> {
    fn len_or_zero(&self) -> u64 {
        self.as_ref().map_or(0, |a| a.len())
    }
}

trait PushAndGet<T> {
    fn push_and_get(&mut self, item: T) -> &mut T;
}
impl<T> PushAndGet<T> for Vec<T> {
    fn push_and_get(&mut self, item: T) -> &mut T {
        self.push(item);
        self.last_mut().unwrap()
    }
}

/// A struct representing a timescale (the number of units that pass per second) that is used to
/// scale time for chapter list (`chpl`) atoms.
///
/// | library          | timescale  |
/// |------------------|------------|
/// | FFMpeg (default) | 10,000,000 |
/// | mp4v2            |      1,000 |
/// | mutagen          |       mvhd |
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChplTimescale {
    /// Use a fixed timescale.
    Fixed(NonZeroU32),
    /// Use the timescale defined in the movie header (mvhd) atom.
    Mvhd,
}

impl ChplTimescale {
    fn fixed_or_mvhd(self, mvhd_timescale: u32) -> u32 {
        match self {
            Self::Fixed(v) => v.get(),
            Self::Mvhd => mvhd_timescale,
        }
    }
}

/// A struct that configures parsing behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadConfig {
    /// Wheter item list metadata will be read.
    pub read_item_list: bool,
    /// Wheter image data will be read.
    pub read_image_data: bool,
    /// Wheter chapter list information will be read.
    pub read_chapter_list: bool,
    /// Wheter chapter track information will be read.
    pub read_chapter_track: bool,
    /// Wheter audio information will be read.
    /// The [`AudioInfo::duration`] will always be read.
    pub read_audio_info: bool,
    /// The timescale that is used to scale time for chapter list (chpl) atoms.
    pub chpl_timescale: ChplTimescale,
}

impl ReadConfig {
    /// The default configuration for reading tags.
    pub const DEFAULT: ReadConfig = ReadConfig {
        read_item_list: true,
        read_image_data: true,
        read_chapter_list: true,
        read_chapter_track: true,
        read_audio_info: true,
        chpl_timescale: ChplTimescale::Fixed(DEFAULT_CHPL_TIMESCALE),
    };
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

pub(crate) struct ParseConfig<'a> {
    cfg: &'a ReadConfig,
    write: bool,
}

pub(crate) fn read_tag(reader: &mut (impl Read + Seek), cfg: &ReadConfig) -> crate::Result<Tag> {
    let parse_cfg = ParseConfig { cfg, write: false };

    let Ftyp(ftyp) = Ftyp::parse(reader)?;

    let len = reader.remaining_stream_len()?;
    let mut parsed_bytes = 0;
    let mut moov = loop {
        if parsed_bytes >= len {
            return Err(crate::Error::new(
                ErrorKind::AtomNotFound(MOVIE),
                "Missing necessary data, no movie (moov) atom found",
            ));
        }

        let head = parse_head(reader)?;
        if head.fourcc() == MOVIE {
            break Moov::parse(reader, &parse_cfg, head.size())?;
        }

        reader.skip(head.content_len() as i64)?;
        parsed_bytes += head.len();
    };

    let mvhd = moov.mvhd;
    let duration = scale_duration(mvhd.timescale, mvhd.duration);

    let metaitems = moov
        .udta
        .as_mut()
        .and_then(|a| a.meta.take())
        .and_then(|a| a.ilst)
        .and_then(|a| a.owned())
        .unwrap_or_default();

    // chapter list atom
    let mut chapter_list = Vec::new();
    if cfg.read_chapter_list {
        if let Some(mut chpl) = moov.udta.and_then(|a| a.chpl).and_then(|a| a.into_owned()) {
            let chpl_timescale = cfg.chpl_timescale.fixed_or_mvhd(mvhd.timescale);

            chpl.sort_by_key(|c| c.start);
            chapter_list.reserve(chpl.len());

            for c in chpl {
                chapter_list.push(Chapter {
                    start: scale_duration(chpl_timescale, c.start),
                    title: c.title,
                });
            }
        }
    }

    // chapter tracks
    let mut chapter_track = Vec::new();
    if cfg.read_chapter_track {
        for chap in moov.trak.iter().filter_map(|a| a.tref.as_ref().and_then(|a| a.chap.as_ref())) {
            for c_id in chap.chapter_ids.iter() {
                let trak = moov.trak.iter().find(|a| a.tkhd.id == *c_id);

                let Some(trak) = trak else {
                    continue; // TODO maybe log warning: referenced chapter track not found
                };

                let Some(mdia) = &trak.mdia else {
                    return Err(crate::Error::new(
                        ErrorKind::AtomNotFound(MEDIA),
                        "Media (mdia) atom of chapter track not found",
                    ));
                };
                let Some(stbl) = mdia.minf.as_ref().and_then(|a| a.stbl.as_ref()) else {
                    return Err(crate::Error::new(
                        ErrorKind::AtomNotFound(SAMPLE_TABLE),
                        "Sample table (stbl) of chapter track not found",
                    ));
                };
                let Some(stsc) = &stbl.stsc else {
                    return Err(crate::Error::new(
                        ErrorKind::AtomNotFound(SAMPLE_TABLE_SAMPLE_TO_CHUNK),
                        "Sample table sample to chunk (stsc) atom of chapter track not found",
                    ));
                };
                let Some(stsz) = &stbl.stsz else {
                    return Err(crate::Error::new(
                        ErrorKind::AtomNotFound(SAMPLE_TABLE_SAMPLE_SIZE),
                        "Sample table sample size (stsz) atom of chapter track not found",
                    ));
                };
                let Some(stts) = &stbl.stts else {
                    return Err(crate::Error::new(
                        ErrorKind::AtomNotFound(SAMPLE_TABLE_TIME_TO_SAMPLE),
                        "Sample table time to sample (stts) atom of chapter track not found",
                    ));
                };
                let timescale = mdia.mdhd.timescale;

                if let Some(co64) = &stbl.co64 {
                    chapter_track.reserve(co64.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapter_track,
                        timescale,
                        &co64.offsets,
                        &stsc.items,
                        stsz.uniform_sample_size,
                        &stsz.sizes,
                        &stts.items,
                    )?;
                } else if let Some(stco) = &stbl.stco {
                    chapter_track.reserve(stco.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapter_track,
                        timescale,
                        &stco.offsets,
                        &stsc.items,
                        stsz.uniform_sample_size,
                        &stsz.sizes,
                        &stts.items,
                    )?;
                } else {
                    todo!("error missing chunk offsets")
                }
            }
        }
    }

    let mut info = AudioInfo { duration, ..Default::default() };
    if cfg.read_audio_info {
        let mp4a = moov.trak.into_iter().find_map(|trak| {
            trak.mdia
                .and_then(|a| a.minf)
                .and_then(|a| a.stbl)
                .and_then(|a| a.stsd)
                .and_then(|a| a.mp4a)
        });
        if let Some(i) = mp4a {
            info.channel_config = i.channel_config;
            info.sample_rate = i.sample_rate;
            info.max_bitrate = i.max_bitrate;
            info.avg_bitrate = i.avg_bitrate;
        }
    }

    let userdata = Userdata { metaitems, chapter_list, chapter_track };
    Ok(Tag { ftyp, info, userdata })
}

fn read_chapters<T: ChunkOffsetInt>(
    reader: &mut (impl Read + Seek),
    chapters: &mut Vec<Chapter>,
    timescale: u32,
    offsets: &[T],
    stsc: &[StscItem],
    stsz_uniform_size: u32,
    stsz_sizes: &[u32],
    stts: &[SttsItem],
) -> crate::Result<()> {
    let mut time = 0;
    let mut stco_idx = 0;
    let mut stsz_iter = stsz_sizes.iter();
    let mut stts_iter = stts.iter().flat_map(|stts_item| {
        std::iter::repeat_n(stts_item.sample_duration, stts_item.sample_count as usize)
    });

    for (stsc_idx, stsc_item) in stsc.iter().enumerate() {
        let stco_end_idx = match stsc.get(stsc_idx + 1) {
            Some(next_stsc_item) => {
                let end_idx = next_stsc_item.first_chunk as usize;
                if end_idx > offsets.len() {
                    todo!("error out of bounds");
                }
                end_idx
            }
            None => offsets.len(),
        };

        for o in offsets[stco_idx..stco_end_idx].iter().copied() {
            let mut current_offset = o.into();

            for _ in 0..stsc_item.samples_per_chunk {
                let size = if stsz_uniform_size != 0 {
                    stsz_uniform_size
                } else {
                    let Some(size) = stsz_iter.next() else {
                        todo!("error");
                    };
                    *size
                };
                let Some(duration) = stts_iter.next() else { todo!("error") };

                let title = read_chapter_title(reader, current_offset)?;
                chapters.push(Chapter { start: scale_duration(timescale, time), title });

                time += duration as u64;

                current_offset += size as u64;
            }
        }

        stco_idx = stco_end_idx;
    }

    Ok(())
}

fn read_chapter_title(reader: &mut (impl Read + Seek), offset: u64) -> crate::Result<String> {
    reader.seek(SeekFrom::Start(offset))?;
    let len = reader.read_be_u16()?;
    let bom = reader.read_be_u16()?;

    // check BOM (byte order mark) for encoding
    let title = match bom {
        0xfeff => reader.read_be_utf16(len as u64 - 2)?,
        0xfffe => reader.read_le_utf16(len as u64 - 2)?,
        _ => {
            reader.skip(-2)?;
            reader.read_utf8(len as u64)?
        }
    };

    Ok(title)
}

/// A struct that configures parsing behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteConfig {
    /// Whether to overwrite item list metadata.
    pub write_item_list: bool,
    /// Whether to overwrite chapter list information.
    pub write_chapter_list: bool,
    /// Whether to overwrite chapter track information.
    pub write_chapter_track: bool,
    /// The timescale that is used to scale time for chapter list (chpl) atoms.
    pub chpl_timescale: ChplTimescale,
}

impl WriteConfig {
    /// The default configuration for writing tags.
    pub const DEFAULT: WriteConfig = WriteConfig {
        write_item_list: true,
        write_chapter_list: true,
        write_chapter_track: true,
        chpl_timescale: ChplTimescale::Fixed(DEFAULT_CHPL_TIMESCALE),
    };
}

impl Default for WriteConfig {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

#[derive(Debug)]
struct MovedData {
    new_pos: u64,
    data: Vec<u8>,
}

pub(crate) fn write_tag(file: &File, cfg: &WriteConfig, userdata: &Userdata) -> crate::Result<()> {
    let reader = &mut BufReader::new(file);

    Ftyp::parse(reader)?;

    let mut moov = None;
    let mut mdat_bounds = None;
    {
        let len = reader.remaining_stream_len()?;
        let mut parsed_bytes = 0;
        while parsed_bytes < len {
            let head = parse_head(reader)?;

            let read_cfg = ReadConfig {
                read_item_list: cfg.write_item_list,
                read_chapter_list: cfg.write_chapter_list,
                read_chapter_track: cfg.write_chapter_track,
                read_audio_info: false,
                read_image_data: false,
                ..Default::default()
            };
            let parse_cfg = ParseConfig { cfg: &read_cfg, write: true };
            match head.fourcc() {
                MOVIE => moov = Some(Moov::parse(reader, &parse_cfg, head.size())?),
                MEDIA_DATA => mdat_bounds = Some(Mdat::read_bounds(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }
    }

    let Some(mut moov) = moov else {
        return Err(crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie (moov) atom found",
        ));
    };
    let Some(mdat_bounds) = mdat_bounds else {
        return Err(crate::Error::new(
            crate::ErrorKind::AtomNotFound(MEDIA_DATA),
            "Missing necessary data, no media data (mdat) atom found",
        ));
    };

    // update atom hierarchy
    let mut append_mdat = Vec::new();
    if cfg.write_item_list || cfg.write_chapter_list || cfg.write_chapter_track {
        update_userdata(&mut moov, &mdat_bounds, &mut append_mdat, userdata, cfg);
    }

    // collect changes
    let mut changes = Vec::<Change<'_>>::new();
    moov.collect_changes(0, 0, &mut changes);
    if !append_mdat.is_empty() {
        changes.push(Change::UpdateLen(UpdateAtomLen {
            bounds: &mdat_bounds,
            fourcc: MEDIA_DATA,
            len_diff: append_mdat.len() as i64,
        }));
        changes.push(Change::AppendMdat(mdat_bounds.end(), &append_mdat));
    }

    changes.sort_by(|a, b| {
        a.old_pos().cmp(&b.old_pos()).then_with(|| {
            // Fix sorting of zero-sized changes in child atoms.
            // ```md
            // moov
            // ├─ trak
            // │  └─ tkhd (to be inserted)
            // └─ udta    (to be inserted)
            // ```
            // Given the hierarchy above, if the changes would order the insertion of the `udta`
            // atom before the `tkhd` the hierarchy would be invalid.
            if a.level() > b.level() {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    });

    // calculate mdat position shift
    let mut mdat_shift: i64 = 0;
    for c in changes.iter() {
        if matches!(c, Change::AppendMdat(..)) {
            break;
        }
        mdat_shift += c.len_diff();
    }

    // read moved data
    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let mut moved_data = Vec::new();
    let len_diff = {
        let mut current_shift: i64 = 0;
        let mut changes_iter = changes.iter().peekable();

        while let Some(change) = changes_iter.next() {
            current_shift += change.len_diff();

            let data_pos = change.old_end();
            let data_end = changes_iter.peek().map_or(old_file_len, |next| next.old_pos());
            let data_len = data_end - data_pos;

            if data_len > 0 && current_shift != 0 {
                let new_pos = (data_pos as i64 + current_shift) as u64;
                let mut data = vec![0; data_len as usize];
                reader.seek(SeekFrom::Start(data_pos))?;
                reader.read_exact(&mut data)?;

                moved_data.push(MovedData { new_pos, data });
            }
        }
        current_shift
    };

    // adjust the file length
    let new_file_len = (old_file_len as i64 + len_diff) as u64;
    file.set_len(new_file_len)?;

    let writer = &mut BufWriter::new(file);

    // write moved data
    for d in moved_data {
        writer.seek(SeekFrom::Start(d.new_pos))?;
        writer.write_all(&d.data)?;
    }

    // write changes
    let mut pos_shift = 0;
    for c in changes.iter() {
        let new_pos = c.old_pos() as i64 + pos_shift;
        writer.seek(SeekFrom::Start(new_pos as u64))?;

        match c {
            Change::UpdateLen(u) => u.update_len(writer)?,
            Change::UpdateChunkOffset(u) => u.offsets.update_offsets(writer, mdat_shift)?,
            Change::Remove(_) => (),
            Change::Replace(r) => r.atom.write(writer)?,
            Change::Insert(i) => i.atom.write(writer)?,
            Change::AppendMdat(_, d) => writer.write_all(d)?,
        }

        pos_shift += c.len_diff();
    }

    writer.flush()?;

    Ok(())
}

fn update_userdata<'a>(
    moov: &mut Moov<'a>,
    mdat_bounds: &AtomBounds,
    append_mdat: &mut Vec<u8>,
    userdata: &'a Userdata,
    cfg: &WriteConfig,
) {
    let udta = moov.udta.get_or_insert_default();

    // item list (ilst)
    if cfg.write_item_list {
        let meta = udta.meta.get_or_insert_default();
        meta.hdlr.get_or_insert_with(Hdlr::meta);

        let ilst = meta.ilst.get_or_insert_default();
        ilst.state.replace_existing();
        ilst.data = IlstData::Borrowed(&userdata.metaitems);
    }

    // chapter list
    if cfg.write_chapter_list {
        let chpl_timescale = cfg.chpl_timescale.fixed_or_mvhd(moov.mvhd.timescale);

        match udta.chpl.as_mut() {
            Some(chpl) if userdata.chapter_list.is_empty() => {
                chpl.state.remove_existing();
            }
            _ => {
                let chpl = udta.chpl.get_or_insert_default();
                chpl.state.replace_existing();
                chpl.data = ChplData::Borrowed(chpl_timescale, &userdata.chapter_list);
            }
        }
    }

    // chapter tracks
    if cfg.write_chapter_track {
        let content_trak = &mut Trak::default(); // TODO: find content track
        let chapter_timescale =
            content_trak.mdia.as_ref().map(|a| a.mdhd.timescale).unwrap_or(moov.mvhd.timescale);

        // find existing chapter tracks
        let chapter_trak_ids = (moov.trak.iter())
            .filter_map(|a| a.tref.as_ref().and_then(|a| a.chap.as_ref()))
            .flat_map(|a| a.chapter_ids.iter().copied())
            .collect::<Vec<_>>();

        if userdata.chapter_track.is_empty() {
            // TODO: remove entire track?
            // TODO: if so also remove chap references in content tracks
            // remove chapter tracks
            //for trak in moov.trak.iter_mut() {
            //    if chapter_trak_ids.contains(&trak.tkhd.id) {
            //        trak.state.remove_existing();
            //    }
            //}
            // TODO: remove media data
        }

        // generate chapter track sample table
        let duration = moov.mvhd.duration;
        let mut chunk_offsets = Vec::with_capacity(userdata.chapter_list.len());
        let mut sample_sizes = Vec::with_capacity(userdata.chapter_list.len());
        let mut time_to_samples = Vec::with_capacity(userdata.chapter_list.len());
        let mut chapters_iter = userdata.chapter_list.iter().enumerate().peekable();
        while let Some((i, c)) = chapters_iter.next() {
            let c_duration = match chapters_iter.peek() {
                Some((_, next)) => unscale_duration(chapter_timescale, next.start - c.start),
                None => unscale_duration(chapter_timescale, c.start) - duration,
            };
            time_to_samples.push(SttsItem {
                sample_count: i as u32,
                sample_duration: c_duration as u32,
            });
            sample_sizes.push(2 + c.title.len() as u32);

            // FIXME: chunk offsets need to be adjusted for the mdat shift, but this is currently
            // only done if the `stco` or `co64` already exists
            chunk_offsets.push(mdat_bounds.end());

            append_mdat.write_be_u16(c.title.len() as u16).ok();
            append_mdat.write_utf8(&c.title).ok();
        }

        // TODO: check more than the first chapter track?
        let chapter_trak =
            match moov.trak.iter_mut().find(|a| chapter_trak_ids.contains(&a.tkhd.id)) {
                Some(trak) => trak,
                None => {
                    // TODO: is this correct?
                    let new_id = moov.trak.iter().map(|t| t.tkhd.id).max().unwrap() + 1;

                    // TODO: add trak reference to content track
                    moov.trak.push_and_get(Trak {
                        state: State::Insert,
                        tkhd: Tkhd {
                            state: State::Insert,
                            id: new_id,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                }
            };

        let mdia = chapter_trak.mdia.get_or_insert_with(|| Mdia {
            state: State::Insert,
            mdhd: Mdhd {
                state: State::Insert,
                timescale: chapter_timescale,
                ..Default::default()
            },
            ..Default::default()
        });

        mdia.hdlr.get_or_insert_with(Hdlr::text_mdia);
        let minf = mdia.minf.get_or_insert_default();

        let gmhd = minf.gmhd.get_or_insert_default();
        gmhd.gmin.get_or_insert_with(Gmin::chapter);
        gmhd.text.get_or_insert_with(Text::media_information_chapter);

        let dinf = minf.dinf.get_or_insert_default();
        let dref = dinf.dref.get_or_insert_default();
        dref.url.get_or_insert_with(Url::track);

        let stbl = minf.stbl.get_or_insert_default();
        let stsd = stbl.stsd.get_or_insert_default();
        stsd.text.get_or_insert_with(Text::media_chapter);

        let stts = stbl.stts.get_or_insert_default();
        stts.items = time_to_samples;

        let stsc = stbl.stsc.get_or_insert_default();
        stsc.items = vec![StscItem {
            first_chunk: 1,
            samples_per_chunk: 1,
            sample_description_id: 1,
        }];

        let stsz = stbl.stsz.get_or_insert_default();
        stsz.sample_size = 0;
        stsz.sizes = sample_sizes;

        let co64 = stbl.co64.get_or_insert_default();
        co64.offsets = chunk_offsets;

        // TODO: remove previous chapter track data in the mdat atom
        // TODO: how to handle shift
    }
}

/// Attempts to dump the metadata atoms to the writer. This doesn't include a complete MPEG-4
/// container hierarchy and won't result in a usable file.
pub(crate) fn dump_tag(
    writer: &mut impl Write,
    cfg: &WriteConfig,
    userdata: &Userdata,
) -> crate::Result<()> {
    const MVHD_TIMESCALE: u32 = 1000;

    let duration = userdata.chapter_list.last().map_or(Duration::ZERO, |c| c.start);
    let scaled_duration = unscale_duration(MVHD_TIMESCALE, duration);

    let ftyp = Ftyp("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned());
    let mut mdat = Mdat::default();
    let mut moov = Moov {
        mvhd: Mvhd {
            version: 1,
            timescale: MVHD_TIMESCALE,
            duration: scaled_duration,
            ..Default::default()
        },
        udta: Some(Udta::default()),
        ..Default::default()
    };

    if cfg.write_item_list {
        let udta = moov.udta.get_or_insert_with(Udta::default);
        udta.meta = Some(Meta {
            state: State::Insert,
            hdlr: Some(Hdlr::meta()),
            ilst: Some(Ilst {
                state: State::Insert,
                data: IlstData::Borrowed(&userdata.metaitems),
            }),
        });
    }

    if cfg.write_chapter_list {
        let chpl_timescale = cfg.chpl_timescale.fixed_or_mvhd(MVHD_TIMESCALE);

        let udta = moov.udta.get_or_insert_with(Udta::default);
        udta.chpl = Some(Chpl {
            state: State::Insert,
            data: ChplData::Borrowed(chpl_timescale, &userdata.chapter_list),
        });
    }

    if cfg.write_chapter_track {
        let mut chunk_offsets = Vec::with_capacity(userdata.chapter_list.len());
        let mut sample_sizes = Vec::with_capacity(userdata.chapter_list.len());
        let mut time_to_samples = Vec::with_capacity(userdata.chapter_list.len());

        let mut chapters_iter = userdata.chapter_list.iter().enumerate().peekable();
        while let Some((i, c)) = chapters_iter.next() {
            let c_duration = match chapters_iter.peek() {
                Some((_, next)) => next.start - c.start,
                None => c.start - duration,
            };
            time_to_samples.push(SttsItem {
                sample_count: i as u32,
                sample_duration: unscale_duration(MVHD_TIMESCALE, c_duration) as u32,
            });
            sample_sizes.push(c.title.len() as u32 + 2);
            chunk_offsets.push(ftyp.len() + mdat.len());

            mdat.data.write_be_u16(c.title.len() as u16).ok();
            mdat.data.write_utf8(&c.title).ok();
        }

        // audio track
        moov.trak.push(Trak {
            state: State::Insert,
            tkhd: Tkhd { id: 1, ..Default::default() },
            tref: Some(Tref {
                state: State::Insert,
                chap: Some(Chap { state: State::Insert, chapter_ids: vec![2] }),
            }),
            mdia: Some(Mdia {
                mdhd: Mdhd {
                    version: 1,
                    timescale: MVHD_TIMESCALE,
                    duration: unscale_duration(MVHD_TIMESCALE, duration),
                    ..Default::default()
                },
                hdlr: Some(Hdlr::mp4a_mdia()),
                ..Default::default()
            }),
        });

        // chapter track
        moov.trak.push(Trak {
            tkhd: Tkhd { id: 2, ..Default::default() },
            mdia: Some(Mdia {
                state: State::Insert,
                mdhd: Mdhd {
                    version: 1,
                    timescale: MVHD_TIMESCALE,
                    ..Default::default()
                },
                hdlr: Some(Hdlr::text_mdia()),
                minf: Some(Minf {
                    state: State::Insert,
                    gmhd: Some(Gmhd {
                        state: State::Insert,
                        gmin: Some(Gmin::chapter()),
                        text: Some(Text::media_information_chapter()),
                    }),
                    dinf: Some(Dinf {
                        state: State::Insert,
                        dref: Some(Dref { state: State::Insert, url: Some(Url::track()) }),
                    }),
                    stbl: Some(Stbl {
                        stsd: Some(Stsd {
                            text: Some(Text::media_chapter()),
                            ..Default::default()
                        }),
                        stts: Some(Stts { state: State::Insert, items: time_to_samples }),
                        stsc: Some(Stsc {
                            state: State::Insert,
                            items: vec![StscItem {
                                first_chunk: 1,
                                samples_per_chunk: 1,
                                sample_description_id: 1,
                            }],
                        }),
                        stsz: Some(Stsz {
                            state: State::Insert,
                            sample_size: 0,
                            sizes: sample_sizes,
                        }),
                        co64: Some(Co64 { state: State::Insert, offsets: chunk_offsets }),
                        ..Default::default()
                    }),
                }),
            }),
            ..Default::default()
        });
    }

    ftyp.write(writer)?;
    mdat.write(writer)?;
    moov.write(writer)?;

    Ok(())
}
