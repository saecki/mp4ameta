use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

use crate::{AudioInfo, ErrorKind, Tag};

pub use data::*;
pub use ident::*;

use content::*;
use info::*;
use template::*;

/// A module for working with identifiers.
pub mod ident;

pub mod data;

mod content;
mod info;
mod template;

/// A struct representing data that is associated with an atom identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomData {
    /// The identifier of the atom.
    pub ident: DataIdent,
    /// The data contained in the atom.
    pub data: Vec<Data>,
}

impl TryFrom<Atom<'_>> for AtomData {
    type Error = crate::Error;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        let mut data = Vec::new();
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;

        for atom in value.content.into_atoms() {
            match atom.ident {
                DATA => {
                    if let Some(d) = atom.content.take_data() {
                        data.push(d);
                    }
                }
                MEAN => mean = atom.content.take_data().and_then(Data::into_string),
                NAME => name = atom.content.take_data().and_then(Data::into_string),
                _ => continue,
            }
        }

        let ident = match (value.ident, mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (ident, _, _) => DataIdent::Fourcc(ident),
        };

        if data.is_empty() {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(DATA),
                "Error constructing atom data, missing data atom".to_owned(),
            ));
        }

        Ok(AtomData::new(ident, data))
    }
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
        self.data.is_empty()
    }

    /// Attempts to write the atom data to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
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
                writer.write_all(&mean.as_bytes())?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_all(&u32::to_be_bytes(name_len))?;
                writer.write_all(NAME.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(&name.as_bytes())?;
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

    /// Consumes self and returns the first children atom matching the identifier, if present.
    fn take_child(self, ident: Fourcc) -> Option<Self> {
        self.content.take_child(ident)
    }

    /// Attempts to write the atom to the writer.
    fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&u32::to_be_bytes(self.len() as u32))?;
        writer.write_all(self.ident.deref())?;
        writer.write_all(&vec![0u8; self.offset as usize])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Validates the filtype and returns it, or an error otherwise.
    fn check_filetype(self) -> crate::Result<String> {
        match self.content {
            Content::RawData(Data::Utf8(s)) => Ok(s),
            _ => Err(crate::Error::new(ErrorKind::NoTag, "No filetype atom found.".to_owned())),
        }
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

    /// Creates a data atom template containing [`ContentT::TypedData`].
    const fn data_atom() -> Self {
        Self::new(DATA, 0, ContentT::TypedData)
    }

    /// Creates a mean atom template containing [`ContentT::RawData`].
    const fn mean_atom() -> Self {
        Self::new(MEAN, 4, ContentT::RawData(data::UTF8))
    }

    /// Creates a name atom template containing [`ContentT::TypedData`].
    const fn name_atom() -> Self {
        Self::new(NAME, 4, ContentT::RawData(data::UTF8))
    }

    /// Attempts to parse one atom, that matches the template, from the `reader`.  This should only
    /// be used if the atom has to be in this exact position, if the parsed and expected `ident`s
    /// don't match this will return an error.
    fn parse_next(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let head = match parse_head(reader) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        if head.ident == self.ident || self.ident == WILDCARD {
            match parse_content(reader, &self.content, self.offset, head.content_len()) {
                Ok(c) => Ok(Atom::new(head.ident, self.offset, c)),
                Err(e) => Err(crate::Error::new(
                    e.kind,
                    format!("Error reading {}: {}", head.ident, e.description),
                )),
            }
        } else {
            Err(crate::Error::new(
                ErrorKind::AtomNotFound(self.ident),
                format!("Expected {} found {}", self.ident, head.ident),
            ))
        }
    }

    /// Attempts to parse one atom hierarchy, that matches this template, from the reader.
    fn parse(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let len = data::remaining_stream_len(reader)?;
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            if head.ident == self.ident || self.ident == WILDCARD {
                return match parse_content(reader, &self.content, self.offset, head.content_len()) {
                    Ok(c) => Ok(Atom::new(head.ident, self.offset, c)),
                    Err(e) => Err(crate::Error::new(
                        e.kind,
                        format!("Error reading {}: {}", head.ident, e.description),
                    )),
                };
            } else {
                reader.seek(SeekFrom::Current(head.content_len() as i64))?;
            }

            parsed_bytes += head.len;
        }

        Err(crate::Error::new(
            ErrorKind::AtomNotFound(self.ident),
            format!("No {} atom found", self.ident),
        ))
    }
}

/// Attempts to parse any amount of atoms, matching the atom hierarchy templates, from the reader.
fn parse_atoms<'a>(
    reader: &mut (impl Read + Seek),
    atoms: &[AtomT],
    len: u64,
) -> crate::Result<Vec<Atom<'a>>> {
    let mut parsed_atoms = Vec::with_capacity(atoms.len());
    let mut pos = 0;

    while pos < len {
        let head = parse_head(reader)?;
        let mut parsed = false;

        for a in atoms {
            if head.ident == a.ident || a.ident == WILDCARD {
                match parse_content(reader, &a.content, a.offset, head.content_len()) {
                    Ok(c) => {
                        parsed_atoms.push(Atom::new(head.ident, a.offset, c));
                        parsed = true;
                    }
                    Err(e) => {
                        return Err(crate::Error::new(
                            e.kind,
                            format!("Error reading {}: {}", head.ident, e.description),
                        ));
                    }
                }
                break;
            }
        }

        if !parsed {
            reader.seek(SeekFrom::Current(head.content_len() as i64))?;
        }

        pos += head.len
    }

    Ok(parsed_atoms)
}

