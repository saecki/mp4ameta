//! Relevant structure of an mp4 file
//!
//! ```md
//! ftyp
//! mdat
//! moov
//! ├─ mvhd
//! ├─ trak
//! │  ├─ tkhd
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

use std::cmp;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::{AudioInfo, Chapter, ErrorKind, Tag};

use head::*;
use ident::*;
use util::*;

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
    fn parse(reader: &mut (impl Read + Seek), cfg: &ReadConfig, size: Size) -> crate::Result<Self> {
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
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self>;
}

trait FindAtom: Atom {
    type Bounds;

    fn find(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        match Self::find_atom(reader, size) {
            Err(mut e) => {
                let mut d = e.description.into_owned();
                insert_str(&mut d, "Error finding ", Self::FOURCC);
                e.description = d.into();
                Err(e)
            }
            a => a,
        }
    }

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds>;
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

/// A struct representing a timescale (the number of units that pass per second).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Timescale {
    /// Use a fixed timescale.
    Fixed(NonZeroU32),
    /// Use the timescale defined in the movie header (mvhd) atom.
    Mvhd,
}

impl Timescale {
    fn or_mvhd(self, mvhd_timescale: u32) -> u32 {
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
    /// Wheter chapter information will be read.
    pub read_chapters: bool,
    /// Wheter audio information will be read.
    pub read_audio_info: bool,

    /// The timescale that is used to scale time for chapter list (chpl) atoms.
    ///
    /// | library          | timescale  |
    /// |------------------|------------|
    /// | FFMpeg (default) | 10,000,000 |
    /// | mp4v2            |      1,000 |
    /// | mutagen          |       mvhd |
    ///
    pub chpl_timescale: Timescale,
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            read_item_list: true,
            read_image_data: true,
            read_chapters: true,
            read_audio_info: true,
            chpl_timescale: Timescale::Fixed(DEFAULT_CHPL_TIMESCALE),
        }
    }
}

