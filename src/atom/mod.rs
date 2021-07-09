//! Relevant structure of an mp4 file
//!
//! ftyp
//! mdat
//! moov
//! ├─ mvhd
//! ├─ trak
//! │  ├─ tkhd
//! │  └─ mdia
//! │     ├─ mdhd
//! │     └─ minf
//! │        └─ stbl
//! │           ├─ stsd
//! │           │  └─ mp4a
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

use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

use crate::{AudioInfo, ErrorKind, Tag};

use content::*;
use head::*;
use template::*;

use co64::*;
use ftyp::*;
use hdlr::*;
use ilst::*;
use mdia::*;
use meta::*;
use minf::*;
use moov::*;
use mp4a::*;
use mvhd::*;
use stbl::*;
use stco::*;
use stsd::*;
use trak::*;
use udta::*;

pub use data::*;
pub use ident::*;

#[macro_use]
pub mod data;
/// A module for working with identifiers.
pub mod ident;

mod content;
mod head;
mod template;

mod co64;
mod ftyp;
mod hdlr;
mod ilst;
mod mdia;
mod meta;
mod minf;
mod moov;
mod mp4a;
mod mvhd;
mod stbl;
mod stco;
mod stsd;
mod trak;
mod udta;

/// A struct representing data that is associated with an atom identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomData {
    /// The identifier of the atom.
    pub ident: DataIdent,
    /// The data contained in the atom.
    pub data: Vec<Data>,
}

impl AtomData {
    /// Creates atom data with the identifier and data.
    pub const fn new(ident: DataIdent, data: Vec<Data>) -> Self {
        Self { ident, data }
    }

    /// Returns the external length of the atom in bytes.
    pub fn len(&self) -> u64 {
        let parent_len = 8;
        let data_len: u64 = self.data.iter().map(|d| 16 + d.len()).sum();

        match &self.ident {
            DataIdent::Fourcc(_) => parent_len + data_len,
            DataIdent::Freeform { mean, name } => {
                let mean_len = 12 + mean.len() as u64;
                let name_len = 12 + name.len() as u64;

                parent_len + mean_len + name_len + data_len
            }
        }
    }

    /// Returns whether the inner data atom is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() || self.data.iter().all(|d| d.is_empty())
    }

    fn parse(reader: &mut (impl Read + Seek), parent: Fourcc, len: u64) -> crate::Result<Self> {
        let mut data = Vec::new();
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                DATA => {
                    let (version, flags) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }
                    let [b2, b1, b0] = flags;
                    let datatype = u32::from_be_bytes([0, b2, b1, b0]);

                    // Skipping 4 byte locale indicator
                    reader.seek(SeekFrom::Current(4))?;

                    data.push(data::parse_data(reader, datatype, head.content_len() - 8)?);
                }
                MEAN => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    mean = Some(data::read_utf8(reader, head.content_len() - 4)?);
                }
                NAME => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    name = Some(data::read_utf8(reader, head.content_len() - 4)?);
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        let ident = match (parent, mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (ident, _, _) => DataIdent::Fourcc(ident),
        };

        if data.is_empty() {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(DATA),
                format!("Error constructing atom data '{}', missing data atom", parent),
            ));
        }

        Ok(AtomData { ident, data })
    }

    /// Attempts to write the atom data to the writer.
    pub fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&u32::to_be_bytes(self.len() as u32))?;

        match &self.ident {
            DataIdent::Fourcc(ident) => writer.write_all(ident.deref())?,
            _ => {
                let (mean, name) = match &self.ident {
                    DataIdent::Freeform { mean, name } => (mean.as_str(), name.as_str()),
                    DataIdent::Fourcc(_) => unreachable!(),
                };
                writer.write_all(FREEFORM.deref())?;

                let mean_len: u32 = 12 + mean.len() as u32;
                writer.write_all(&u32::to_be_bytes(mean_len))?;
                writer.write_all(MEAN.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(mean.as_bytes())?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_all(&u32::to_be_bytes(name_len))?;
                writer.write_all(NAME.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(name.as_bytes())?;
            }
        }

        for d in self.data.iter() {
            let data_len: u32 = 16 + d.len() as u32;
            writer.write_all(&u32::to_be_bytes(data_len))?;
            writer.write_all(DATA.deref())?;
            d.write_typed(writer)?;
        }

        Ok(())
    }
}

