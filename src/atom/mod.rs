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

use std::borrow::Cow;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::num::NonZeroU32;
use std::ops::Deref;

use crate::{AudioInfo, Chapter, ErrorKind, Tag, Userdata};

use change::{
    AtomRef, Change, ChunkOffsetInt, ChunkOffsets, CollectChanges, LeafAtomCollectChanges,
    SimpleCollectChanges, UpdateAtomLen, UpdateChunkOffsets,
};
use head::{AtomBounds, Head, Size, find_bounds};
use ident::*;
use state::State;
use util::*;

use chap::Chap;
use chpl::{Chpl, ChplData};
use co64::Co64;
use dinf::Dinf;
use dref::Dref;
use ftyp::Ftyp;
use gmhd::Gmhd;
use gmin::Gmin;
use hdlr::Hdlr;
use ilst::Ilst;
use mdat::Mdat;
use mdhd::Mdhd;
use mdia::Mdia;
use meta::Meta;
use minf::Minf;
use moov::Moov;
use mp4a::Mp4a;
use mvhd::Mvhd;
use stbl::{Stbl, Table};
use stco::Stco;
use stsc::{Stsc, StscItem};
use stsd::Stsd;
use stsz::Stsz;
use stts::{Stts, SttsItem};
use text::Text;
use tkhd::Tkhd;
use trak::Trak;
use tref::Tref;
use udta::Udta;
use url::*;

pub use data::Data;
pub use metaitem::MetaItem;

/// A module for working with identifiers.
pub mod ident;

#[macro_use]
mod util;
mod change;
mod head;
mod state;

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

trait AtomSize {
    fn len(&self) -> u64 {
        self.size().len()
    }

    fn size(&self) -> Size;
}

