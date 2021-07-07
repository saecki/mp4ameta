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
use template::*;

use co64::*;
use ftyp::*;
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
pub use ilst::AtomData;

#[macro_use]
pub mod data;
/// A module for working with identifiers.
pub mod ident;

mod content;
mod template;

mod co64;
mod ftyp;
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

/// A struct that represents a MPEG-4 audio metadata atom.
#[derive(Clone, Default, Eq, PartialEq)]
struct Atom<'a> {
    /// The 4 byte identifier of the atom.
    ident: Fourcc,
    /// The offset in bytes separating the head from the content.
    offset: u64,
    /// The content of an atom.
    content: Content<'a>,
}

impl fmt::Debug for Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atom {{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl<'a> Atom<'a> {
    /// Creates an atom containing the provided content at a n byte offset.
    const fn new(ident: Fourcc, offset: u64, content: Content<'a>) -> Self {
        Self { ident, offset, content }
    }

    /// Returns the length of the atom in bytes.
    fn len(&self) -> u64 {
        8 + self.offset + self.content.len()
    }

    /// Attempts to write the atom to the writer.
    fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&u32::to_be_bytes(self.len() as u32))?;
        writer.write_all(self.ident.deref())?;
        writer.write_all(&vec![0u8; self.offset as usize])?;

        self.content.write_to(writer)?;

        Ok(())
    }
}

/// A template representing a MPEG-4 audio metadata atom.
#[derive(Clone, Default, Eq, PartialEq)]
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

/// A head specifying the size and type of an atom.
///
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional extended length
struct Head {
    /// Whether the head is of standard size (8 bytes) with a 32 bit length or extended (16 bytes)
    /// with a 64 bit length.
    short: bool,
    /// The length including this head.
    len: u64,
    /// The identifier.
    fourcc: Fourcc,
}

impl Head {
    const fn new(short: bool, len: u64, ident: Fourcc) -> Self {
        Self { short, len, fourcc: ident }
    }

    const fn head_len(&self) -> u64 {
        match self.short {
            true => 8,
            false => 16,
        }
    }

    const fn content_len(&self) -> u64 {
        match self.short {
            true => self.len - 8,
            false => self.len - 16,
        }
    }
}

/// Attempts to parse the atom's head containing a 32 bit unsigned integer determining the size of
/// the atom in bytes and the following 4 byte identifier from the reader. If the 32 len is set to
/// 1 an extended 64 bit length is read.
fn parse_head(reader: &mut impl Read) -> crate::Result<Head> {
    let len = match data::read_u32(reader) {
        Ok(l) => l as u64,
        Err(e) => {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length".to_owned(),
            ));
        }
    };
    let mut ident = Fourcc([0u8; 4]);
    if let Err(e) = reader.read_exact(&mut *ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".to_owned(),
        ));
    }

    if len == 1 {
        match data::read_u64(reader) {
            Ok(l) => Ok(Head::new(false, l, ident)),
            Err(e) => Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading extended atom length".to_owned(),
            )),
        }
    } else if len < 8 {
        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            format!("Read length of '{}' which is less than 8 bytes: {}", ident, len),
        ))
    } else {
        Ok(Head::new(true, len, ident))
    }
}

/// Attempts to parse a full atom head.
///
/// 1 byte version
/// 3 bytes flags
fn parse_full_head(reader: &mut impl Read) -> crate::Result<(u8, [u8; 3])> {
    let version = match data::read_u8(reader) {
        Ok(v) => v,
        Err(e) => {
            return Err(crate::Error::new(
                crate::ErrorKind::Io(e),
                "Error reading version of full atom head".to_owned(),
            ));
        }
    };

    let mut flags = [0u8; 3];
    if let Err(e) = reader.read_exact(&mut flags) {
        return Err(crate::Error::new(
            crate::ErrorKind::Io(e),
            "Error reading flags of full atom head".to_owned(),
        ));
    };

    Ok((version, flags))
}

/// A struct storing the position and size of an atom.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AtomBounds {
    pos: u64,
    short: bool,
    len: u64,
    ident: Fourcc,
}

impl AtomBounds {
    fn new(pos: u64, short: bool, len: u64, ident: Fourcc) -> Self {
        Self { pos, short, len, ident }
    }

    const fn head_len(&self) -> u64 {
        match self.short {
            true => 8,
            false => 16,
        }
    }

    const fn content_len(&self) -> u64 {
        match self.short {
            true => self.len - 8,
            false => self.len - 16,
        }
    }

    const fn content_pos(&self) -> u64 {
        self.pos + self.head_len()
    }

    const fn end(&self) -> u64 {
        self.pos + self.len
    }
}

/// A struct a hierarchy of atom bounds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

        match atoms.iter().find(|a| a.ident == head.fourcc) {
            Some(a) => {
                let atom_pos = reader.seek(SeekFrom::Current(0))? - head.head_len();
                let bounds = AtomBounds::new(atom_pos, head.short, head.len, head.fourcc);

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
                                    format!("Error finding {}: {}", head.fourcc, e.description),
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

        pos += head.len;
    }

    Ok(found_atoms)
}

trait ParseAtom: Sized {
    const FOURCC: Fourcc;

    fn parse(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        match Self::parse_atom(reader, len) {
            Err(mut e) => {
                e.description = format!("Error parsing {}: {}", Self::FOURCC, e.description);
                Err(e)
            }
            a => a,
        }
    }