/// A template representing a MPEG-4 audio metadata atom.
#[derive(Clone, Eq, PartialEq)]
struct AtomT {
    /// The 4 byte identifier of the atom.
    ident: Fourcc,
    /// The offset in bytes separating the head from the content.
    offset: u64,
    /// The content template of an atom template.
    content: ContentT,
}

impl fmt::Debug for AtomT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomT {{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl AtomT {
    /// Creates an atom template containing the provided content at a n byte offset.
    const fn new(ident: Fourcc, offset: u64, content: ContentT) -> Self {
        Self { ident, offset, content }
    }
}

/// A struct a hierarchy of atom bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundAtom {
    bounds: AtomBounds,
    atoms: Vec<FoundAtom>,
}

impl Deref for FoundAtom {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl DerefMut for FoundAtom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bounds
    }
}

impl FoundAtom {
    const fn new(bounds: AtomBounds, atoms: Vec<FoundAtom>) -> Self {
        FoundAtom { bounds, atoms }
    }
}

/// Finds out the position and size of any atoms matching the template hierarchy.
fn find_atoms(
    reader: &mut (impl Read + Seek),
    atoms: &[AtomT],
    len: u64,
) -> crate::Result<Vec<FoundAtom>> {
    let mut found_atoms = Vec::new();
    let mut pos = 0;

    while pos < len {
        let head = parse_head(reader)?;

        match atoms.iter().find(|a| a.ident == head.fourcc()) {
            Some(a) => {
                let atom_pos = reader.seek(SeekFrom::Current(0))? - head.head_len();
                let bounds = AtomBounds::new(atom_pos, head);

                match &a.content {
                    ContentT::Atoms(c) if !c.is_empty() => {
                        if a.offset != 0 {
                            reader.seek(SeekFrom::Current(a.offset as i64))?;
                        }
                        match find_atoms(reader, c, head.content_len() - a.offset) {
                            Ok(a) => found_atoms.push(FoundAtom::new(bounds, a)),
                            Err(e) => {
                                return Err(crate::Error::new(
                                    e.kind,
                                    format!("Error finding {}: {}", head.fourcc(), e.description),
                                ));
                            }
                        }
                    }
                    _ => {
                        reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                        found_atoms.push(FoundAtom::new(bounds, Vec::new()));
                    }
                }
            }
            None => {
                reader.seek(SeekFrom::Current(head.content_len() as i64))?;
            }
        }

        pos += head.len();
    }

    Ok(found_atoms)
}

trait Atom: Sized {
    const FOURCC: Fourcc;
}

trait ParseAtom: Atom {
    fn parse(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        match Self::parse_atom(reader, size) {
            Err(mut e) => {
                e.description = format!("Error parsing {}: {}", Self::FOURCC, e.description);
                Err(e)
            }
            a => a,
        }
    }

    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self>;
}