pub(crate) fn read_tag(reader: &mut (impl Read + Seek), cfg: &ReadConfig) -> crate::Result<Tag> {
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
            break Moov::parse(reader, cfg, head.size())?;
        }

        reader.skip(head.content_len() as i64)?;
        parsed_bytes += head.len();
    };

    let mvhd = moov.mvhd.ok_or_else(|| {
        crate::Error::new(
            ErrorKind::AtomNotFound(MOVIE_HEADER),
            "Missing necessary data, no movie header (mvhd) atom found",
        )
    })?;
    let duration = scale_duration(mvhd.timescale, mvhd.duration);

    let metaitems = moov
        .udta
        .as_mut()
        .and_then(|a| a.meta.take())
        .and_then(|a| a.ilst)
        .and_then(|a| a.owned())
        .unwrap_or_default();

    let mut chapters = Vec::new();
    if cfg.read_chapters {
        // chapter list atom
        if let Some(mut chpl) = moov.udta.and_then(|a| a.chpl).and_then(|a| a.owned()) {
            let chpl_timescale = cfg.chpl_timescale.or_mvhd(mvhd.timescale);

            chpl.sort_by_key(|c| c.start);
            chapters.reserve(chpl.len());

            for c in chpl {
                chapters.push(Chapter {
                    start: scale_duration(chpl_timescale, c.start),
                    title: c.title,
                });
            }
        }

        // chapter tracks
        for chap in moov.trak.iter().filter_map(|a| a.tref.as_ref().and_then(|a| a.chap.as_ref())) {
            for c_id in chap.chapter_ids.iter() {
                let chapter_track =
                    moov.trak.iter().find(|a| a.tkhd.as_ref().map_or(false, |a| a.id == *c_id));

                let chapter_track = match chapter_track {
                    Some(t) => t,
                    None => continue, // TODO maybe log warning: referenced chapter track not found
                };

                let mdia = chapter_track.mdia.as_ref();
                let stbl = mdia.and_then(|a| a.minf.as_ref()).and_then(|a| a.stbl.as_ref());
                let stts = stbl.and_then(|a| a.stts.as_ref());

                let timescale = mdia
                    .and_then(|a| a.mdhd.as_ref().map(|a| a.timescale))
                    .unwrap_or(mvhd.timescale);

                if let Some(stco) = stbl.and_then(|a| a.stco.as_ref()) {
                    chapters.reserve(stco.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapters,
                        timescale,
                        stco.offsets.iter().map(|o| *o as u64),
                        stts.map_or([].iter(), |a| a.items.iter()),
                    )?;
                } else if let Some(co64) = stbl.and_then(|a| a.co64.as_ref()) {
                    chapters.reserve(co64.offsets.len());
                    read_chapters(
                        reader,
                        &mut chapters,
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

    Ok(Tag { ftyp, info, metaitems, chapters })
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
    /// Wheter to overwrite item list metadata.
    pub write_item_list: bool,
    /// Wheter to overwrite chapter information.
    pub write_chapters: bool,

    /// The timescale that is used to scale time for chapter list (chpl) atoms.
    ///
    /// | library          | timescale  |
    /// |------------------|------------|
    /// | FFMpeg (default) | 10,000,000 |
    /// | mp4v2            |      1,000 |
    /// | mutagen          |       mvhd |
    ///
    pub chpl_timescale: Timescale,
}

impl Default for WriteConfig {
    fn default() -> Self {
        Self {
            write_item_list: true,
            write_chapters: true,
            chpl_timescale: Timescale::Fixed(DEFAULT_CHPL_TIMESCALE),
        }
    }
}

trait LenDiff {
    fn len_diff(&self) -> i64;
}

impl<T: WriteAtom> LenDiff for Option<NewAtom<T>> {
    fn len_diff(&self) -> i64 {
        self.as_ref().map_or(0, |a| a.len_diff())
    }
}

#[derive(Debug)]
struct NewAtom<T> {
    old_pos: u64,
    old_end: u64,
    atom: T,
}

impl<T> NewAtom<T> {
    const fn old_len(&self) -> u64 {
        self.old_end - self.old_pos
    }
}

impl<T: WriteAtom> NewAtom<T> {
    fn new_len(&self) -> u64 {
        self.atom.len()
    }

    fn len_diff(&self) -> i64 {
        self.new_len() as i64 - self.old_len() as i64
    }
}

#[derive(Debug)]
enum AtomRef<'a> {
    Udta(&'a Udta<'a>),
    Chpl(&'a Chpl<'a>),
    Meta(&'a Meta<'a>),
    Hdlr(&'a Hdlr),
    Ilst(&'a Ilst<'a>),
}

impl AtomRef<'_> {
    fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self {
            AtomRef::Udta(a) => a.write(writer),
            AtomRef::Chpl(a) => a.write(writer),
            AtomRef::Meta(a) => a.write(writer),
            AtomRef::Hdlr(a) => a.write(writer),
            AtomRef::Ilst(a) => a.write(writer),
        }
    }

    fn len(&self) -> u64 {
        match self {
            AtomRef::Udta(a) => a.len(),
            AtomRef::Chpl(a) => a.len(),
            AtomRef::Meta(a) => a.len(),
            AtomRef::Hdlr(a) => a.len(),
            AtomRef::Ilst(a) => a.len(),
        }
    }
}

#[derive(Debug)]
struct UpdateAtom<'a> {
    old_bounds: &'a AtomBounds,
    len_diff: i64,
}

impl UpdateAtom<'_> {
    fn new_len(&self) -> u64 {
        (self.old_bounds.len() as i64 + self.len_diff) as u64
    }
}

#[derive(Debug)]
struct MovedData {
    new_pos: u64,
    data: Vec<u8>,
}