/// Attempts to parse the atom template's content from the reader.
fn parse_content<'a>(
    reader: &mut (impl Read + Seek),
    content: &ContentT,
    offset: u64,
    len: u64,
) -> crate::Result<Content<'a>> {
    match len {
        0 => Ok(Content::Empty),
        _ => {
            if offset != 0 {
                reader.seek(SeekFrom::Current(offset as i64))?;
            }
            content.parse(reader, len - offset)
        }
    }
}

/// A head specifying the size and type of an atom.
///
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional length
struct Head {
    /// Whether the head is of standard size (8 bytes) with a 32 bit length or extended (16 bytes)
    /// with a 64 bit length.
    short: bool,
    /// The length including this head.
    len: u64,
    /// The identifier.
    ident: Fourcc,
}

impl Head {
    const fn new(short: bool, len: u64, ident: Fourcc) -> Self {
        Self { short, len, ident }
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
            format!("Read length of {} which is less than 8 bytes: {}", ident, len),
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

        match atoms.iter().find(|a| a.ident == head.ident) {
            Some(a) => {
                let atom_pos = reader.seek(SeekFrom::Current(0))? - head.head_len();
                let bounds = AtomBounds::new(atom_pos, head.short, head.len, head.ident);

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
                                    format!("Error finding {}: {}", head.ident, e.description),
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

/// Attempts to read MPEG-4 audio metadata from the reader.
pub(crate) fn read_tag_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
    let mut tag_atoms: Vec<AtomData> = Vec::new();
    let mut mvhd_info = None;
    let mut mp4a_info = None;

    let ftyp = FILETYPE_ATOM_T.parse_next(reader)?;
    let ftyp_string = ftyp.check_filetype()?;

    let moov = METADATA_READ_ATOM_T.parse(reader)?;
    for a in moov.content.into_atoms() {
        match a.ident {
            MOVIE_HEADER => {
                if let Content::MovieHeader(i) = a.content {
                    mvhd_info = Some(i);
                }
            }
            TRACK => {
                let mdia = a.take_child(MEDIA);
                let minf = mdia.and_then(|a| a.take_child(MEDIA_INFORMATION));
                let stbl = minf.and_then(|a| a.take_child(SAMPLE_TABLE));
                let stsd = stbl.and_then(|a| a.take_child(SAMPLE_TABLE_SAMPLE_DESCRIPTION));
                let mp4a = stsd.and_then(|a| a.take_child(MP4_AUDIO));

                if let Some(mp4a) = mp4a {
                    if let Content::Mp4Audio(i) = mp4a.content {
                        mp4a_info = Some(i);
                    }
                }
            }
            USER_DATA => {
                let meta = a.take_child(METADATA);
                let ilst = meta.and_then(|a| a.take_child(ITEM_LIST));

                if let Some(ilst) = ilst {
                    ilst.content
                        .into_atoms()
                        .filter(|a| a.ident != FREE)
                        .filter_map(|a| AtomData::try_from(a).ok())
                        .for_each(|a| {
                            let other = tag_atoms.iter_mut().find(|o| a.ident == o.ident);

                            match other {
                                Some(other) => other.data.extend(a.data),
                                None => tag_atoms.push(a),
                            }
                        });
                }
            }
            _ => (),
        }
    }

    let mut info = AudioInfo::default();
    if let Some(i) = mvhd_info {
        info.duration = i.duration;
    }
    if let Some(i) = mp4a_info {
        info.channel_config = i.channel_config;
        info.sample_rate = i.sample_rate;
        info.max_bitrate = i.max_bitrate;
        info.avg_bitrate = i.avg_bitrate;
    }

    Ok(Tag::new(ftyp_string, info, tag_atoms))
}

/// Attempts to write the metadata atoms to the file inside the item list atom.
pub(crate) fn write_tag_to(file: &File, atoms: &[AtomData]) -> crate::Result<()> {
    let mut reader = BufReader::new(file);

    let ftyp = FILETYPE_ATOM_T.parse_next(&mut reader)?;
    ftyp.check_filetype()?;

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
                        let chunk_offset = ChunkOffsetInfo::parse(&mut reader, a.content_len())?;

                        writer.seek(SeekFrom::Start(chunk_offset.table_pos))?;
                        for co in chunk_offset.offsets.iter() {
                            let new_offset = (*co as i64 + len_diff) as u32;
                            writer.write_all(&u32::to_be_bytes(new_offset))?;
                        }
                        writer.flush()?;
                    }
                    SAMPLE_TABLE_CHUNK_OFFSET_64 => {
                        reader.seek(SeekFrom::Start(a.content_pos()))?;
                        let chunk_offset = ChunkOffsetInfo64::parse(&mut reader, a.content_len())?;

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
        if a.short {
            writer.seek(SeekFrom::Start(a.pos))?;
            writer.write_all(&u32::to_be_bytes(new_len as u32))?;
        } else {
            writer.seek(SeekFrom::Start(a.pos))?;
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
