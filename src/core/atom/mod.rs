use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Deref;

use crate::{data, Content, ContentT, Data, ErrorKind, Tag};

pub use audio::*;
pub use ident::*;
pub use template::*;

mod audio;
mod ident;
mod template;

/// A list of valid file types in lowercase defined by the filetype (`ftyp`) atom.
#[rustfmt::skip]
pub const VALID_FILETYPES: [&str; 8] = [
    "iso2",
    "isom",
    "m4a ",
    "m4b ",
    "m4p ",
    "m4v ",
    "mp41",
    "mp42",
];

/// A struct representing data that is associated with an Atom identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomData {
    /// The identifier of the atom.
    pub ident: DataIdent,
    /// The data contained in the atom.
    pub data: Data,
}

impl TryFrom<Atom> for AtomData {
    type Error = crate::Error;

    fn try_from(value: Atom) -> Result<Self, Self::Error> {
        let mut data: Option<Data> = None;
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;

        for atom in value.content.into_iter() {
            match atom.ident {
                DATA => data = atom.content.take_data(),
                MEAN => mean = atom.content.take_data().and_then(Data::take_string),
                NAME => name = atom.content.take_data().and_then(Data::take_string),
                _ => continue,
            }
        }

        let ident = match (value.ident, mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (ident, _, _) => DataIdent::FourCC(ident),
        };

        match data {
            Some(data) => Ok(Self::new(ident, data)),
            None => Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(DATA),
                "Error constructing atom data, missing data atom".to_owned(),
            )),
        }
    }
}

impl TryFrom<&Atom> for AtomData {
    type Error = crate::Error;

    fn try_from(value: &Atom) -> Result<Self, Self::Error> {
        if let Some(data) = value.child(DATA).and_then(|a| a.content.data()) {
            let mean_data = value.child(MEAN).and_then(|a| a.content.data());
            let mean = mean_data.and_then(Data::string).map(str::to_owned);

            let name_atom = value.child(NAME).and_then(|a| a.content.data());
            let name = name_atom.and_then(Data::string).map(str::to_owned);

            let ident = match (value.ident, mean, name) {
                (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
                (ident, _, _) => DataIdent::FourCC(ident),
            };

            return Ok(Self::new(ident, data.clone()));
        }

        Err(crate::Error::new(
            crate::ErrorKind::AtomNotFound(DATA),
            "Error constructing atom data, missing data atom".to_owned(),
        ))
    }
}

impl AtomData {
    /// Creates atom data with the `identifier` and `data`.
    pub const fn new(ident: DataIdent, data: Data) -> Self {
        Self { ident, data }
    }