trait WriteAtom: Atom {
    fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self.write_atom(writer) {
            Err(mut e) => {
                e.description = format!("Error writing {}: {}", Self::FOURCC, e.description);
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

trait LenOrZero {
    fn len_or_zero(&self) -> u64;
}

impl<T: WriteAtom> LenOrZero for Option<T> {
    fn len_or_zero(&self) -> u64 {
        self.as_ref().map_or(0, |a| a.len())
    }
}

/// Attempts to read MPEG-4 audio metadata from the reader.
pub(crate) fn read_tag_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
    let Ftyp(ftyp) = Ftyp::parse(reader)?;

    let len = data::remaining_stream_len(reader)?;
    let mut parsed_bytes = 0;
    let moov = loop {
        if parsed_bytes >= len {
            return Err(crate::Error::new(
                ErrorKind::AtomNotFound(MOVIE),
                "Missing necessary data, no movie (moov) atom found".to_owned(),
            ));
        }

        let head = parse_head(reader)?;

        match head.fourcc() {
            MOVIE => {
                break Moov::parse(reader, head.size())?;
            }
            _ => {
                reader.seek(SeekFrom::Current(head.content_len() as i64))?;
            }
        }

        parsed_bytes += head.len();
    };

    let mvhd = moov.mvhd;
    let mp4a = moov.trak.into_iter().find_map(|trak| {
        trak.mdia
            .and_then(|mdia| mdia.minf)
            .and_then(|minf| minf.stbl)
            .and_then(|stbl| stbl.stsd)
            .and_then(|stsd| stsd.mp4a)
    });
    let ilst = moov
        .udta
        .and_then(|udta| udta.meta)
        .and_then(|meta| meta.ilst)
        .and_then(|ilst| ilst.owned())
        .unwrap_or_default();

    let mut info = AudioInfo::default();
    if let Some(i) = mvhd {
        info.duration = Some(i.duration);
    }
    if let Some(i) = mp4a {
        info.channel_config = i.channel_config;
        info.sample_rate = i.sample_rate;
        info.max_bitrate = i.max_bitrate;
        info.avg_bitrate = i.avg_bitrate;
    }

    Ok(Tag::new(ftyp, info, ilst))
}

/// Attempts to write the metadata atoms to the file inside the item list atom.
pub(crate) fn write_tag_to(file: &File, atoms: &[AtomData]) -> crate::Result<()> {
    let mut reader = BufReader::new(file);

    Ftyp::parse(&mut reader)?;

    let len = data::remaining_stream_len(&mut reader)?;
    let found_atoms = find_atoms(&mut reader, METADATA_WRITE_ATOM_T.deref(), len)?;

    let mdat_pos =
        found_atoms.iter().find(|a| a.fourcc() == MEDIA_DATA).map(|a| a.pos()).unwrap_or(0);
    let moov = found_atoms.iter().find(|a| a.fourcc() == MOVIE).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie (moov) atom found".to_owned(),
        )
    })?;
    let udta = moov.atoms.iter().find(|a| a.fourcc() == USER_DATA);
    let meta = udta.and_then(|a| a.atoms.iter().find(|a| a.fourcc() == METADATA));
    let hdlr = meta.and_then(|a| a.atoms.iter().find(|a| a.fourcc() == HANDLER_REFERENCE));
    let ilst = meta.and_then(|a| a.atoms.iter().find(|a| a.fourcc() == ITEM_LIST));

    let mut new_atoms_start = 0;
    let mut moved_data_start = 0;
    let mut len_diff = 0;

    let mut update_atoms = Vec::new();
    let mut new_udta = None;
    let mut new_meta = None;
    let mut new_hdlr = None;
    let new_ilst = Ilst::Borrowed(atoms);

    if hdlr.is_none() {
        new_hdlr = Some(Meta::hdlr());
    }
    if let Some(ilst) = ilst {
        new_atoms_start = ilst.pos();
        moved_data_start = ilst.end();
        len_diff -= ilst.len() as i64;
    }

    match meta {
        Some(meta) => {
            update_atoms.push(meta);
            if ilst.is_none() {
                new_atoms_start = meta.end();
                moved_data_start = meta.end();
            }
        }
        None => {
            new_meta = Some(Meta { hdlr: new_hdlr.take(), ilst: Some(new_ilst.clone()) });
        }
    }
    match udta {
        Some(udta) => {
            update_atoms.push(udta);
            if meta.is_none() {
                new_atoms_start = udta.end();
                moved_data_start = udta.end();
            }
        }
        None => {
            new_udta = Some(Udta { meta: new_meta.take() });
            new_atoms_start = moov.end();
            moved_data_start = moov.end();
        }
    }
    update_atoms.push(moov);

    let new_atom_len = if let Some(a) = &new_udta {
        a.len()
    } else if let Some(a) = &new_meta {
        a.len()
    } else {
        new_hdlr.len_or_zero() + new_ilst.len()
    };
    len_diff += new_atom_len as i64;

    // reading moved data
    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let moved_data_len = old_file_len - moved_data_start;
    let mut moved_data = Vec::with_capacity(moved_data_len as usize);
    reader.seek(SeekFrom::Start(moved_data_start))?;
    reader.read_to_end(&mut moved_data)?;

    let mut writer = BufWriter::new(file);

    // adjusting sample table chunk offsets
    if mdat_pos > moov.pos() {
        let stbl_atoms = moov
            .atoms
            .iter()
            .filter(|a| a.fourcc() == TRACK)
            .filter_map(|a| a.atoms.iter().find(|a| a.fourcc() == MEDIA))
            .filter_map(|a| a.atoms.iter().find(|a| a.fourcc() == MEDIA_INFORMATION))
            .filter_map(|a| a.atoms.iter().find(|a| a.fourcc() == SAMPLE_TABLE));

        for stbl in stbl_atoms {
            for a in stbl.atoms.iter() {
                match a.fourcc() {
                    SAMPLE_TABLE_CHUNK_OFFSET => {
                        reader.seek(SeekFrom::Start(a.content_pos()))?;
                        let chunk_offset = Stco::parse(&mut reader, a.size())?;

                        writer.seek(SeekFrom::Start(chunk_offset.table_pos))?;
                        for co in chunk_offset.offsets.iter() {
                            let new_offset = (*co as i64 + len_diff) as u32;
                            writer.write_all(&u32::to_be_bytes(new_offset))?;
                        }
                        writer.flush()?;
                    }
                    SAMPLE_TABLE_CHUNK_OFFSET_64 => {
                        reader.seek(SeekFrom::Start(a.content_pos()))?;
                        let chunk_offset = Co64::parse(&mut reader, a.size())?;

                        writer.seek(SeekFrom::Start(chunk_offset.table_pos))?;
                        for co in chunk_offset.offsets.iter() {
                            let new_offset = (*co as i64 + len_diff) as u64;
                            writer.write_all(&u64::to_be_bytes(new_offset))?;
                        }
                        writer.flush()?;
                    }
                    _ => (),
                }
            }
        }
    }

    // update existing ilst hierarchy atom lengths
    for a in update_atoms.iter().rev() {
        let new_len = a.len() as i64 + len_diff;
        writer.seek(SeekFrom::Start(a.pos()))?;
        if a.ext() {
            writer.write_all(&u32::to_be_bytes(1))?;
            writer.seek(SeekFrom::Current(4))?;
            writer.write_all(&u64::to_be_bytes(new_len as u64))?;
        } else {
            writer.write_all(&u32::to_be_bytes(new_len as u32))?;
        }
    }

    // adjusting the file length
    file.set_len((old_file_len as i64 + len_diff) as u64)?;

    // write missing ilst hierarchy and metadata
    writer.seek(SeekFrom::Start(new_atoms_start))?;

    if let Some(a) = new_udta {
        a.write(&mut writer)?;
    } else if let Some(a) = new_meta {
        a.write(&mut writer)?;
    } else {
        if let Some(a) = new_hdlr {
            a.write(&mut writer)?;
        }
        new_ilst.write(&mut writer)?;
    }

    // writing moved data
    writer.seek(SeekFrom::Start((moved_data_start as i64 + len_diff) as u64))?;
    writer.write_all(&moved_data)?;
    writer.flush()?;

    Ok(())
}

/// Attempts to dump the metadata atoms to the writer. This doesn't include a complete MPEG-4
/// container hierarchy and won't result in a usable file.
pub(crate) fn dump_tag_to(writer: &mut impl Write, atoms: &[AtomData]) -> crate::Result<()> {
    let ftyp = Ftyp("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned());
    #[rustfmt::skip]
    let moov = Moov {
        udta: Some(Udta {
            meta: Some(Meta {
                hdlr: Some(Meta::hdlr()),
                ilst: Some(Ilst::Borrowed(atoms)),
            }),
        }),
        ..Default::default()
    };

    ftyp.write(writer)?;
    moov.write(writer)?;

    Ok(())
}