pub(crate) fn write_tag(
    file: &File,
    cfg: &WriteConfig,
    metaitems: &[MetaItem],
    chapters: &[Chapter],
) -> crate::Result<()> {
    let mut reader = BufReader::new(file);
    let reader = &mut reader;

    Ftyp::parse(reader)?;

    let mut moov = None;
    let mut mdat = None;

    {
        let len = reader.remaining_stream_len()?;
        let mut parsed_bytes = 0;
        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MOVIE => moov = Some(Moov::find(reader, head.size())?),
                MEDIA_DATA => mdat = Some(Mdat::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }
    }

    let mdat_pos = mdat.map_or(0, |a| a.pos());
    let moov = moov.ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie (moov) atom found",
        )
    })?;
    let mvhd = moov.mvhd.as_ref().ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie header (mvhd) atom found",
        )
    })?;
    let udta = &moov.udta;
    let chpl = udta.as_ref().and_then(|a| a.chpl.as_ref());
    let meta = udta.as_ref().and_then(|a| a.meta.as_ref());
    let hdlr = meta.as_ref().and_then(|a| a.hdlr.as_ref());
    let ilst = meta.as_ref().and_then(|a| a.ilst.as_ref());

    let mut update_atoms = Vec::new();
    let mut new_atoms = Vec::new();

    let mut new_udta = None;
    let mut new_chpl = None;
    let mut _chpl = Vec::new();
    let mut new_meta = None;
    let mut new_hdlr = None;
    let mut new_ilst = None;

    // check wich atoms are missing
    if cfg.write_item_list {
        if hdlr.is_none() {
            new_hdlr = Some(NewAtom { old_pos: 0, old_end: 0, atom: Hdlr::meta() });
        }
        match ilst {
            Some(ilst) => {
                new_ilst = Some(NewAtom {
                    old_pos: ilst.pos(),
                    old_end: ilst.end(),
                    atom: Ilst::Borrowed(metaitems),
                });
            }
            None => {
                new_ilst = Some(NewAtom {
                    old_pos: 0,
                    old_end: 0,
                    atom: Ilst::Borrowed(metaitems),
                });
            }
        }

        match meta {
            Some(meta) => {
                if hdlr.is_none() {
                    if let Some(a) = &mut new_hdlr {
                        a.old_pos = meta.content_pos();
                        a.old_end = meta.content_pos();
                    }
                }
                if ilst.is_none() {
                    if let Some(a) = &mut new_ilst {
                        a.old_pos = meta.end();
                        a.old_end = meta.end();
                    }
                }

                update_atoms.push(UpdateAtom {
                    old_bounds: &meta.bounds,
                    len_diff: new_hdlr.len_diff() + new_ilst.len_diff(),
                });
            }
            None => {
                new_meta = Some(NewAtom {
                    old_pos: 0,
                    old_end: 0,
                    atom: Meta {
                        hdlr: new_hdlr.take().map(|a| a.atom),
                        ilst: new_ilst.take().map(|a| a.atom),
                    },
                });
            }
        }
    }

    if cfg.write_chapters {
        let chpl_timescale = cfg.chpl_timescale.or_mvhd(mvhd.timescale);
        _chpl = chapters
            .iter()
            .map(|c| BorrowedChplItem {
                start: unscale_duration(chpl_timescale, c.start),
                title: &c.title,
            })
            .collect();

        match chpl {
            Some(chpl) => {
                new_chpl = Some(NewAtom {
                    old_pos: chpl.pos(),
                    old_end: chpl.end(),
                    atom: Chpl::Borrowed(&_chpl),
                });
            }
            None => {
                new_chpl = Some(NewAtom {
                    old_pos: 0,
                    old_end: 0,
                    atom: Chpl::Borrowed(&_chpl),
                });
            }
        }
    }

    if cfg.write_item_list || cfg.write_chapters {
        match udta {
            Some(udta) => {
                if meta.is_none() {
                    if let Some(a) = &mut new_meta {
                        a.old_pos = udta.end();
                        a.old_end = udta.end();
                    }
                }
                if chpl.is_none() {
                    if let Some(a) = &mut new_chpl {
                        a.old_pos = udta.end();
                        a.old_end = udta.end();
                    }
                }

                update_atoms.push(UpdateAtom {
                    old_bounds: &udta.bounds,
                    len_diff: new_chpl.len_diff()
                        + new_meta.len_diff()
                        + new_hdlr.len_diff()
                        + new_ilst.len_diff(),
                });
            }
            None => {
                new_udta = Some(NewAtom {
                    old_pos: moov.end(),
                    old_end: moov.end(),
                    atom: Udta {
                        chpl: new_chpl.take().map(|a| a.atom),
                        meta: new_meta.take().map(|a| a.atom),
                    },
                });
            }
        }

        update_atoms.push(UpdateAtom {
            old_bounds: &moov.bounds,
            len_diff: new_udta.len_diff()
                + new_chpl.len_diff()
                + new_meta.len_diff()
                + new_hdlr.len_diff()
                + new_ilst.len_diff(),
        });
    }

    // add new atoms to the list
    if let Some(a) = &new_udta {
        new_atoms.push(NewAtom {
            old_pos: a.old_pos,
            old_end: a.old_end,
            atom: AtomRef::Udta(&a.atom),
        });
    }
    if let Some(a) = &new_chpl {
        new_atoms.push(NewAtom {
            old_pos: a.old_pos,
            old_end: a.old_end,
            atom: AtomRef::Chpl(&a.atom),
        });
    }
    if let Some(a) = &new_meta {
        new_atoms.push(NewAtom {
            old_pos: a.old_pos,
            old_end: a.old_end,
            atom: AtomRef::Meta(&a.atom),
        });
    }
    if let Some(a) = &new_ilst {
        new_atoms.push(NewAtom {
            old_pos: a.old_pos,
            old_end: a.old_end,
            atom: AtomRef::Ilst(&a.atom),
        });
    }
    if let Some(a) = &new_hdlr {
        new_atoms.push(NewAtom {
            old_pos: a.old_pos,
            old_end: a.old_pos,
            atom: AtomRef::Hdlr(&a.atom),
        });
    }

    new_atoms.sort_by_key(|a| a.old_pos);

    let mut writer = BufWriter::new(file);
    let writer = &mut writer;

    let mut mdat_shift: i64 = 0;
    for a in new_atoms.iter() {
        if a.old_pos <= mdat_pos {
            mdat_shift += a.atom.len() as i64 - a.old_len() as i64;
        }
    }

    // adjust sample table chunk offsets
    if mdat_shift != 0 {
        let stbl_atoms = moov.trak.iter().filter_map(|a| {
            a.mdia.as_ref().and_then(|a| a.minf.as_ref()).and_then(|a| a.stbl.as_ref())
        });
        let parse_cfg = ReadConfig::default();

        for stbl in stbl_atoms {
            if let Some(a) = &stbl.stco {
                reader.seek(SeekFrom::Start(a.content_pos()))?;
                let chunk_offset = Stco::parse(reader, &parse_cfg, a.size())?;

                writer.seek(SeekFrom::Start(a.content_pos() + 8))?;
                for co in chunk_offset.offsets.iter() {
                    let new_offset = (*co as i64 + mdat_shift) as u32;
                    writer.write_be_u32(new_offset)?;
                }
                writer.flush()?;
            }
            if let Some(a) = &stbl.co64 {
                reader.seek(SeekFrom::Start(a.content_pos()))?;
                let chunk_offset = Co64::parse(reader, &parse_cfg, a.size())?;

                writer.seek(SeekFrom::Start(a.content_pos() + 8))?;
                for co in chunk_offset.offsets.iter() {
                    let new_offset = (*co as i64 + mdat_shift) as u64;
                    writer.write_be_u64(new_offset)?;
                }
                writer.flush()?;
            }
        }
    }

    // update changed atom lengths
    for a in update_atoms.iter().rev() {
        writer.seek(SeekFrom::Start(a.old_bounds.pos()))?;
        if a.old_bounds.ext() {
            writer.write_be_u32(1)?;
            writer.skip(4)?;
            writer.write_be_u64(a.new_len())?;
        } else {
            writer.write_be_u32(a.new_len() as u32)?;
        }
        writer.flush()?;
    }

    // read moved data
    let mut reader = BufReader::new(file);
    let reader = &mut reader;

    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let mut len_diff: i64 = 0;
    let mut moved_data = Vec::new();
    {
        let mut new_atoms_iter = new_atoms.iter().peekable();

        while let Some(a) = new_atoms_iter.next() {
            len_diff += a.atom.len() as i64 - a.old_len() as i64;

            let data_pos = a.old_end;
            let data_end = new_atoms_iter.peek().map_or(old_file_len, |next| next.old_pos);
            let data_len = data_end - data_pos;
            let new_pos = (data_pos as i64 + len_diff) as u64;

            let mut data = vec![0; data_len as usize];
            reader.seek(SeekFrom::Start(data_pos))?;
            reader.read_exact(&mut data)?;

            moved_data.push(MovedData { new_pos, data });
        }
    }

    // adjust the file length
    let new_file_len = (old_file_len as i64 + len_diff) as u64;
    file.set_len(new_file_len)?;

    let mut writer = BufWriter::new(file);
    let writer = &mut writer;

    // writing moved data
    for d in moved_data {
        writer.seek(SeekFrom::Start(d.new_pos))?;
        writer.write_all(&d.data)?;
        writer.flush()?;
    }

    // write new atoms
    {
        let mut pos_shift = 0;
        for a in new_atoms.iter() {
            let new_pos = a.old_pos as i64 + pos_shift;

            writer.seek(SeekFrom::Start(new_pos as u64))?;
            a.atom.write(writer)?;
            writer.flush()?;

            pos_shift += a.atom.len() as i64 - a.old_len() as i64;
        }
    }

    Ok(())
}