    /// Returns the external length of the atom in bytes.
    pub fn len(&self) -> usize {
        let parent_len = 8;
        let data_len = 16 + self.data.len();

        match &self.ident {
            DataIdent::FourCC(_) => parent_len + data_len,
            DataIdent::Freeform { mean, name } => {
                let mean_len = 12 + mean.len();
                let name_len = 12 + name.len();

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
        writer.write_all(&(self.len() as u32).to_be_bytes())?;

        match &self.ident {
            DataIdent::FourCC(ident) => writer.write_all(ident.deref())?,
            _ => {
                let (mean, name) = match &self.ident {
                    DataIdent::Freeform { mean, name } => (mean.as_str(), name.as_str()),
                    DataIdent::FourCC(_) => unreachable!(),
                };
                writer.write_all(FREEFORM.deref())?;

                let mean_len: u32 = 12 + mean.len() as u32;
                writer.write_all(&mean_len.to_be_bytes())?;
                writer.write_all(MEAN.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(&mean.as_bytes())?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_all(&name_len.to_be_bytes())?;
                writer.write_all(NAME.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(&name.as_bytes())?;
            }
        }

        let data_len: u32 = 16 + self.data.len() as u32;
        writer.write_all(&data_len.to_be_bytes())?;
        writer.write_all(DATA.deref())?;
        self.data.write_typed(writer)?;

        Ok(())
    }
}

/// A struct that represents a MPEG-4 audio metadata atom.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Atom {
    /// The 4 byte identifier of the atom.
    pub ident: FourCC,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl From<AtomData> for Atom {
    #[rustfmt::skip]
    fn from(value: AtomData) -> Self {
        match value.ident {
            DataIdent::Freeform { mean, name } => {
                Self::new(FREEFORM, 0, Content::Atoms(vec![
                    Self::mean_atom_with(mean),
                    Self::name_atom_with(name),
                    Self::data_atom_with(value.data),
                ]))
            }
            DataIdent::FourCC(ident) => Self::new(ident, 0, Content::data_atom_with(value.data)),
        }
    }
}

impl From<&AtomData> for Atom {
    #[rustfmt::skip]
    fn from(value: &AtomData) -> Self {
        match &value.ident {
            DataIdent::Freeform { mean, name } => {
                Self::new(FREEFORM, 0, Content::Atoms(vec![
                    Self::mean_atom_with(mean.clone()),
                    Self::name_atom_with(name.clone()),
                    Self::data_atom_with(value.data.clone()),
                ]))
            }
            DataIdent::FourCC(ident) => Self::new(*ident, 0, Content::data_atom_with(value.data.clone())),
        }
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atom {{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl Atom {
    /// Creates an atom containing the provided content at a n byte offset.
    pub const fn new(ident: FourCC, offset: usize, content: Content) -> Self {
        Self { ident, offset, content }
    }

    /// Creates a mean atom containing [`Content::RawData`](crate::Content::RawData)
    /// with the provided `mean` string.
    pub const fn mean_atom_with(mean: String) -> Self {
        Self::new(MEAN, 4, Content::RawData(Data::Utf8(mean)))
    }

    /// Creates a name atom containing [`Content::RawData`](crate::Content::RawData)
    /// with the provided `name` string.
    pub const fn name_atom_with(name: String) -> Self {
        Self::new(NAME, 4, Content::RawData(Data::Utf8(name)))
    }

    /// Creates a data atom containing [`Content::TypedData`](crate::Content::TypedData)
    /// with the provided `data`.
    pub const fn data_atom_with(data: Data) -> Self {
        Self::new(DATA, 0, Content::TypedData(data))
    }

    /// Returns the length of the atom in bytes.
    pub fn len(&self) -> usize {
        8 + self.offset + self.content.len()
    }

    /// Returns true if the atom has no `offset` or `content` and only consists of it's 8 byte head.
    pub fn is_empty(&self) -> bool {
        self.offset + self.content.len() == 0
    }

    /// Returns a reference to the first children atom matching the `identifier`, if present.
    pub fn child(&self, ident: FourCC) -> Option<&Self> {
        self.content.child(ident)
    }

    /// Returns a mutable reference to the first children atom matching the `identifier`, if
    /// present.
    pub fn child_mut(&mut self, ident: FourCC) -> Option<&mut Self> {
        self.content.child_mut(ident)
    }

    /// Returns a mutable reference to the first children atom, if present.
    pub fn mut_first_child(&mut self) -> Option<&mut Self> {
        self.content.first_child_mut()
    }

    /// Consumes self and returns the first children atom matching the `identifier`, if present.
    pub fn take_child(self, ident: FourCC) -> Option<Self> {
        self.content.take_child(ident)
    }

    /// Consumes self and returns the first children atom, if present.
    pub fn take_first_child(self) -> Option<Self> {
        self.content.take_first_child()
    }

    /// Attempts to write the atom to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&(self.len() as u32).to_be_bytes())?;
        writer.write_all(self.ident.deref())?;
        writer.write_all(&vec![0u8; self.offset])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Validates the filtype and returns it, or an error otherwise.
    pub fn check_filetype(self) -> crate::Result<String> {
        match self.content {
            Content::RawData(Data::Utf8(s)) => {
                if let Some(major_brand) = &s.get(0..4) {
                    if VALID_FILETYPES.iter().any(|f| f.eq_ignore_ascii_case(major_brand)) {
                        return Ok(s);
                    }
                }

                Err(crate::Error::new(
                    ErrorKind::InvalidFiletype(s),
                    "Invalid filetype.".to_owned(),
                ))
            }
            _ => Err(crate::Error::new(ErrorKind::NoTag, "No filetype atom found.".to_owned())),
        }
    }
}

/// A template representing a MPEG-4 audio metadata atom.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct AtomT {
    /// The 4 byte identifier of the atom.
    pub ident: FourCC,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content template of an atom template.
    pub content: ContentT,
}

impl fmt::Debug for AtomT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomT {{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl AtomT {
    /// Creates an atom template containing the provided content at a n byte offset.
    pub const fn new(ident: FourCC, offset: usize, content: ContentT) -> Self {
        Self { ident, offset, content }
    }

    /// Creates a data atom template containing [`ContentT::TypedData`](crate::ContentT::TypedData).
    pub const fn data_atom() -> Self {
        Self::new(DATA, 0, ContentT::TypedData)
    }

    /// Creates a mean atom template containing [`ContentT::RawData`](crate::ContentT::RawData).
    pub const fn mean_atom() -> Self {
        Self::new(MEAN, 4, ContentT::RawData(data::UTF8))
    }

    /// Creates a name atom template containing [`ContentT::TypedData`](crate::ContentT::TypedData).
    pub const fn name_atom() -> Self {
        Self::new(NAME, 4, ContentT::RawData(data::UTF8))
    }

    /// Returns a reference to the first children atom template matching the identifier, if present.
    pub fn child(&self, ident: FourCC) -> Option<&Self> {
        self.content.child(ident)
    }

    /// Returns a reference to the first children atom template, if present.
    pub fn first_child(&self) -> Option<&Self> {
        self.content.first_child()
    }

    /// Returns a mutable reference to the first children atom template matching the identifier, if
    /// present.
    pub fn child_mut(&mut self, ident: FourCC) -> Option<&mut Self> {
        self.content.child_mut(ident)
    }

    /// Returns a mutable reference to the first children atom template, if present.
    pub fn first_child_mut(&mut self) -> Option<&mut Self> {
        self.content.first_child_mut()
    }

    /// Consumes self and returns the first children atom template matching the `identifier`, if
    /// present.
    pub fn take_child(self, ident: FourCC) -> Option<Self> {
        self.content.take_child(ident)
    }

    /// Consumes self and returns the first children atom template, if present.
    pub fn take_first_child(self) -> Option<Self> {
        self.content.take_first_child()
    }

    /// Attempts to parse one atom, that matches the template, from the `reader`.  This should only
    /// be used if the atom has to be in this exact position, if the parsed and expected `ident`s
    /// don't match this will return an error.
    pub fn parse_next(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let (len, ident) = match parse_head(reader) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        if ident == self.ident || self.ident == WILDCARD {
            match parse_content(reader, &self.content, self.offset, len - 8) {
                Ok(c) => Ok(Atom::new(ident, self.offset, c)),
                Err(e) => Err(crate::Error::new(
                    e.kind,
                    format!("Error reading {}: {}", ident, e.description),
                )),
            }
        } else {
            Err(crate::Error::new(
                ErrorKind::AtomNotFound(self.ident),
                format!("Expected {} found {}", self.ident, ident),
            ))
        }
    }

    /// Attempts to parse one atom hierarchy, that matches this template, from the reader.
    pub fn parse(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let len = data::remaining_stream_len(reader)? as usize;
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let (atom_len, atom_ident) = parse_head(reader)?;

            if atom_ident == self.ident || self.ident == WILDCARD {
                return match parse_content(reader, &self.content, self.offset, atom_len - 8) {
                    Ok(c) => Ok(Atom::new(atom_ident, self.offset, c)),
                    Err(e) => Err(crate::Error::new(
                        e.kind,
                        format!("Error reading {}: {}", atom_ident, e.description),
                    )),
                };
            } else {
                reader.seek(SeekFrom::Current((atom_len - 8) as i64))?;
            }

            parsed_bytes += atom_len;
        }

        Err(crate::Error::new(
            ErrorKind::AtomNotFound(self.ident),
            format!("No {} atom found", self.ident),
        ))
    }
}

/// Attempts to parse any amount of atoms, matching the atom hierarchy templates, from the reader.
pub(crate) fn parse_atoms(
    reader: &mut (impl Read + Seek),
    atoms: &[AtomT],
    len: usize,
) -> crate::Result<Vec<Atom>> {
    let mut parsed_atoms = Vec::with_capacity(atoms.len());
    let mut pos = 0;

    while pos < len {
        let (atom_len, atom_ident) = parse_head(reader)?;
        let mut parsed = false;

        for a in atoms {
            if atom_ident == a.ident || a.ident == WILDCARD {
                match parse_content(reader, &a.content, a.offset, atom_len - 8) {
                    Ok(c) => {
                        parsed_atoms.push(Atom::new(atom_ident, a.offset, c));
                        parsed = true;
                    }
                    Err(e) => {
                        return Err(crate::Error::new(
                            e.kind,
                            format!("Error reading {}: {}", atom_ident, e.description),
                        ));
                    }
                }
                break;
            }
        }

        if !parsed {
            reader.seek(SeekFrom::Current((atom_len - 8) as i64))?;
        }

        pos += atom_len;
    }

    Ok(parsed_atoms)
}

/// Attempts to parse the atom template's content from the reader.
pub(crate) fn parse_content(
    reader: &mut (impl Read + Seek),
    content: &ContentT,
    offset: usize,
    length: usize,
) -> crate::Result<Content> {
    match length {
        0 => Ok(Content::Empty),
        _ => {
            if offset != 0 {
                reader.seek(SeekFrom::Current(offset as i64))?;
            }
            content.parse(reader, length - offset)
        }
    }
}

/// Attempts to parse the atom's head containing a 32 bit unsigned integer determining the size of
/// the atom in bytes and the following 4 byte identifier from the reader.
pub(crate) fn parse_head(reader: &mut impl Read) -> crate::Result<(usize, FourCC)> {
    let len = match data::read_u32(reader) {
        Ok(l) => l as usize,
        Err(e) => {
            return Err(crate::Error::new(e.kind, "Error reading atom length".to_owned()));
        }
    };
    let mut ident = FourCC([0u8; 4]);
    if let Err(e) = reader.read_exact(&mut *ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".to_owned(),
        ));
    }

    if len < 8 {
        return Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            format!("Read length of {} which is less than 8 bytes: {}", ident, len),
        ));
    }

    Ok((len, ident))
}

pub(crate) fn parse_ext_head(reader: &mut impl Read) -> crate::Result<(u8, [u8; 3])> {
    let version = data::read_u8(reader)?;
    let mut flags = [0u8; 3];
    reader.read_exact(&mut flags)?;

    Ok((version, flags))
}

pub(crate) fn parse_desc_head(reader: &mut impl Read) -> crate::Result<(u8, usize, usize)> {
    let tag = data::read_u8(reader)?;

    let mut head_len = 1;
    let mut len = 0;
    while head_len < 5 {
        let b = data::read_u8(reader)?;
        len = (len << 7) | (b & 0x7F) as u32;
        head_len += 1;
        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((tag, head_len, len as usize))
}

/// A struct representing of a sample table chunk offset atom (`stco`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ChunkOffset {
    pub pos: u64,
    pub version: u8,
    pub flags: [u8; 3],
    pub offsets: Vec<u32>,
}

/// Parses the content of a sample table chunk offset atom (`stco`).
fn parse_chunk_offset(reader: &mut (impl Read + Seek)) -> crate::Result<ChunkOffset> {
    let pos = reader.seek(SeekFrom::Current(0))?;

    let version = data::read_u8(reader)?;
    let mut flags = [0u8; 3];
    reader.read_exact(&mut flags)?;

    match version {
        0 => {
            let entries = data::read_u32(reader)?;
            let mut offsets = Vec::new();

            for _ in 0..entries {
                let offset = data::read_u32(reader)?;
                offsets.push(offset);
            }

            Ok(ChunkOffset { pos, version, flags, offsets })
        }
        _ => Err(crate::Error::new(
            crate::ErrorKind::UnknownVersion(version),
            "Unknown sample table chunk offset (stco) version".to_owned(),
        )),
    }
}

/// A struct storing the position and size of an atom.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AtomInfo {
    pub ident: FourCC,
    pub pos: u64,
    pub len: usize,
}

impl AtomInfo {
    fn new(ident: FourCC, pos: u64, len: usize) -> Self {
        Self { ident, pos, len }
    }
}

/// Finds out the position and size of any atoms matching the template hierarchy.
fn find_atoms(
    reader: &mut (impl Read + Seek),
    atoms: &[AtomT],
    len: usize,
) -> crate::Result<Vec<AtomInfo>> {
    let mut atom_info = Vec::new();
    let mut pos = 0;

    while pos < len {
        let (atom_len, atom_ident) = parse_head(reader)?;

        match atoms.iter().find(|a| a.ident == atom_ident) {
            Some(a) => {
                let atom_pos = reader.seek(SeekFrom::Current(0))? - 8;
                atom_info.push(AtomInfo::new(atom_ident, atom_pos, atom_len));

                if let ContentT::Atoms(c) = &a.content {
                    if a.offset != 0 {
                        reader.seek(SeekFrom::Current(a.offset as i64))?;
                    }
                    match find_atoms(reader, c, atom_len - 8 - a.offset) {
                        Ok(mut a) => atom_info.append(&mut a),
                        Err(e) => {
                            return Err(crate::Error::new(
                                e.kind,
                                format!("Error finding {}: {}", atom_ident, e.description),
                            ));
                        }
                    }
                } else {
                    reader.seek(SeekFrom::Current((atom_len - 8) as i64))?;
                }
            }
            None => {
                reader.seek(SeekFrom::Current((atom_len - 8) as i64))?;
            }
        }

        pos += atom_len;
    }

    Ok(atom_info)
}

/// Attempts to read MPEG-4 audio metadata from the reader.
pub(crate) fn read_tag_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
    let mut tag_atoms = None;
    let mut mvhd_data = None;
    let mut audio_info = None;

    let ftyp = FILETYPE_ATOM_T.parse_next(reader)?;
    let ftyp_string = ftyp.check_filetype()?;

    let moov = METADATA_READ_ATOM_T.parse(reader)?;
    for a in moov.content.into_iter() {
        match a.ident {
            MOVIE_HEADER => {
                if let Content::RawData(Data::Reserved(v)) = a.content {
                    mvhd_data = Some(v);
                }
            }
            TRACK => {
                if let Some(mdia) = a.take_child(MEDIA) {
                    if let Some(minf) = mdia.take_child(MEDIA_INFORMATION) {
                        if let Some(stbl) = minf.take_child(SAMPLE_TABLE) {
                            if let Some(stsd) = stbl.take_child(SAMPLE_TABLE_SAMPLE_DESCRIPTION) {
                                if let Some(mp4a) = stsd.take_child(MP4_AUDIO) {
                                    if let Content::Mp4Audio(a) = mp4a.content {
                                        audio_info = Some(a);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            USER_DATA => {
                if let Some(meta) = a.take_child(METADATA) {
                    if let Some(ilst) = meta.take_child(ITEM_LIST) {
                        if let Content::Atoms(atoms) = ilst.content {
                            tag_atoms = Some(
                                atoms
                                    .into_iter()
                                    .filter(|a| a.ident != FREE)
                                    .filter_map(|a| AtomData::try_from(a).ok())
                                    .collect(),
                            );
                        }
                    }
                }
            }
            _ => (),
        }
    }

    let tag_atoms = match tag_atoms {
        Some(t) => t,
        None => Vec::new(),
    };

    Ok(Tag::new(ftyp_string, mvhd_data, audio_info.unwrap_or_default(), tag_atoms))
}

/// Attempts to write the metadata atoms to the file inside the item list atom.
pub(crate) fn write_tag_to(file: &File, atoms: &[AtomData]) -> crate::Result<()> {
    let mut reader = BufReader::new(file);

    let ftyp = FILETYPE_ATOM_T.parse_next(&mut reader)?;
    ftyp.check_filetype()?;

    let len = data::remaining_stream_len(&mut reader)? as usize;
    let atom_info = find_atoms(&mut reader, METADATA_WRITE_ATOM_T.deref(), len)?;

    let mdat_info = atom_info.iter().find(|a| a.ident == MEDIA_DATA).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MEDIA_DATA),
            "Missing necessary data, no media data atom found".to_owned(),
        )
    })?;
    let moov_info = atom_info.iter().find(|a| a.ident == MOVIE).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(MOVIE),
            "Missing necessary data, no movie atom found".to_owned(),
        )
    })?;
    let udta_info = atom_info.iter().find(|a| a.ident == USER_DATA).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(USER_DATA),
            "Missing necessary data, no user data atom found".to_owned(),
        )
    })?;
    let meta_info = atom_info.iter().find(|a| a.ident == METADATA).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(METADATA),
            "Missing necessary data, no metadata atom found".to_owned(),
        )
    })?;
    let ilst_info = atom_info.iter().find(|a| a.ident == ITEM_LIST).ok_or_else(|| {
        crate::Error::new(
            crate::ErrorKind::AtomNotFound(ITEM_LIST),
            "Missing necessary data, no item list atom found".to_owned(),
        )
    })?;

    let mut writer = BufWriter::new(file);
    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let metadata_pos = ilst_info.pos + 8;
    let old_metadata_len = ilst_info.len - 8;
    let new_metadata_len = atoms.iter().map(AtomData::len).sum::<usize>();
    let metadata_len_diff = new_metadata_len as i64 - old_metadata_len as i64;

    match metadata_len_diff {
        0 => {
            // writing metadata
            writer.seek(SeekFrom::Start(metadata_pos as u64))?;
            for a in atoms {
                a.write_to(&mut writer)?;
            }
        }
        len_diff if len_diff <= -8 => {
            // writing metadata
            writer.seek(SeekFrom::Start(metadata_pos as u64))?;
            for a in atoms {
                a.write_to(&mut writer)?;
            }

            // Fill remaining space with a free atom
            let free = Atom::new(FREE, (len_diff.abs() - 8) as usize, Content::Empty);
            free.write_to(&mut writer)?;
        }
        _ => {
            // reading additional data after metadata
            let additional_data_len = old_file_len - (metadata_pos + old_metadata_len as u64);
            let mut additional_data = Vec::with_capacity(additional_data_len as usize);
            reader.seek(SeekFrom::Start(metadata_pos + old_metadata_len as u64))?;
            reader.read_to_end(&mut additional_data)?;

            // adjusting sample table chunk offsets
            if mdat_info.pos > moov_info.pos {
                reader.seek(SeekFrom::Start(0))?;

                // TODO: support inner `co64` atoms (64 bit chunks)
                // [Chunk offset atoms](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25715)
                let stco_info = atom_info.iter().filter(|a| a.ident == SAMPLE_TABLE_CHUNK_OFFSET);

                let mut stco_present = false;
                for a in stco_info {
                    reader.seek(SeekFrom::Start(a.pos as u64 + 8))?;
                    let chunk_offset = parse_chunk_offset(&mut reader)?;

                    writer.seek(SeekFrom::Start(chunk_offset.pos + 8))?;
                    for co in chunk_offset.offsets.iter() {
                        let new_offset = (*co as i64 + metadata_len_diff) as u32;
                        writer.write_all(&new_offset.to_be_bytes())?;
                    }
                    stco_present = true;
                }

                if !stco_present {
                    return Err(crate::Error::new(
                        crate::ErrorKind::AtomNotFound(SAMPLE_TABLE_CHUNK_OFFSET),
                        "No sample table chunk offset atom found".to_owned(),
                    ));
                }
            }

            // adjusting the file length
            file.set_len((old_file_len as i64 + metadata_len_diff as i64) as u64)?;

            // adjusting the atom lengths
            let mut write_pos = |a: &AtomInfo| -> crate::Result<()> {
                let new_len = (a.len as i64 + metadata_len_diff) as u32;
                writer.seek(SeekFrom::Start(a.pos as u64))?;
                writer.write_all(&new_len.to_be_bytes())?;
                Ok(())
            };
            write_pos(moov_info)?;
            write_pos(udta_info)?;
            write_pos(meta_info)?;
            write_pos(ilst_info)?;

            // writing metadata
            writer.seek(SeekFrom::Current(4))?;
            for a in atoms {
                a.write_to(&mut writer)?;
            }

            // writing additional data after metadata
            writer.write_all(&additional_data)?;
        }
    }
    writer.flush()?;

    Ok(())
}

/// Attempts to dump the metadata atoms to the writer. This doesn't include a complete MPEG-4
/// container hierarchy and won't result in a usable file.
pub(crate) fn dump_tag_to(writer: &mut impl Write, atoms: Vec<Atom>) -> crate::Result<()> {
    #[rustfmt::skip]
    let ftyp = Atom::new(FILETYPE, 0, Content::RawData(
        Data::Utf8("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned())),
    );
    #[rustfmt::skip]
    let moov = Atom::new(MOVIE, 0, Content::atom(
        Atom::new(USER_DATA, 0, Content::atom(
            Atom::new(METADATA, 4, Content::atom(
                Atom::new(ITEM_LIST, 0, Content::Atoms(atoms))
            )),
        )),
    ));

    ftyp.write_to(writer)?;
    moov.write_to(writer)?;

    Ok(())
}
