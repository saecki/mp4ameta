use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Deref;

use crate::{data, Content, ContentT, Data, DataT, ErrorKind, Tag};

use crate::core::data::remaining_stream_len;
pub use template::*;

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

/// (`ftyp`) Identifier of an atom information about the filetype.
pub const FILETYPE: Ident = Ident(*b"ftyp");
/// (`mdat`)
pub const MEDIA_DATA: Ident = Ident(*b"mdat");
/// (`moov`) Identifier of an atom containing a structure of children storing metadata.
pub const MOVIE: Ident = Ident(*b"moov");
/// (`mvhd`) Identifier of an atom containing information about the whole movie (or audio file).
pub const MOVIE_HEADER: Ident = Ident(*b"mvhd");
/// (`trak`) Identifier of an atom containing information about a single track.
pub const TRACK: Ident = Ident(*b"trak");
/// (`mdia`) Identifier of an atom containing information about a tracks media type and data.
pub const MEDIA: Ident = Ident(*b"mdia");
/// (`mdhd`) Identifier of an atom containing information about a track
pub const MEDIA_HEADER: Ident = Ident(*b"mdhd");
/// (`minf`)
pub const METADATA_INFORMATION: Ident = Ident(*b"minf");
/// (`stbl`)
pub const SAMPLE_TABLE: Ident = Ident(*b"stbl");
/// (`stco`)
pub const SAMPLE_TABLE_CHUNK_OFFSET: Ident = Ident(*b"stco");
/// (`udta`) Identifier of an atom containing user metadata.
pub const USER_DATA: Ident = Ident(*b"udta");
/// (`meta`) Identifier of an atom containing a metadata item list.
pub const METADATA: Ident = Ident(*b"meta");
/// (`ilst`) Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: Ident = Ident(*b"ilst");
/// (`data`) Identifier of an atom containing typed data.
pub const DATA: Ident = Ident(*b"data");
/// (`mean`)
pub const MEAN: Ident = Ident(*b"mean");
/// (`name`)
pub const NAME: Ident = Ident(*b"name");
/// (`free`)
pub const FREE: Ident = Ident(*b"free");

/// (`----`)
pub const WILDCARD: Ident = Ident(*b"----");

// iTunes 4.0 atoms
/// (`rtng`)
pub const ADVISORY_RATING: Ident = Ident(*b"rtng");
/// (`©alb`)
pub const ALBUM: Ident = Ident(*b"\xa9alb");
/// (`aART`)
pub const ALBUM_ARTIST: Ident = Ident(*b"aART");
/// (`©ART`)
pub const ARTIST: Ident = Ident(*b"\xa9ART");
/// (`covr`)
pub const ARTWORK: Ident = Ident(*b"covr");
/// (`tmpo`)
pub const BPM: Ident = Ident(*b"tmpo");
/// (`©cmt`)
pub const COMMENT: Ident = Ident(*b"\xa9cmt");
/// (`cpil`)
pub const COMPILATION: Ident = Ident(*b"cpil");
/// (`©wrt`)
pub const COMPOSER: Ident = Ident(*b"\xa9wrt");
/// (`cprt`)
pub const COPYRIGHT: Ident = Ident(*b"cprt");
/// (`©gen`)
pub const CUSTOM_GENRE: Ident = Ident(*b"\xa9gen");
/// (`disk`)
pub const DISC_NUMBER: Ident = Ident(*b"disk");
/// (`©too`)
pub const ENCODER: Ident = Ident(*b"\xa9too");
/// (`gnre`)
pub const STANDARD_GENRE: Ident = Ident(*b"gnre");
/// (`©nam`)
pub const TITLE: Ident = Ident(*b"\xa9nam");
/// (`trkn`)
pub const TRACK_NUMBER: Ident = Ident(*b"trkn");
/// (`©day`)
pub const YEAR: Ident = Ident(*b"\xa9day");

