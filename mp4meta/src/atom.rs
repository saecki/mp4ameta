use std::{fmt, io};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Content, Data, data, ErrorKind};

/// A list of valid file types defined by the `ftyp` atom.
const VALID_FILE_TYPES: [&str; 2] = ["M4A ", "M4B "];

/// Identifier of an atom containing information about the filetype.
pub const FILE_TYPE: [u8; 4] = *b"ftyp";
/// Identifier of an atom containing a sturcture of children storing metadata.
pub const MOVIE: [u8; 4] = *b"moov";
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
    pub head: [u8; 4],
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl Atom {
    /// Creates a new empty atom.
    pub fn new() -> Atom {
        Atom { head: *b"    ", offset: 0, content: Content::Empty }
    }

    /// Creates an atom containing the provided content at a n byte offset.
    pub fn with(head: [u8; 4], offset: usize, content: Content) -> Atom {
        Atom { head, offset, content }
    }

    /// Creates an atom containing `Content::RawData` with the provided data.
    pub fn with_raw_data(head: [u8; 4], offset: usize, data: Data) -> Atom {
        Atom::with(head, offset, Content::RawData(data))
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
        8 + self.content.len()
    }

    /// Attempts to read MPEG-4 audio metadata from the reader.
    pub fn read_from(reader: &mut (impl io::Read + io::Seek)) -> crate::Result<Vec<Atom>> {
        let mut ftyp = Atom::filetype_atom();
        ftyp.parse(reader)?;

        if !ftyp.is_valid_filetype() {
            return Err(crate::Error::new(
                ErrorKind::NoTag,
                "File does not contain MPEG-4 audio metadata",
            ));
        }

        let mut moov = Atom::metadata_atom();
        moov.parse(reader)?;

        if let Some(udta) = moov.first_child() {
            if let Some(meta) = udta.first_child() {
                if let Some(ilst) = meta.first_child() {
                    if let Content::Atoms(atoms) = &ilst.content {
                        return Ok(atoms.to_vec());
                    }
                }
            }
        }

        Err(crate::Error::new(ErrorKind::Parsing, "wrong metadata structure"))
    }

    /// Attempts to write the MPEG-4 audio metadata to the writer.
    pub fn write_to(&self, writer: &mut impl io::Write) -> crate::Result<()> {
        writer.write_u32::<BigEndian>(self.len() as u32)?;
        writer.write(&self.head)?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Attempts to parse itself from the reader.
    pub fn parse(&mut self, reader: &mut (impl io::Read + io::Seek)) -> crate::Result<()> {
        loop {
            let h = match Atom::parse_head(reader) {
                Ok(h) => h,
                Err(e) => match &e.kind {
                    crate::ErrorKind::Io(ioe) => if ioe.kind() == io::ErrorKind::UnexpectedEof {
                        return Err(crate::Error::new(
                            ErrorKind::AtomNotFound(self.head),
                            "Reached EOF without finding a matching atom",
                        ));
                    } else {
                        return Err(e);
                    },
                    _ => return Err(e),
                },
            };
            let length = h.0;
            let head = h.1;

            if head == self.head {
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
            let h = Atom::parse_head(reader)?;
            let atom_length = h.0;
            let atom_head = h.1;

            let mut parsed = false;
            for a in atoms.into_iter() {
                if atom_head == a.head {
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

    /// Attempts to parse a 32 bit unsigned integer determining the size of the atom in bytes and
    /// the following 4 byte head from the reader.
    pub fn parse_head(reader: &mut (impl io::Read + io::Seek)) -> crate::Result<(usize, [u8; 4])> {
        let length = match reader.read_u32::<BigEndian>() {
            Ok(l) => l as usize,
            Err(e) => return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length",
            )),
        };
        let mut head = [0u8; 4];
        if let Err(e) = reader.read_exact(&mut head) {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom head",
            ));
        }

        Ok((length, head))
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

    /// Attempts to return the first children atom if it's content is of type `Content::Atoms`.
    pub fn first_child(&self) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            return v.first();
        }

        None
    }

    /// Attempts to return the first children atom if it's content is of type `Content::Atoms`.
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
        Atom::with(
            MOVIE, 0, Content::atom_with(
                USER_DATA, 0, Content::atom_with(
                    METADATA, 4, Content::atom_with(
                        ITEM_LIST, 0, Content::atoms()
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
    pub fn empty_metadata_atom() -> Atom {
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
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head &&
            self.offset == other.offset &&
            self.content == other.content
    }

    fn ne(&self, other: &Self) -> bool {
        self.head != other.head ||
            self.offset != other.offset ||
            self.content != other.content
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head_string: String = self.head.iter().map(|b| char::from(*b)).collect();
        write!(f, "Atom{{ {}, {}: {:#?} }}", head_string, self.offset, self.content)
    }
}