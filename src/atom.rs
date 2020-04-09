use std::{fmt, io};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Content, Data, data, ErrorKind, Tag};

/// A list of valid file types defined by the `ftyp` atom.
const VALID_FILE_TYPES: [&str; 2] = ["M4A ", "M4B "];

/// Identifier of an atom containing information about the filetype.
pub const FILE_TYPE: [u8; 4] = *b"ftyp";
/// Identifier of an atom containing a sturcture of children storing metadata.
pub const MOVIE: [u8; 4] = *b"moov";
pub const TRACK: [u8; 4] = *b"trak";
pub const MEDIA: [u8; 4] = *b"mdia";
pub const MEDIA_HEADER: [u8; 4] = *b"mdhd";
/// Identifier of an atom containing user metadata.
pub const USER_DATA: [u8; 4] = *b"udta";
/// Identifier of an atom containing a metadata item list.
pub const METADATA: [u8; 4] = *b"meta";
/// Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: [u8; 4] = *b"ilst";

// ITunes 4.0 atoms
pub const ALBUM: [u8; 4] = *b"\xa9alb";
pub const ALBUM_ARTIST: [u8; 4] = *b"aART";
pub const ARTIST: [u8; 4] = *b"\xa9ART";
pub const ARTWORK: [u8; 4] = *b"covr";
pub const BPM: [u8; 4] = *b"tmpo";
pub const COMMENT: [u8; 4] = *b"\xa9cmt";
pub const COMPILATION: [u8; 4] = *b"cpil";
pub const COMPOSER: [u8; 4] = *b"\xa9wrt";
pub const COPYRIGHT: [u8; 4] = *b"cprt";
pub const CUSTOM_GENRE: [u8; 4] = *b"\xa9gen";
pub const DISK_NUMBER: [u8; 4] = *b"disk";
pub const ENCODER: [u8; 4] = *b"\xa9too";
pub const RATING: [u8; 4] = *b"rtng";
pub const STANDARD_GENRE: [u8; 4] = *b"gnre";
pub const TITLE: [u8; 4] = *b"\xa9nam";
pub const TRACK_NUMBER: [u8; 4] = *b"trkn";
pub const YEAR: [u8; 4] = *b"\xa9day";

// ITunes 4.2 atoms
pub const GROUPING: [u8; 4] = *b"\xa9grp";
pub const MEDIA_TYPE: [u8; 4] = *b"stik";

// ITunes 4.9 atoms
pub const CATEGORY: [u8; 4] = *b"catg";
pub const EPISODE_GLOBAL_UNIQUE_ID: [u8; 4] = *b"egid";
pub const KEYWORD: [u8; 4] = *b"keyw";
pub const PODCAST: [u8; 4] = *b"pcst";
pub const PODCAST_URL: [u8; 4] = *b"purl";

// ITunes 5.0
pub const DESCRIPTION: [u8; 4] = *b"desc";
pub const LYRICS: [u8; 4] = *b"\xa9lyr";

// ITunes 6.0
pub const TV_EPISODE: [u8; 4] = *b"tves";
pub const TV_EPISODE_NUMBER: [u8; 4] = *b"tven";
pub const TV_NETWORK_NAME: [u8; 4] = *b"tvnn";
pub const TV_SEASON: [u8; 4] = *b"tvsn";
pub const TV_SHOW_NAME: [u8; 4] = *b"tvsh";

// ITunes 6.0.2
pub const PURCHASE_DATE: [u8; 4] = *b"purd";

// ITunes 7.0
pub const GAPLESS_PLAYBACK: [u8; 4] = *b"pgap";