// iTunes 4.2 atoms
/// (`©grp`)
pub const GROUPING: Ident = Ident(*b"\xa9grp");
/// (`stik`)
pub const MEDIA_TYPE: Ident = Ident(*b"stik");

// iTunes 4.9 atoms
/// (`catg`)
pub const CATEGORY: Ident = Ident(*b"catg");
/// (`keyw`)
pub const KEYWORD: Ident = Ident(*b"keyw");
/// (`pcst`)
pub const PODCAST: Ident = Ident(*b"pcst");
/// (`egid`)
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: Ident = Ident(*b"egid");
/// (`purl`)
pub const PODCAST_URL: Ident = Ident(*b"purl");

// iTunes 5.0
/// (`desc`)
pub const DESCRIPTION: Ident = Ident(*b"desc");
/// (`©lyr`)
pub const LYRICS: Ident = Ident(*b"\xa9lyr");

// iTunes 6.0
/// (`tves`)
pub const TV_EPISODE: Ident = Ident(*b"tves");
/// (`tven`)
pub const TV_EPISODE_NUMBER: Ident = Ident(*b"tven");
/// (`tvnn`)
pub const TV_NETWORK_NAME: Ident = Ident(*b"tvnn");
/// (`tvsn`)
pub const TV_SEASON: Ident = Ident(*b"tvsn");
/// (`tvsh`)
pub const TV_SHOW_NAME: Ident = Ident(*b"tvsh");

// iTunes 6.0.2
/// (`purd`)
pub const PURCHASE_DATE: Ident = Ident(*b"purd");

// iTunes 7.0
/// (`pgap`)
pub const GAPLESS_PLAYBACK: Ident = Ident(*b"pgap");

// Work, Movement
/// (`©mvn`)
pub const MOVEMENT: Ident = Ident(*b"\xa9mvn");
/// (`©mvc`)
pub const MOVEMENT_COUNT: Ident = Ident(*b"\xa9mvc");
/// (`©mvi`)
pub const MOVEMENT_INDEX: Ident = Ident(*b"\xa9mvi");
/// (`©wrk`)
pub const WORK: Ident = Ident(*b"\xa9wrk");
/// (`shwm`)
pub const SHOW_MOVEMENT: Ident = Ident(*b"shwm");

/// A 4 byte atom identifier.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Ident(pub [u8; 4]);

impl Deref for Ident {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident({})", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

/// A struct representing data that is associated with an Atom identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomData {
    /// The 4 byte identifier of the atom.
    pub ident: Ident,
    /// The data corresponding to the identifier.
    pub data: Data,
}

impl AtomData {
    /// Creates atom data with the `identifier` and `data`.
    pub const fn new(ident: Ident, data: Data) -> Self {
        Self { ident, data }
    }

    /// Creates atom data with the `identifier` and raw `data` contained by the
    /// atom.
    pub fn try_from_raw(atom: Atom) -> Option<Self> {
        match atom.content {
            Content::RawData(d) => Some(Self::new(atom.ident, d)),
            _ => None,
        }
    }

    /// Creates atom data with the `identifier` and typed `data` contained by a children data atom.
    pub fn try_from_typed(atom: Atom) -> Option<Self> {
        if let Some(d) = atom.content.take_child(DATA) {
            if let Content::TypedData(data) = d.content {
                return Some(Self::new(atom.ident, data));
            }
        }
        None
    }

    /// Creates an atom with the `ident`, `offset` 0, containing a data atom with the `data`.
    pub fn into_typed(self) -> Atom {
        Atom::new(self.ident, 0, Content::data_atom_with(self.data))
    }

    /// Creates an atom with the `ident`, `offset` 0, containing a data atom with the `data`.
    pub fn to_typed(&self) -> Atom {
        Atom::new(self.ident, 0, Content::data_atom_with(self.data.clone()))
    }

    /// Creates an atom with the `ident`, `offset` 0, containing the raw `data`.
    pub fn into_raw(self) -> Atom {
        Atom::new(self.ident, 0, Content::RawData(self.data))
    }