    fn parse_atom(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self>;
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

        match head.fourcc {
            MOVIE => {
                break Moov::parse(reader, head.content_len())?;
            }
            _ => {
                reader.seek(SeekFrom::Current(head.content_len() as i64))?;
            }
        }

        parsed_bytes += head.len;
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
        .map_or(Vec::new(), |ilst| ilst.0);

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

    let mdat_pos = found_atoms.iter().find(|a| a.ident == MEDIA_DATA).map(|a| a.pos).unwrap_or(0);
    let moov = found_atoms.iter().find(|a| a.ident == MOVIE).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie (moov) atom found".to_owned(),
        )
    })?;
    let udta = moov.atoms.iter().find(|a| a.ident == USER_DATA);
    let meta = udta.and_then(|a| a.atoms.iter().find(|a| a.ident == METADATA));
    let hdlr = meta.and_then(|a| a.atoms.iter().find(|a| a.ident == HANDLER_REFERENCE));
    let ilst = meta.and_then(|a| a.atoms.iter().find(|a| a.ident == ITEM_LIST));

    let mut update_atoms = Vec::new();
    let mut new_atoms = Vec::new();
    let mut new_atoms_start = 0;
    let mut moved_data_start = 0;
    let mut len_diff = 0;

    if hdlr.is_none() {
        new_atoms.push(template::meta_handler_reference_atom());
    }
    if let Some(ilst) = ilst {
        new_atoms_start = ilst.pos;
        moved_data_start = ilst.end();
        len_diff -= ilst.len as i64;
    }
    new_atoms.push(Atom::new(ITEM_LIST, 0, Content::AtomDataRef(atoms)));

    match meta {
        Some(meta) => {
            update_atoms.push(meta);
            if ilst.is_none() {
                new_atoms_start = meta.end();
                moved_data_start = meta.end();
            }
        }
        None => {
            new_atoms = vec![Atom::new(METADATA, 4, Content::Atoms(new_atoms))];
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
            new_atoms = vec![Atom::new(USER_DATA, 0, Content::Atoms(new_atoms))];
            new_atoms_start = moov.end();
            moved_data_start = moov.end();
        }
    }
    len_diff += new_atoms.iter().map(|a| a.len()).sum::<u64>() as i64;
    update_atoms.push(moov);

    // reading moved data
    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let moved_data_len = old_file_len - moved_data_start;
    let mut moved_data = Vec::with_capacity(moved_data_len as usize);
    reader.seek(SeekFrom::Start(moved_data_start))?;
    reader.read_to_end(&mut moved_data)?;

    let mut writer = BufWriter::new(file);

    // adjusting sample table chunk offsets
    if mdat_pos > moov.pos {
        let stbl_atoms = moov
            .atoms
            .iter()
            .filter(|a| a.ident == TRACK)
            .filter_map(|a| a.atoms.iter().find(|a| a.ident == MEDIA))
            .filter_map(|a| a.atoms.iter().find(|a| a.ident == MEDIA_INFORMATION))
            .filter_map(|a| a.atoms.iter().find(|a| a.ident == SAMPLE_TABLE));

        for stbl in stbl_atoms {
            for a in stbl.atoms.iter() {
                match a.ident {
                    SAMPLE_TABLE_CHUNK_OFFSET => {
                        reader.seek(SeekFrom::Start(a.content_pos()))?;
                        let chunk_offset = Stco::parse(&mut reader, a.content_len())?;

                        writer.seek(SeekFrom::Start(chunk_offset.table_pos))?;
                        for co in chunk_offset.offsets.iter() {
                            let new_offset = (*co as i64 + len_diff) as u32;
                            writer.write_all(&u32::to_be_bytes(new_offset))?;
                        }
                        writer.flush()?;
                    }
                    SAMPLE_TABLE_CHUNK_OFFSET_64 => {
                        reader.seek(SeekFrom::Start(a.content_pos()))?;
                        let chunk_offset = Co64::parse(&mut reader, a.content_len())?;

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
        let new_len = a.len as i64 + len_diff;
        writer.seek(SeekFrom::Start(a.pos))?;
        if a.short {
            writer.write_all(&u32::to_be_bytes(new_len as u32))?;
        } else {
            writer.write_all(&u32::to_be_bytes(1))?;
            writer.seek(SeekFrom::Current(4))?;
            writer.write_all(&u64::to_be_bytes(new_len as u64))?;
        }
    }

    // adjusting the file length
    file.set_len((old_file_len as i64 + len_diff) as u64)?;

    // write missing ilst hierarchy and metadata
    if !new_atoms.is_empty() {
        writer.seek(SeekFrom::Start(new_atoms_start))?;
        for a in new_atoms.iter() {
            a.write_to(&mut writer)?;
        }
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
    #[rustfmt::skip]
    let ftyp = Atom::new(FILETYPE, 0, Content::RawData(
        Data::Utf8("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned())),
    );
    #[rustfmt::skip]
    let moov = Atom::new(MOVIE, 0, Content::atom(
        Atom::new(USER_DATA, 0, Content::atom(
            Atom::new(METADATA, 4, Content::Atoms(vec![
                template::meta_handler_reference_atom(),
                Atom::new(ITEM_LIST, 0, Content::AtomDataRef(atoms)),
            ])),
        )),
    ));

    ftyp.write_to(writer)?;
    moov.write_to(writer)?;

    Ok(())
}