/// A structure that represents a MPEG-4 audio metadata atom.
#[derive(Clone)]
pub struct Atom {
    /// The 4 byte identifier of the atom.
    pub identifier: [u8; 4],
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl Atom {
    /// Creates a new empty atom.
    pub fn new() -> Atom {
        Atom { identifier: *b"    ", offset: 0, content: Content::Empty }
    }

    /// Creates an atom containing the provided content at a n byte offset.
    pub fn with(identifier: [u8; 4], offset: usize, content: Content) -> Atom {
        Atom { identifier, offset, content }
    }

    /// Creates an atom containing `Content::RawData` with the provided data.
    pub fn with_raw_data(identifier: [u8; 4], offset: usize, data: Data) -> Atom {
        Atom::with(identifier, offset, Content::RawData(data))
    }

    /// Creates a data atom containing unparsed `Content::TypedData`.
    pub fn data_atom() -> Atom {
        Atom::with(*b"data", 0, Content::TypedData(Data::Unparsed(data::TYPED)))
    }

    /// Creates a data atom containing `Content::TypedData` with the provided data.
    pub fn data_atom_with(data: Data) -> Atom {
        Atom::with(*b"data", 0, Content::TypedData(data))
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        8 + self.offset + self.content.len()
    }

    /// Attempts to read MPEG-4 audio metadata from the reader.
    pub fn read_from(reader: &mut (impl io::Read + io::Seek)) -> crate::Result<Tag> {
        let mut ftyp = Atom::filetype_atom();
        let mut moov = Atom::metadata_atom();

        let mut tag_atoms = Vec::new();
        let mut tag_readonly_atoms = Vec::new();

        ftyp.parse(reader)?;
        if !ftyp.is_valid_filetype() {
            return Err(crate::Error::new(
                ErrorKind::NoTag,
                "File does not contain MPEG-4 audio metadata",
            ));
        }

        moov.parse(reader)?;

        if let Some(trak) = moov.child(TRACK) {
            if let Some(mdia) = trak.child(MEDIA) {
                if let Some(mdhd) = mdia.child(MEDIA_HEADER) {
                    tag_readonly_atoms.push(mdhd.clone());
                }
            }
        }

        if let Some(udta) = moov.child(USER_DATA) {
            if let Some(meta) = udta.first_child() {
                if let Some(ilst) = meta.first_child() {
                    if let Content::Atoms(atoms) = &ilst.content {
                        tag_atoms = atoms.to_vec();
                    }
                }
            }
        }

        Ok(Tag::with(tag_atoms, tag_readonly_atoms))
    }

    /// Attempts to write the atom to the writer.
    pub fn write_to(&self, writer: &mut impl io::Write) -> crate::Result<()> {
        writer.write_u32::<BigEndian>(self.len() as u32)?;
        writer.write(&self.identifier)?;
        writer.write(&vec![0u8; self.offset])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Attempts to parse itself from the reader.
    pub fn parse(&mut self, reader: &mut (impl io::Read + io::Seek)) -> crate::Result<()> {
        loop {
            let (length, identifier) = match Atom::parse_head(reader) {
                Ok(h) => h,
                Err(e) => match &e.kind {
                    crate::ErrorKind::Io(ioe) => if ioe.kind() == io::ErrorKind::UnexpectedEof {
                        return Err(crate::Error::new(
                            ErrorKind::AtomNotFound(self.identifier),
                            "Reached EOF without finding a matching atom",
                        ));
                    } else {
                        return Err(e);
                    },
                    _ => return Err(e),
                },
            };

            if identifier == self.identifier {
                return self.parse_content(reader, length);
            } else if length > 8 {
                reader.seek(io::SeekFrom::Current((length - 8) as i64))?;
            }
        }
    }

    /// Attempts to parse the list of atoms from the reader.
    pub fn parse_atoms(atoms: &mut Vec<Atom>, reader: &mut (impl io::Read + io::Seek), length: usize) -> crate::Result<()> {
        let mut parsed_atoms = 0;
        let mut parsed_bytes = 0;
        let atom_count = atoms.len();

        while parsed_bytes < length && parsed_atoms < atom_count {
            let (atom_length, atom_identifier) = Atom::parse_head(reader)?;

            let mut parsed = false;
            for a in atoms.into_iter() {
                if atom_identifier == a.identifier {
                    a.parse_content(reader, atom_length)?;
                    parsed = true;
                    parsed_atoms += 1;
                    break;
                }
            }

            if atom_length > 8 && !parsed {
                reader.seek(io::SeekFrom::Current((atom_length - 8) as i64))?;
            }

            parsed_bytes += atom_length;
        }

        Ok(())
    }

    /// Locates the metadata item list atom and returns a list of tuples containing the position
    /// from the beginning of the file and length in bytes of the atoms inside the hierarchy leading
    /// to it.
    pub fn locate_metadata_item_list(reader: &mut (impl io::Read + io::Seek)) -> crate::Result<Vec<(usize, usize)>> {
        let mut atom_pos_and_len = Vec::new();
        let mut destination = &Atom::item_list_atom();
        let mut ftyp = Atom::filetype_atom();

        ftyp.parse(reader)?;

        if !ftyp.is_valid_filetype() {
            return Err(crate::Error::new(
                ErrorKind::NoTag,
                "File does not contain MPEG-4 audio metadata",
            ));
        }

        while let Ok((length, identifier)) = Atom::parse_head(reader) {
            if identifier == destination.identifier {
                atom_pos_and_len.push((reader.seek(io::SeekFrom::Current(0))? as usize - 8, length));
                reader.seek(io::SeekFrom::Current(destination.offset as i64))?;

                match destination.first_child() {
                    Some(a) => destination = a,
                    None => break,
                }
            } else {
                reader.seek(io::SeekFrom::Current(length as i64 - 8))?;
            }
        }

        Ok(atom_pos_and_len)
    }

    /// Attempts to parse the atoms head containing a 32 bit unsigned integer determining the size
    /// of the atom in bytes and the following 4 byte identifier from the reader.
    pub fn parse_head(reader: &mut (impl io::Read + io::Seek)) -> crate::Result<(usize, [u8; 4])> {
        let length = match reader.read_u32::<BigEndian>() {
            Ok(l) => l as usize,
            Err(e) => return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length",
            )),
        };
        let mut identifier = [0u8; 4];
        if let Err(e) = reader.read_exact(&mut identifier) {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom identifier",
            ));
        }

        Ok((length, identifier))
    }

    /// Attempts to parse the content of the provided length from the reader.
    pub fn parse_content(&mut self, reader: &mut (impl io::Read + io::Seek), length: usize) -> crate::Result<()> {
        if length > 8 {
            if self.offset != 0 {
                reader.seek(io::SeekFrom::Current(self.offset as i64))?;
            }
            self.content.parse(reader, length - 8)?;
        } else {
            self.content = Content::Empty;
        };

        Ok(())
    }


    /// Attempts to return a reference to the first children atom matching the identifier.
    pub fn child(&self, identifier: [u8; 4]) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            for a in v {
                if a.identifier == identifier {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Attempts to return a mutable reference to the first children atom matching the identifier.
    pub fn mut_child(&mut self, identifier: [u8; 4]) -> Option<&mut Atom> {
        if let Content::Atoms(v) = &mut self.content {
            for a in v {
                if a.identifier == identifier {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Attempts to return a reference to the first children atom.
    pub fn first_child(&self) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            return v.first();
        }

        None
    }

    /// Attempts to return a mutable reference to the first children atom.
    pub fn mut_first_child(&mut self) -> Option<&mut Atom> {
        if let Content::Atoms(v) = &mut self.content {
            return v.first_mut();
        }

        None
    }

    /// Return true if the filetype specified in the `ftyp` atom is valid, false otherwise.
    pub fn is_valid_filetype(&self) -> bool {
        if let Content::RawData(Data::Utf8(s)) = &self.content {
            for f in &VALID_FILE_TYPES {
                if s.starts_with(f) {
                    return true;
                }
            }
        }

        return false;
    }

    /// Returns a atom filetype hierarchy needed to parse the filetype:
    pub fn filetype_atom() -> Atom {
        Atom::with_raw_data(FILE_TYPE, 0, Data::Unparsed(data::UTF8))
    }

    /// Returns a atom metadata hierarchy needed to parse metadata.
    pub fn metadata_atom() -> Atom {
        Atom::with(MOVIE, 0, Content::atoms()
            .add_atom_with(TRACK, 0, Content::atoms()
                .add_atom_with(MEDIA, 0, Content::atoms()
                    .add_atom_with(MEDIA_HEADER, 0, Content::RawData(
                        Data::Unparsed(data::RESERVED)
                    )),
                ),
            )
            .add_atom_with(USER_DATA, 0, Content::atoms()
                .add_atom_with(METADATA, 4, Content::atoms()
                    .add_atom_with(ITEM_LIST, 0, Content::atoms()
                        .add_atom_with(ALBUM, 0, Content::data_atom())
                        .add_atom_with(ALBUM_ARTIST, 0, Content::data_atom())
                        .add_atom_with(ARTIST, 0, Content::data_atom())
                        .add_atom_with(BPM, 0, Content::data_atom())
                        .add_atom_with(CATEGORY, 0, Content::data_atom())
                        .add_atom_with(COMMENT, 0, Content::data_atom())
                        .add_atom_with(COMPILATION, 0, Content::data_atom())
                        .add_atom_with(COMPOSER, 0, Content::data_atom())
                        .add_atom_with(COPYRIGHT, 0, Content::data_atom())
                        .add_atom_with(CUSTOM_GENRE, 0, Content::data_atom())
                        .add_atom_with(DESCRIPTION, 0, Content::data_atom())
                        .add_atom_with(DISK_NUMBER, 0, Content::data_atom())
                        .add_atom_with(ENCODER, 0, Content::data_atom())
                        .add_atom_with(EPISODE_GLOBAL_UNIQUE_ID, 0, Content::data_atom())
                        .add_atom_with(GAPLESS_PLAYBACK, 0, Content::data_atom())
                        .add_atom_with(GROUPING, 0, Content::data_atom())
                        .add_atom_with(KEYWORD, 0, Content::data_atom())
                        .add_atom_with(LYRICS, 0, Content::data_atom())
                        .add_atom_with(MEDIA_TYPE, 0, Content::data_atom())
                        .add_atom_with(PODCAST, 0, Content::data_atom())
                        .add_atom_with(PODCAST_URL, 0, Content::data_atom())
                        .add_atom_with(PURCHASE_DATE, 0, Content::data_atom())
                        .add_atom_with(RATING, 0, Content::data_atom())
                        .add_atom_with(STANDARD_GENRE, 0, Content::data_atom())
                        .add_atom_with(TITLE, 0, Content::data_atom())
                        .add_atom_with(TRACK_NUMBER, 0, Content::data_atom())
                        .add_atom_with(TV_EPISODE, 0, Content::data_atom())
                        .add_atom_with(TV_EPISODE_NUMBER, 0, Content::data_atom())
                        .add_atom_with(TV_NETWORK_NAME, 0, Content::data_atom())
                        .add_atom_with(TV_SEASON, 0, Content::data_atom())
                        .add_atom_with(TV_SHOW_NAME, 0, Content::data_atom())
                        .add_atom_with(YEAR, 0, Content::data_atom())
                        .add_atom_with(ARTWORK, 0, Content::data_atom()),
                    ),
                ),
            ),
        )
    }

    /// Returns a atom metadata hierarchy.
    pub fn item_list_atom() -> Atom {
        Atom::with(
            MOVIE, 0, Content::atom_with(
                USER_DATA, 0, Content::atom_with(
                    METADATA, 4, Content::atom_with(
                        ITEM_LIST, 0, Content::atoms(),
                    ),
                ),
            ),
        )
    }

    /// Returns the identifier formatted as a string.
    pub fn format_identifier(identifier: [u8; 4]) -> String {
        identifier.iter().map(|b| char::from(*b)).collect()
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier &&
            self.offset == other.offset &&
            self.content == other.content
    }

    fn ne(&self, other: &Self) -> bool {
        self.identifier != other.identifier ||
            self.offset != other.offset ||
            self.content != other.content
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier_string = Atom::format_identifier(self.identifier);
        write!(f, "Atom{{ {}, {}, {:#?} }}", identifier_string, self.offset, self.content)
    }
}