trait WriteAtom: AtomSize + Atom {
    fn write(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        match self.write_atom(writer, changes) {
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
        head::write(writer, head)
    }

    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()>;
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

impl<T: AtomSize> LenOrZero for Option<T> {
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

/// The timescale which is used for the chapter list (`chpl`).
///
/// | library          | timescale  |
/// |------------------|------------|
/// | FFMpeg (default) | 10,000,000 |
/// | mp4v2            |      1,000 |
/// | mutagen          |       mvhd |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChplTimescale {
    /// Use a fixed timescale: the number of units that pass per second.
    Fixed(NonZeroU32),
    /// Use the timescale defined in the movie header (mvhd) atom.
    Mvhd,
}

impl Default for ChplTimescale {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ChplTimescale {
    pub const DEFAULT: Self = Self::Fixed(chpl::DEFAULT_TIMESCALE);

    fn fixed_or_mvhd(self, mvhd_timescale: u32) -> u32 {
        match self {
            Self::Fixed(v) => v.get(),
            Self::Mvhd => mvhd_timescale,
        }
    }
}

/// Configure what kind of data should be rad
///
/// The item list stores tags such as the artist, album, title, and also the cover art of a song.
/// And there are two separate ways of storing chapter information:
/// - A chapter list
/// - A chapter track
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadConfig {
    /// Wheter the metatdata item list will be read.
    pub read_meta_items: bool,
    /// Wheter image data will be read, mostly for performance reasons.
    /// If disabled, images will still show up as empty [`Data`].
    pub read_image_data: bool,
    /// Wheter chapter list information will be read.
    pub read_chapter_list: bool,
    /// Wheter chapter track information will be read.
    pub read_chapter_track: bool,
    /// Wheter audio information will be read.
    /// Even if disabled, the [`AudioInfo::duration`] will be read.
    pub read_audio_info: bool,
    /// The timescale that is used to scale time for chapter list (chpl) atoms.
    pub chpl_timescale: ChplTimescale,
}

impl ReadConfig {
    /// The default configuration for reading tags.
    pub const DEFAULT: ReadConfig = ReadConfig {
        read_meta_items: true,
        read_image_data: true,
        read_chapter_list: true,
        read_chapter_track: true,
        read_audio_info: true,
        chpl_timescale: ChplTimescale::DEFAULT,
    };
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

pub struct ParseConfig<'a> {
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

        let head = head::parse(reader)?;
        if head.fourcc() == MOVIE {
            break Moov::parse(reader, &parse_cfg, head.size())?;
        }

        reader.skip(head.content_len() as i64)?;
        parsed_bytes += head.len();
    };

    let mvhd = moov.mvhd;
    let duration = scale_duration(mvhd.timescale, mvhd.duration);

    let meta_items = moov
        .udta
        .as_mut()
        .and_then(|a| a.meta.take())
        .and_then(|a| a.ilst)
        .map(|a| a.data.into_owned())
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
        // https://developer.apple.com/documentation/quicktime-file-format/chapter_lists
        // > If more than one enabled track includes a 'chap' track reference,
        // > QuickTime uses the first chapter list that it finds.
        let traks = &moov.trak;
        let chapter_trak = traks.iter().find_map(|trak| {
            let chap = trak.tref.as_ref().and_then(|tref| tref.chap.as_ref())?;
            traks.iter().find(|trak| chap.chapter_ids.contains(&trak.tkhd.id))
        });
        if let Some(trak) = chapter_trak {
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

            let stsc_items = stsc.items.get_or_read(reader)?;
            let stsz_sizes = stsz.sizes.get_or_read(reader)?;
            let stts_items = stts.items.get_or_read(reader)?;

            chapter_track.reserve(stsz_sizes.len());

            if let Some(co64) = &stbl.co64 {
                let co64_offsets = co64.offsets.get_or_read(reader)?;

                read_track_chapters(
                    reader,
                    &mut chapter_track,
                    timescale,
                    &co64_offsets,
                    &stsc_items,
                    stsz.uniform_sample_size,
                    &stsz_sizes,
                    &stts_items,
                )
                .map_err(|mut e| {
                    let mut desc = e.description.into_owned();
                    desc.insert_str(0, "Error reading chapters: ");
                    e.description = desc.into();
                    e
                })?;
            } else if let Some(stco) = &stbl.stco {
                let stco_offsets = stco.offsets.get_or_read(reader)?;

                chapter_track.reserve(stco.offsets.len());
                read_track_chapters(
                    reader,
                    &mut chapter_track,
                    timescale,
                    &stco_offsets,
                    stsc_items.as_ref(),
                    stsz.uniform_sample_size,
                    stsz_sizes.as_ref(),
                    stts_items.as_ref(),
                )
                .map_err(|mut e| {
                    let mut desc = e.description.into_owned();
                    desc.insert_str(0, "Error reading chapters: ");
                    e.description = desc.into();
                    e
                })?;
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

    let userdata = Userdata { meta_items, chapter_list, chapter_track };
    Ok(Tag { ftyp, info, userdata })
}

fn read_track_chapters<T: ChunkOffsetInt>(
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
                    return Err(crate::Error::new(
                        ErrorKind::InvalidSampleTable,
                        "Sample table sample to chunk (stsc) first chunk index is out of bounds",
                    ));
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
                        return Err(crate::Error::new(
                            ErrorKind::InvalidSampleTable,
                            "Missing sample table sample size (stsz) item",
                        ));
                    };
                    *size
                };
                let Some(duration) = stts_iter.next() else {
                    return Err(crate::Error::new(
                        ErrorKind::InvalidSampleTable,
                        "Missing sample time to sample (stts) duration",
                    ));
                };

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

/// Configure which metadata is (over)written.
///
/// The item list stores tags such as the artist, album, title, and also the cover art of a song.
/// And there are two separate ways of storing chapter information:
/// - A chapter list
/// - A chapter track
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteConfig {
    /// Whether to overwrite the metadata item list.
    pub write_meta_items: bool,
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
        write_meta_items: true,
        write_chapter_list: true,
        write_chapter_track: true,
        chpl_timescale: ChplTimescale::DEFAULT,
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
    let mut reader = BufReader::new(file);

    Ftyp::parse(&mut reader)?;

    let mut moov = None;
    let mut mdat_bounds = None;
    {
        let len = reader.remaining_stream_len()?;
        let mut parsed_bytes = 0;
        while parsed_bytes < len {
            let head = head::parse(&mut reader)?;

            let read_cfg = ReadConfig {
                read_meta_items: cfg.write_meta_items,
                read_chapter_list: cfg.write_chapter_list,
                read_chapter_track: cfg.write_chapter_track,
                read_audio_info: false,
                read_image_data: false,
                ..Default::default()
            };
            let parse_cfg = ParseConfig { cfg: &read_cfg, write: true };
            match head.fourcc() {
                MOVIE => moov = Some(Moov::parse(&mut reader, &parse_cfg, head.size())?),
                MEDIA_DATA => mdat_bounds = Some(Mdat::read_bounds(&mut reader, head.size())?),
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
    let mut changes = Vec::new();
    if cfg.write_meta_items || cfg.write_chapter_list || cfg.write_chapter_track {
        update_userdata(&mut reader, &mut changes, &mut moov, &mdat_bounds, userdata, cfg)?;
    }

    for trak in moov.trak.iter() {
        if !trak.state.is_existing() {
            continue;
        }

        let Some(stbl) = (trak.mdia.as_ref())
            .filter(|mdia| mdia.state.is_existing())
            .and_then(|mdia| mdia.minf.as_ref())
            .filter(|minf| minf.state.is_existing())
            .and_then(|minf| minf.stbl.as_ref())
            .filter(|stbl| stbl.state.is_existing())
        else {
            continue;
        };

        if let Some(co64) = &stbl.co64 {
            if let State::Existing(bounds) = &co64.state {
                let offsets = co64.offsets.get_or_read(&mut reader)?;
                let offsets = ChunkOffsets::Co64(offsets);
                let update = UpdateChunkOffsets { bounds, offsets };
                changes.push(Change::UpdateChunkOffset(update));
            }
        }
        if let Some(stco) = &stbl.stco {
            if let State::Existing(bounds) = &stco.state {
                let offsets = stco.offsets.get_or_read(&mut reader)?;
                let offsets = ChunkOffsets::Stco(offsets);
                let update = UpdateChunkOffsets { bounds, offsets };
                changes.push(Change::UpdateChunkOffset(update));
            }
        }
    }

    // collect changes
    moov.collect_changes(0, 0, &mut changes);

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
            a.level().cmp(&b.level()).reverse()
        })
    });

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

    // no more reading from here on
    drop(reader);

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
    let append_idx = changes.iter().position(|c| matches!(c, Change::AppendMdat(..)));
    let end = append_idx.unwrap_or(changes.len());
    let shifting_changes = &changes[..end];

    let mut pos_shift = 0;
    for c in changes.iter() {
        let new_pos = c.old_pos() as i64 + pos_shift;
        writer.seek(SeekFrom::Start(new_pos as u64))?;

        match c {
            Change::UpdateLen(u) => u.update_len(writer)?,
            Change::UpdateChunkOffset(u) => u.offsets.update_offsets(writer, shifting_changes)?,
            Change::Remove(_) => (),
            Change::Replace(r) => r.atom.write(writer, shifting_changes)?,
            Change::Insert(i) => i.atom.write(writer, shifting_changes)?,
            Change::RemoveMdat(_, _) => (),
            Change::AppendMdat(_, d) => writer.write_all(d)?,
        }

        pos_shift += c.len_diff();
    }

    writer.flush()?;

    Ok(())
}

fn update_userdata<'a>(
    reader: &mut (impl Read + Seek),
    changes: &mut Vec<Change<'a>>,
    moov: &mut Moov<'a>,
    mdat_bounds: &'a AtomBounds,
    userdata: &'a Userdata,
    cfg: &WriteConfig,
) -> crate::Result<()> {
    let udta = moov.udta.get_or_insert_default();

    // item list (ilst)
    if cfg.write_meta_items {
        let meta = udta.meta.get_or_insert_default();
        meta.hdlr.get_or_insert_with(Hdlr::meta);

        let ilst = meta.ilst.get_or_insert_default();
        ilst.state.replace_existing();
        ilst.data = Cow::Borrowed(&userdata.meta_items);
    }

    // chapter list
    if cfg.write_chapter_list {
        match udta.chpl.as_mut() {
            None if userdata.chapter_list.is_empty() => (),
            Some(chpl) if userdata.chapter_list.is_empty() => {
                chpl.state.remove_existing();
            }
            _ => {
                let chpl_timescale = cfg.chpl_timescale.fixed_or_mvhd(moov.mvhd.timescale);
                let chpl = udta.chpl.get_or_insert_default();
                chpl.state.replace_existing();
                chpl.data = ChplData::Borrowed(chpl_timescale, &userdata.chapter_list);
            }
        }
    }

    // chapter tracks
    'chapter_track: {
        if !cfg.write_chapter_track {
            break 'chapter_track;
        }

        // https://developer.apple.com/documentation/quicktime-file-format/chapter_lists
        // > If more than one enabled track includes a 'chap' track reference,
        // > QuickTime uses the first chapter list that it finds.
        let chapter_trak_idx = moov.trak.iter().find_map(|trak| {
            let chap = trak.tref.as_ref().and_then(|tref| tref.chap.as_ref())?;
            moov.trak.iter().position(|trak| chap.chapter_ids.contains(&trak.tkhd.id))
        });

        if userdata.chapter_track.is_empty() {
            let Some(idx) = chapter_trak_idx else {
                // avoid doing redundant work
                break 'chapter_track;
            };

            // remove chapter track
            let chapter_trak = &mut moov.trak[idx];
            chapter_trak.state.remove_existing();

            // remove all chap track references
            for trak in moov.trak.iter_mut() {
                let Some(tref) = &mut trak.tref else {
                    continue;
                };
                let State::Existing(tref_bounds) = &tref.state else {
                    continue;
                };

                let Some(chap) = &mut tref.chap else {
                    continue;
                };
                let State::Existing(chap_bounds) = &chap.state else {
                    continue;
                };

                if tref_bounds.content_len() == chap_bounds.len() {
                    tref.state.remove_existing();
                } else {
                    chap.state.remove_existing();
                }
            }

            break 'chapter_track;
        }

        // generate chapter track sample table
        let mut new_chapter_media_data = Vec::new();
        let duration = moov.mvhd.duration;
        let chapter_timescale = moov.mvhd.timescale;
        let chunk_offsets = vec![mdat_bounds.end()];
        let mut sample_sizes = Vec::with_capacity(userdata.chapter_track.len());
        let mut time_to_samples = Vec::with_capacity(userdata.chapter_track.len());
        let mut chapters_iter = userdata.chapter_track.iter().peekable();
        while let Some(c) = chapters_iter.next() {
            let c_duration = match chapters_iter.peek() {
                Some(next) => {
                    let c_duration = next.start.saturating_sub(c.start);
                    unscale_duration(chapter_timescale, c_duration)
                }
                None => {
                    let start = unscale_duration(chapter_timescale, c.start);
                    duration.saturating_sub(start)
                }
            };

            time_to_samples.push(SttsItem {
                sample_count: 1,
                sample_duration: c_duration as u32,
            });

            const ENCD: [u8; 12] = [
                0, 0, 0, 12, // size
                b'e', b'n', b'c', b'd', // fourcc
                0, 0, 1, 0, // content
            ];
            let title_len = c.title.len().min(u16::MAX as usize);
            let sample_size = 2 + title_len + ENCD.len();
            sample_sizes.push(sample_size as u32);

            new_chapter_media_data.write_be_u16(title_len as u16).ok();
            new_chapter_media_data.write_utf8(&c.title[..title_len]).ok();
            new_chapter_media_data.extend(ENCD);
        }

        let chapter_trak = match chapter_trak_idx {
            Some(idx) => &mut moov.trak[idx],
            None => {
                let new_id = moov.trak.iter().map(|t| t.tkhd.id).max().unwrap() + 1;

                // add chap track reference to all other tracks
                for trak in moov.trak.iter_mut() {
                    let tref = trak.tref.get_or_insert_default();
                    let chap = tref.chap.get_or_insert_default();
                    chap.state.replace_existing();
                    chap.chapter_ids = vec![new_id];
                }

                // add chapter track
                moov.trak.push_and_get(Trak {
                    state: State::Insert,
                    tkhd: Tkhd { version: 0, flags: [0, 0, 0], id: new_id, duration },
                    ..Default::default()
                })
            }
        };

        let mdia = chapter_trak.mdia.get_or_insert_with(|| Mdia {
            state: State::Insert,
            mdhd: Mdhd {
                timescale: chapter_timescale,
                duration,
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
        stts.state.replace_existing();
        stts.items = Table::Full(time_to_samples);

        let stsc = stbl.stsc.get_or_insert_default();
        stsc.state.replace_existing();
        let prev_stsc = std::mem::replace(
            &mut stsc.items,
            Table::Full(vec![StscItem {
                first_chunk: 1,
                samples_per_chunk: sample_sizes.len() as u32,
                sample_description_id: 1,
            }]),
        );

        let stsz = stbl.stsz.get_or_insert_default();
        stsz.state.replace_existing();
        let prev_stsz_uniform_sample_size = std::mem::replace(&mut stsz.uniform_sample_size, 0);
        let prev_stsz_sizes = std::mem::replace(&mut stsz.sizes, Table::Full(sample_sizes));

        let prev_stsc = prev_stsc.get_or_read(reader)?;
        let prev_stsz_sizes = prev_stsz_sizes.get_or_read(reader)?;

        let prev_stco = stbl.stco.as_mut().map(|stco| {
            stco.state.remove_existing();
            std::mem::take(&mut stco.offsets)
        });

        let co64 = stbl.co64.get_or_insert_default();
        co64.state.replace_existing();
        let prev_co64 = std::mem::replace(&mut co64.offsets, Table::Full(chunk_offsets));

        // remove previous chapter data from the mdat atom
        if co64.state.has_existed() {
            let prev_co64 = prev_co64.get_or_read(reader)?;
            remove_chapter_media_data(
                changes,
                &prev_co64,
                &prev_stsc,
                prev_stsz_uniform_sample_size,
                &prev_stsz_sizes,
            )?;
        } else if let Some(prev_stco) = prev_stco {
            let prev_stco = prev_stco.get_or_read(reader)?;
            remove_chapter_media_data(
                changes,
                &prev_stco,
                &prev_stsc,
                prev_stsz_uniform_sample_size,
                &prev_stsz_sizes,
            )?;
        }

        if !new_chapter_media_data.is_empty() {
            changes.push(Change::AppendMdat(mdat_bounds.end(), new_chapter_media_data));
        }

        let len_diff = changes.iter().map(|c| c.len_diff()).sum();
        if len_diff != 0 {
            changes.push(Change::UpdateLen(UpdateAtomLen {
                bounds: mdat_bounds,
                fourcc: MEDIA_DATA,
                len_diff,
            }));
        }
    }

    Ok(())
}

fn remove_chapter_media_data<T: ChunkOffsetInt>(
    changes: &mut Vec<Change<'_>>,
    offsets: &[T],
    stsc: &[StscItem],
    stsz_uniform_size: u32,
    stsz_sizes: &[u32],
) -> crate::Result<()> {
    let mut stco_idx = 0;
    let mut stsz_iter = stsz_sizes.iter();

    for (stsc_idx, stsc_item) in stsc.iter().enumerate() {
        let stco_end_idx = match stsc.get(stsc_idx + 1) {
            Some(next_stsc_item) => {
                let end_idx = next_stsc_item.first_chunk as usize;
                if end_idx > offsets.len() {
                    return Err(crate::Error::new(
                        ErrorKind::InvalidSampleTable,
                        "Sample table sample to chunk (stsc) first chunk index is out of bounds",
                    ));
                }
                end_idx
            }
            None => offsets.len(),
        };

        for o in offsets[stco_idx..stco_end_idx].iter().copied() {
            let offset = o.into();
            let chunk_size = if stsz_uniform_size != 0 {
                stsc_item.samples_per_chunk as u64 * stsz_uniform_size as u64
            } else {
                let mut chunk_size = 0;
                for _ in 0..stsc_item.samples_per_chunk {
                    let Some(size) = stsz_iter.next() else {
                        return Err(crate::Error::new(
                            ErrorKind::InvalidSampleTable,
                            "Missing sample table sample size (stsz) item",
                        ));
                    };
                    chunk_size += *size as u64;
                }
                chunk_size
            };

            changes.push(Change::RemoveMdat(offset, chunk_size));
        }

        stco_idx = stco_end_idx;
    }

    Ok(())
}