/// Attempts to dump the metadata atoms to the writer. This doesn't include a complete MPEG-4
/// container hierarchy and won't result in a usable file.
pub(crate) fn dump_tag(writer: &mut impl Write, cfg: &WriteConfig, tag: &Tag) -> crate::Result<()> {
    const MVHD_TIMESCALE: u32 = 1000;
    let Tag { metaitems, chapters, info, .. } = tag;

    let chapter_start = chapters.last().map_or(Duration::ZERO, |c| c.start);
    let duration = cmp::max(info.duration, chapter_start);
    let scaled_duration = unscale_duration(MVHD_TIMESCALE, duration);

    let ftyp = Ftyp("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned());
    let mdat = Mdat::default();
    let mut moov = Moov {
        mvhd: Some(Mvhd {
            version: 1,
            timescale: MVHD_TIMESCALE,
            duration: scaled_duration,
            ..Default::default()
        }),
        udta: Some(Udta::default()),
        ..Default::default()
    };

    if cfg.write_item_list && !metaitems.is_empty() {
        let udta = moov.udta.get_or_insert_with(Udta::default);
        udta.meta = Some(Meta {
            hdlr: Some(Hdlr::meta()),
            ilst: Some(Ilst::Borrowed(metaitems)),
        });
    }

    let mut _chpl = Vec::new();
    if cfg.write_chapters && !chapters.is_empty() {
        let chpl_timescale = cfg.chpl_timescale.or_mvhd(MVHD_TIMESCALE);

        let udta = moov.udta.get_or_insert_with(Udta::default);
        _chpl = chapters
            .iter()
            .map(|c| BorrowedChplItem {
                start: unscale_duration(chpl_timescale, c.start),
                title: &c.title,
            })
            .collect();

        udta.chpl = Some(Chpl::Borrowed(&_chpl));
    }

    ftyp.write(writer)?;
    mdat.write(writer)?;
    moov.write(writer)?;

    Ok(())
}
