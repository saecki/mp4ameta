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

    dbg!(&moov);

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
        if let Some(mut chpl) = moov.udta.and_then(|a| a.chpl).and_then(|a| a.owned()) {
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

                let mdia = trak.mdia.as_ref();
                let stbl = mdia.and_then(|a| a.minf.as_ref()).and_then(|a| a.stbl.as_ref());
                let stts = stbl.and_then(|a| a.stts.as_ref());

                let timescale = mdia.map(|a| a.mdhd.timescale).unwrap_or(mvhd.timescale);

                if let Some(stco) = stbl.and_then(|a| a.stco.as_ref()) {
                    chapter_track.reserve(stco.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapter_track,
                        timescale,
                        stco.offsets.iter().map(|o| *o as u64),
                        stts.map_or([].iter(), |a| a.items.iter()),
                    )?;
                } else if let Some(co64) = stbl.and_then(|a| a.co64.as_ref()) {
                    chapter_track.reserve(co64.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapter_track,
                        timescale,
                        co64.offsets.iter().copied(),
                        stts.map_or([].iter(), |a| a.items.iter()),
                    )?;
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

fn read_chapters<'a>(
    reader: &mut (impl Read + Seek),
    chapters: &mut Vec<Chapter>,
    timescale: u32,
    offsets: impl Iterator<Item = u64>,
    mut durations: impl Iterator<Item = &'a SttsItem>,
) -> crate::Result<()> {
    let mut start = 0;

    for o in offsets {
        let duration = durations.next().map_or(0, |i| i.sample_duration) as u64;
        let title = read_chapter_title(reader, o)?;
        chapters.push(Chapter { start: scale_duration(timescale, start), title });

        start += duration;
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

    // update atom hierachty
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
        if a.old_pos() < b.old_pos() {
            Ordering::Less
        } else if a.old_pos() > b.old_pos() {
            Ordering::Greater
        } else if a.level() > b.level() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
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
    let mut len_diff: i64 = 0;
    let mut moved_data = Vec::new();
    {
        let mut changes_iter = changes.iter().peekable();

        while let Some(a) = changes_iter.next() {
            len_diff += a.len_diff();

            let data_pos = a.old_end();
            let data_end = changes_iter.peek().map_or(old_file_len, |next| next.old_pos());
            let data_len = data_end - data_pos;

            if data_len > 0 {
                let new_pos = (data_pos as i64 + len_diff) as u64;
                let mut data = vec![0; data_len as usize];
                reader.seek(SeekFrom::Start(data_pos))?;
                reader.read_exact(&mut data)?;

                moved_data.push(MovedData { new_pos, data });
            }
        }
    }

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
    let udta = moov.udta.get_or_insert(Udta { state: State::Insert, ..Default::default() });

    // item list (ilst)
    if cfg.write_item_list {
        let meta = udta.meta.get_or_insert(Meta { state: State::Insert, ..Default::default() });

        meta.hdlr.get_or_insert_with(Hdlr::meta);

        match meta.ilst.as_mut() {
            Some(ilst) => {
                ilst.state.replace_existing();
                ilst.data = IlstData::Borrowed(&userdata.metaitems);
            }
            None => {
                meta.ilst = Some(Ilst {
                    state: State::Insert,
                    data: IlstData::Borrowed(&userdata.metaitems),
                });
            }
        }
    }

    // chapter list
    if cfg.write_chapter_list {
        let chpl_timescale = cfg.chpl_timescale.fixed_or_mvhd(moov.mvhd.timescale);
        let chpl_items = userdata
            .chapter_list
            .iter()
            .map(|c| BorrowedChplItem {
                start: unscale_duration(chpl_timescale, c.start),
                title: &c.title,
            })
            .collect();

        match udta.chpl.as_mut() {
            Some(chpl) if userdata.chapter_list.is_empty() => {
                chpl.state.remove_existing();
            }
            Some(chpl) => {
                chpl.state.replace_existing();
                chpl.data = ChplData::Borrowed(chpl_items);
            }
            None => {
                udta.chpl = Some(Chpl {
                    state: State::Insert,
                    data: ChplData::Borrowed(chpl_items),
                });
            }
        }
    }

    // chapter tracks
    if cfg.write_chapter_track {
        // TODO: remove entire track?
        // remove chapter tracks
        //for trak in moov.trak.iter_mut() {
        //    if chapter_track_ids.contains(&trak.tkhd.id) {
        //        trak.state.remove_existing();
        //    }
        //}

        let mut chapter_track_ids = Vec::new();
        for tref in moov.trak.iter_mut().filter_map(|a| a.tref.as_mut()) {
            if let Some(chap) = &mut tref.chap {
                chap.state.remove_existing();

                if let State::Existing(bounds) = &tref.state {
                    if bounds.content_len() == chap.len() {
                        tref.state.remove_existing();
                    }
                }

                chapter_track_ids.extend(chap.chapter_ids.iter().copied());
            }
        }

        let content_trak = &Trak::default(); // TODO: find content track
        let chapter_timescale =
            content_trak.mdia.as_ref().map(|a| a.mdhd.timescale).unwrap_or(moov.mvhd.timescale);
        // TODO: add trak reference to content track

        let new_id = moov.trak.iter().map(|t| t.tkhd.id).max().unwrap() + 1; // TODO: is this correct?
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

            // chunk offsets will be adjusted for the mdat shift later on
            chunk_offsets.push(mdat_bounds.end());

            append_mdat.write_be_u16(c.title.len() as u16).ok();
            append_mdat.write_utf8(&c.title).ok();
        }

        let chapter_track = Trak {
            state: State::Insert,
            tkhd: Tkhd {
                state: State::Insert,
                id: new_id,
                ..Default::default()
            },
            mdia: Some(Mdia {
                state: State::Insert,
                mdhd: Mdhd {
                    state: State::Insert,
                    timescale: chapter_timescale,
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
        };
        moov.trak.push(chapter_track);
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
        let chpl_items = userdata
            .chapter_list
            .iter()
            .map(|c| BorrowedChplItem {
                start: unscale_duration(chpl_timescale, c.start),
                title: &c.title,
            })
            .collect();

        udta.chpl = Some(Chpl {
            state: State::Insert,
            data: ChplData::Borrowed(chpl_items),
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
