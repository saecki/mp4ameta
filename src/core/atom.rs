use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Deref;

use crate::{data, Content, ContentT, Data, DataT, ErrorKind, Tag};

/// A lowercase list of valid file types defined by the `ftyp` atom.
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

// iTunes 4.0 atoms
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
/// (`rtng`)
pub const ADVISORY_RATING: Ident = Ident(*b"rtng");
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

/// (`----`)
pub const WILDCARD: Ident = Ident(*b"----");

lazy_static! {
    /// Lazily initialized static reference to a `ftyp` atom template.
    pub static ref FILETYPE_ATOM_T: AtomT = filetype_atom_t();
    /// Lazily initialized static reference to an atom metadata hierarchy template needed to parse metadata.
    pub static ref METADATA_ATOM_T: AtomT = metadata_atom_t();
    /// Lazily initialized static reference to an atom hierarchy template leading to an empty `ilst` atom.
    pub static ref ITEM_LIST_ATOM_T: AtomT = item_list_atom_t();
    /// Lazily initialized static reference to an atom hierarchy template leading to a `stco` atom.
    pub static ref SAMPLE_TABLE_CHUNK_OFFSET_ATOM_T: AtomT = sample_table_chunk_offset_atom_t();
}

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

    /// Attempts to parse an atom, that matches the template, from the `reader`.  This should only
    /// be used if the atom has to be in this exact position, if the parsed and expected
    /// `identifier`s don't match this will return an error.
    pub fn parse_next(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let (length, ident) = match parse_head(reader) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        if ident == self.ident {
            match self.parse_content(reader, length) {
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

    /// Attempts to parse an atom, that matches the template, from the reader.
    pub fn parse(&self, reader: &mut (impl Read + Seek)) -> crate::Result<Atom> {
        let current_position = reader.seek(SeekFrom::Current(0))?;
        let complete_length = reader.seek(SeekFrom::End(0))?;
        let length = (complete_length - current_position) as usize;
        reader.seek(SeekFrom::Start(current_position))?;

        let mut parsed_bytes = 0;

        while parsed_bytes < length {
            let (atom_length, atom_ident) = parse_head(reader)?;

            if atom_ident == self.ident {
                return match self.parse_content(reader, atom_length) {
                    Ok(c) => Ok(Atom::new(self.ident, self.offset, c)),
                    Err(e) => Err(crate::Error::new(
                        e.kind,
                        format!("Error reading {}: {}", atom_ident, e.description),
                    )),
                };
            } else {
                reader.seek(SeekFrom::Current((atom_length - 8) as i64))?;
            }

            parsed_bytes += atom_length;
        }

        Err(crate::Error::new(
            ErrorKind::AtomNotFound(self.ident),
            format!("No {} atom found", self.ident),
        ))
    }

    /// Attempts to parse the atom template's content from the reader.
    pub fn parse_content(
        &self,
        reader: &mut (impl Read + Seek),
        length: usize,
    ) -> crate::Result<Content> {
        if length > 8 {
            if self.offset != 0 {
                reader.seek(SeekFrom::Current(self.offset as i64))?;
            }
            self.content.parse(reader, length - 8 - self.offset)
        } else {
            Ok(Content::Empty)
        }
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

    let moov = METADATA_ATOM_T.parse(reader)?;
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

    let mut mdat_pos = None;
    while let Ok((length, ident)) = parse_head(&mut reader) {
        match ident {
            MEDIA_DATA => {
                mdat_pos = Some(reader.seek(SeekFrom::Current(0))?);
            }
            _ => {
                reader.seek(SeekFrom::Current(length as i64 - 8))?;
            }
        }
    }
    let mdat_pos = match mdat_pos {
        Some(p) => p,
        None => {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(MEDIA_DATA),
                "No media data atom found".to_owned(),
            ))
        }
    };

    reader.seek(SeekFrom::Start(0))?;

    // TODO: support contained `co64` atoms (64 bit chunks) [Chunk offset atoms](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-25715)
    // TODO: support multiple tracks
    let mut stco_destination = SAMPLE_TABLE_CHUNK_OFFSET_ATOM_T.deref();
    let mut chunk_offsets = Vec::new();
    let mut chunk_offset_pos = None;
    while let Ok((length, ident)) = parse_head(&mut reader) {
        match ident {
            SAMPLE_TABLE_CHUNK_OFFSET => {
                let _version = data::read_u32(&mut reader)?;
                let entries = data::read_u32(&mut reader)?;

                chunk_offset_pos = Some(reader.seek(SeekFrom::Current(0))?);

                for _ in 0..entries {
                    let offset = data::read_u32(&mut reader)?;
                    chunk_offsets.push(offset);
                }
                break;
            }
            _ if ident == stco_destination.ident => match stco_destination.first_child() {
                Some(a) => stco_destination = a,
                None => {
                    return Err(crate::Error::new(
                        crate::ErrorKind::AtomNotFound(SAMPLE_TABLE_CHUNK_OFFSET),
                        "No sample table content offset atom found".to_owned(),
                    ))
                }
            },
            _ => {
                reader.seek(SeekFrom::Current(length as i64 - 8))?;
            }
        }
    }
    let chunk_offset_pos = match chunk_offset_pos {
        Some(p) => p,
        None => {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(SAMPLE_TABLE_CHUNK_OFFSET),
                "No sample table content offset atom found".to_owned(),
            ))
        }
    };

    reader.seek(SeekFrom::Start(0))?;

    let mut atom_pos_and_len = Vec::new();
    let mut ilst_destination = ITEM_LIST_ATOM_T.deref();

    while let Ok((length, ident)) = parse_head(&mut reader) {
        match ident {
            ITEM_LIST => {
                let pos = reader.seek(SeekFrom::Current(0))? as usize - 8;
                atom_pos_and_len.push((pos, length));
                break;
            }
            _ if ident == ilst_destination.ident => {
                let pos = reader.seek(SeekFrom::Current(0))? as usize - 8;
                atom_pos_and_len.push((pos, length));

                reader.seek(SeekFrom::Current(ilst_destination.offset as i64))?;

                match ilst_destination.first_child() {
                    Some(a) => ilst_destination = a,
                    None => {
                        return Err(crate::Error::new(
                            crate::ErrorKind::AtomNotFound(ITEM_LIST),
                            "No item list atom found".to_owned(),
                        ))
                    }
                }
            }
            _ => {
                reader.seek(SeekFrom::Current(length as i64 - 8))?;
            }
        }
    }

    let mut writer = BufWriter::new(file);
    let old_file_len = reader.seek(SeekFrom::End(0))?;
    let metadata_pos = atom_pos_and_len[atom_pos_and_len.len() - 1].0 + 8;
    let old_metadata_len = atom_pos_and_len[atom_pos_and_len.len() - 1].1 - 8;
    let new_metadata_len = atoms.iter().map(|a| a.len()).sum::<usize>();
    let metadata_len_diff = new_metadata_len as i32 - old_metadata_len as i32;

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
            let free = Atom::new(FREE, (len_diff * -1 - 8) as usize, Content::Empty);
            free.write_to(&mut writer)?;
        }
        _ => {
            // reading additional data after metadata
            let additional_data_len = old_file_len as usize - (metadata_pos + old_metadata_len);
            let mut additional_data = Vec::with_capacity(additional_data_len);
            reader.seek(SeekFrom::Start((metadata_pos + old_metadata_len) as u64))?;
            reader.read_to_end(&mut additional_data)?;

            // adjusting the file length
            file.set_len((old_file_len as i64 + metadata_len_diff as i64) as u64)?;

            // adjusting the atom lengths
            for (pos, len) in atom_pos_and_len.iter() {
                writer.seek(SeekFrom::Start(*pos as u64))?;
                writer.write_all(&((*len as i32 + metadata_len_diff) as u32).to_be_bytes())?;
            }

            // writing metadata
            writer.seek(SeekFrom::Current(4))?;
            for a in atoms {
                a.write_to(&mut writer)?;
            }

            // writing additional data after metadata
            writer.write_all(&additional_data)?;

            let moov_pos = atom_pos_and_len[0].0;
            if mdat_pos > moov_pos as u64 {
                writer.seek(SeekFrom::Start(chunk_offset_pos as u64))?;

                for co in chunk_offsets.iter() {
                    let new_offset = (*co as i32 + metadata_len_diff) as u32;
                    writer.write_all(&new_offset.to_be_bytes())?;
                }
            }
        }
    }
    writer.flush()?;

    Ok(())
}