    /// Creates an atom with the `ident`, `offset` 0, containing the raw `data`.
    pub fn to_raw(&self) -> Atom {
        Atom::new(self.ident, 0, Content::RawData(self.data.clone()))
    }
}

/// A struct that represents a MPEG-4 audio metadata atom.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Atom {
    /// The 4 byte identifier of the atom.
    pub ident: Ident,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atom{{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl Atom {
    /// Creates an atom containing the provided content at a n byte offset.
    pub const fn new(ident: Ident, offset: usize, content: Content) -> Self {
        Self { ident, offset, content }
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
    pub fn child(&self, ident: Ident) -> Option<&Self> {
        self.content.child(ident)
    }

    /// Returns a mutable reference to the first children atom matching the `identifier`, if
    /// present.
    pub fn child_mut(&mut self, ident: Ident) -> Option<&mut Self> {
        self.content.child_mut(ident)
    }

    /// Returns a mutable reference to the first children atom, if present.
    pub fn mut_first_child(&mut self) -> Option<&mut Self> {
        self.content.first_child_mut()
    }

    /// Consumes self and returns the first children atom matching the `identifier`, if present.
    pub fn take_child(self, ident: Ident) -> Option<Self> {
        self.content.take_child(ident)
    }

    /// Consumes self and returns the first children atom, if present.
    pub fn take_first_child(self) -> Option<Self> {
        self.content.take_first_child()
    }

    /// Attempts to write the atom to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&(self.len() as u32).to_be_bytes())?;
        writer.write_all(&*self.ident)?;
        writer.write_all(&vec![0u8; self.offset])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Checks if the filetype is valid, returns an error otherwise.
    pub fn check_filetype(&self) -> crate::Result<()> {
        match &self.content {
            Content::RawData(Data::Utf8(s)) => {
                let major_brand = s.split('\u{0}').next().unwrap();
                if VALID_FILETYPES.iter().any(|e| e.eq_ignore_ascii_case(major_brand)) {
                    return Ok(());
                }

                Err(crate::Error::new(
                    ErrorKind::InvalidFiletype(s.to_string()),
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
    pub ident: Ident,
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content template of an atom template.
    pub content: ContentT,
}

impl fmt::Debug for AtomT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomT{{ {}, {}, {:#?} }}", self.ident, self.offset, self.content)
    }
}

impl AtomT {
    /// Creates an atom template containing the provided content at a n byte offset.
    pub const fn new(ident: Ident, offset: usize, content: ContentT) -> Self {
        Self { ident, offset, content }
    }

    /// Creates a data atom template containing [`ContentT::TypedData`](crate::ContentT::TypedData).
    pub const fn data_atom() -> Self {
        Self::new(DATA, 0, ContentT::TypedData)
    }

    /// Creates a mean atom template containing [`ContentT::RawData`](crate::ContentT::RawData).
    pub const fn mean_atom() -> Self {
        Self::new(MEAN, 0, ContentT::RawData(DataT::new(data::UTF8)))
    }

    /// Creates a name atom template containing [`ContentT::TypedData`](crate::ContentT::TypedData).
    pub const fn name_atom() -> Self {
        Self::new(NAME, 0, ContentT::RawData(DataT::new(data::UTF8)))
    }

    /// Returns a reference to the first children atom template matching the identifier, if present.
    pub fn child(&self, ident: Ident) -> Option<&Self> {
        self.content.child(ident)
    }

    /// Returns a reference to the first children atom template, if present.
    pub fn first_child(&self) -> Option<&Self> {
        self.content.first_child()
    }

    /// Returns a mutable reference to the first children atom template matching the identifier, if
    /// present.
    pub fn child_mut(&mut self, ident: Ident) -> Option<&mut Self> {
        self.content.child_mut(ident)
    }

    /// Returns a mutable reference to the first children atom template, if present.
    pub fn first_child_mut(&mut self) -> Option<&mut Self> {
        self.content.first_child_mut()
    }

    /// Consumes self and returns the first children atom template matching the `identifier`, if
    /// present.
    pub fn take_child(self, ident: Ident) -> Option<Self> {
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

        if ident == self.ident {
            match parse_content(reader, &self.content, self.offset, len - 8) {
                Ok(c) => Ok(Atom::new(self.ident, self.offset, c)),
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

            if atom_ident == self.ident {
                return match parse_content(reader, &self.content, self.offset, atom_len - 8) {
                    Ok(c) => Ok(Atom::new(self.ident, self.offset, c)),
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

/// Attempts to read MPEG-4 audio metadata from the reader.
pub fn read_tag_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
    let mut tag_atoms = None;
    let mut mvhd_data = None;

    let ftyp = FILETYPE_ATOM_T.parse_next(reader)?;
    ftyp.check_filetype()?;
    let ftyp_data = match ftyp.content {
        Content::RawData(Data::Utf8(s)) => Some(s),
        _ => None,
    };

    let moov = METADATA_READ_ATOM_T.parse(reader)?;
    for a in moov.content.into_iter() {
        match a.ident {
            MOVIE_HEADER => {
                if let Content::RawData(Data::Reserved(v)) = a.content {
                    mvhd_data = Some(v);
                }
            }
            USER_DATA => {
                if let Some(meta) = a.take_child(METADATA) {
                    if let Some(ilst) = meta.take_child(ITEM_LIST) {
                        if let Content::Atoms(atoms) = ilst.content {
                            tag_atoms = Some(
                                atoms
                                    .into_iter()
                                    .filter(|a| {
                                        if let Some(d) = a.child(DATA) {
                                            if let Content::TypedData(_) = d.content {
                                                return true;
                                            }
                                        }
                                        false
                                    })
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

    Ok(Tag::new(ftyp_data, mvhd_data, tag_atoms))
}

/// Attempts to write the metadata atoms to the file inside the item list atom.
pub fn write_tag_to(file: &File, atoms: &[Atom]) -> crate::Result<()> {
    let mut reader = BufReader::new(file);

    let ftyp = FILETYPE_ATOM_T.parse_next(&mut reader)?;
    ftyp.check_filetype()?;

    let len = remaining_stream_len(&mut reader)? as usize;
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
    let new_metadata_len = atoms.iter().map(|a| a.len()).sum::<usize>();
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
pub fn dump_tag_to(writer: &mut impl Write, atoms: Vec<Atom>) -> crate::Result<()> {
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

/// Attempts to parse any amount of atoms, matching the atom hierarchy templates, from the reader.
pub fn parse_atoms(
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
            if atom_ident == a.ident {
                match parse_content(reader, &a.content, a.offset, atom_len - 8) {
                    Ok(c) => {
                        parsed_atoms.push(Atom::new(a.ident, a.offset, c));
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
pub fn parse_content(
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
pub fn parse_head(reader: &mut impl Read) -> crate::Result<(usize, Ident)> {
    let len = match data::read_u32(reader) {
        Ok(l) => l as usize,
        Err(e) => {
            return Err(crate::Error::new(e.kind, "Error reading atom length".to_owned()));
        }
    };
    let mut ident = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".to_owned(),
        ));
    }

    debug_assert!(len >= 8, "Atom length is less than 8 bytes");

    Ok((len, Ident(ident)))
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

    let mut version = [0u8; 1];
    let mut flags = [0u8; 3];
    reader.read_exact(&mut version)?;
    reader.read_exact(&mut flags)?;

    let entries = data::read_u32(reader)?;
    let mut offsets = Vec::new();

    for _ in 0..entries {
        let offset = data::read_u32(reader)?;
        offsets.push(offset);
    }

    Ok(ChunkOffset { pos, version: version[0], flags, offsets })
}

/// A struct storing the position and size of an atom.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AtomInfo {
    pub ident: Ident,
    pub pos: u64,
    pub len: usize,
}

impl AtomInfo {
    fn new(ident: Ident, pos: u64, len: usize) -> Self {
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