/// Attempts to dump the metadata atoms to the writer. This doesn't include a complete MPEG-4
/// container hierarchy and won't result in a usable file.
pub fn dump_tag_to(writer: &mut impl Write, atoms: Vec<Atom>) -> crate::Result<()> {
    #[rustfmt::skip]
    let ftyp = Atom::new( FILETYPE, 0, Content::RawData(
        Data::Utf8("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".to_owned())),
    );
    #[rustfmt::skip]
    let moov = Atom::new(MOVIE, 0, Content::atom(
        Atom::new( USER_DATA, 0, Content::atom(
            Atom::new( METADATA, 4, Content::atom(
                Atom::new(ITEM_LIST, 0, Content::Atoms(atoms))
            )),
        )),
    ));

    ftyp.write_to(writer)?;
    moov.write_to(writer)?;

    Ok(())
}

/// Attempts to parse the list of atoms, matching the templates, from the reader.
pub fn parse_atoms(
    atoms: &[AtomT],
    reader: &mut (impl Read + Seek),
    length: usize,
) -> crate::Result<Vec<Atom>> {
    let mut parsed_bytes = 0;
    let mut parsed_atoms = Vec::with_capacity(atoms.len());

    while parsed_bytes < length {
        let (atom_length, atom_ident) = parse_head(reader)?;

        let mut parsed = false;
        for a in atoms {
            if atom_ident == a.ident {
                match a.parse_content(reader, atom_length) {
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

        if atom_length > 8 && !parsed {
            reader.seek(SeekFrom::Current((atom_length - 8) as i64))?;
        }

        parsed_bytes += atom_length;
    }

    Ok(parsed_atoms)
}

/// Attempts to parse the atom's head containing a 32 bit unsigned integer determining the size of
/// the atom in bytes and the following 4 byte identifier from the reader.
pub fn parse_head(reader: &mut (impl Read + Seek)) -> crate::Result<(usize, Ident)> {
    let length = match data::read_u32(reader) {
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

    Ok((length, Ident(ident)))
}

/// Returns an `ftyp` atom template needed to parse the filetype.
fn filetype_atom_t() -> AtomT {
    AtomT::new(FILETYPE, 0, ContentT::RawData(DataT::new(data::UTF8)))
}

/// Returns an atom metadata hierarchy template needed to parse metadata.
#[rustfmt::skip]
fn metadata_atom_t() -> AtomT {
    AtomT::new( MOVIE, 0, ContentT::Atoms(vec![
        AtomT::new(MOVIE_HEADER, 0, ContentT::RawData(
            DataT::new(data::RESERVED)
        )),
        AtomT::new(USER_DATA, 0, ContentT::atom_t(
            AtomT::new(METADATA, 4, ContentT::atom_t(
                AtomT::new(ITEM_LIST, 0, ContentT::Atoms(vec![
                    AtomT::new(WILDCARD, 0, ContentT::Atoms(vec![
                        AtomT::data_atom(),
                        AtomT::mean_atom(),
                        AtomT::name_atom(),
                    ])),
                    AtomT::new(ADVISORY_RATING, 0, ContentT::data_atom_t()),
                    AtomT::new(ALBUM, 0, ContentT::data_atom_t()),
                    AtomT::new(ALBUM_ARTIST, 0, ContentT::data_atom_t()),
                    AtomT::new(ARTIST, 0, ContentT::data_atom_t()),
                    AtomT::new(BPM, 0, ContentT::data_atom_t()),
                    AtomT::new(CATEGORY, 0, ContentT::data_atom_t()),
                    AtomT::new(COMMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(COMPILATION, 0, ContentT::data_atom_t()),
                    AtomT::new(COMPOSER, 0, ContentT::data_atom_t()),
                    AtomT::new(COPYRIGHT, 0, ContentT::data_atom_t()),
                    AtomT::new(CUSTOM_GENRE, 0, ContentT::data_atom_t()),
                    AtomT::new(DESCRIPTION, 0, ContentT::data_atom_t()),
                    AtomT::new(DISC_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(ENCODER, 0, ContentT::data_atom_t()),
                    AtomT::new(GAPLESS_PLAYBACK, 0, ContentT::data_atom_t()),
                    AtomT::new(GROUPING, 0, ContentT::data_atom_t()),
                    AtomT::new(KEYWORD, 0, ContentT::data_atom_t()),
                    AtomT::new(LYRICS, 0, ContentT::data_atom_t()),
                    AtomT::new(MEDIA_TYPE, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT_COUNT, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT_INDEX, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST_EPISODE_GLOBAL_UNIQUE_ID, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST_URL, 0, ContentT::data_atom_t()),
                    AtomT::new(PURCHASE_DATE, 0, ContentT::data_atom_t()),
                    AtomT::new(SHOW_MOVEMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(STANDARD_GENRE, 0, ContentT::data_atom_t()),
                    AtomT::new(TITLE, 0, ContentT::data_atom_t()),
                    AtomT::new(TRACK_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_EPISODE, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_EPISODE_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_NETWORK_NAME, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_SEASON, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_SHOW_NAME, 0, ContentT::data_atom_t()),
                    AtomT::new(WORK, 0, ContentT::data_atom_t()),
                    AtomT::new(YEAR, 0, ContentT::data_atom_t()),
                    AtomT::new(ARTWORK, 0, ContentT::data_atom_t()),
                ])),
            )),
        )),
    ]))
}

/// Returns an atom hierarchy leading to an empty `ilst` atom template.
#[rustfmt::skip]
fn item_list_atom_t() -> AtomT {
    AtomT::new(MOVIE, 0, ContentT::atom_t(
        AtomT::new(USER_DATA, 0, ContentT::atom_t(
            AtomT::new(METADATA, 4, ContentT::atom_t(
                AtomT::new(ITEM_LIST, 0, ContentT::atoms_t())
            ))
        ))
    ))
}

/// Returns an atom hierarchy leading to a `stco` atom template.
#[rustfmt::skip]
fn sample_table_chunk_offset_atom_t() -> AtomT {
    AtomT::new(MOVIE, 0, ContentT::atom_t(
        AtomT::new(TRACK, 0, ContentT::atom_t(
            AtomT::new(MEDIA, 0, ContentT::atom_t(
                AtomT::new(METADATA_INFORMATION, 0, ContentT::atom_t(
                    AtomT::new(SAMPLE_TABLE, 0, ContentT::atom_t(
                        AtomT::new(SAMPLE_TABLE_CHUNK_OFFSET, 0, ContentT::RawData(DataT::new(data::RESERVED)))
                    ))
                )),
            )),
        )),
    ))
}
