use std::{fmt, io};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{Content, Data, ErrorKind, Tag};

/// A list of valid file types defined by the `ftyp` `Atom`.
const VALID_FILE_TYPES: [&str; 2] = ["M4A ", "M4B "];

/// Identifier of an `Atom` containing information about the filetype.
pub const FILE_TYPE: [u8; 4] = *b"ftyp";
/// Identifier of an `Atom` containing a sturcture of children storing metadata.
pub const MOVIE: [u8; 4] = *b"moov";
/// Identifier of an `Atom` containing user metadata.
pub const USER_DATA: [u8; 4] = *b"udta";
/// Identifier of an `Atom` containing a metadata item list.
pub const METADATA: [u8; 4] = *b"meta";
/// Identifier of an `Atom` containing a list of metadata atoms.
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
pub const DISK_NUMBER: [u8; 4] = *b"disk";
pub const ENCODER: [u8; 4] = *b"\xa9too";
pub const GENERIC_GENRE: [u8; 4] = *b"gnre";
pub const GENRE: [u8; 4] = *b"\xa9gen";
pub const RATING: [u8; 4] = *b"rtng";
pub const TITLE: [u8; 4] = *b"\xa9nam";
pub const TRACK_NUMBER: [u8; 4] = *b"trkn";
pub const YEAR: [u8; 4] = *b"\xa9day";

// ITunes 4.2 atoms
pub const GROUPING: [u8; 4] = *b"\xa9grp";

// ITunes 4.9 atoms
pub const CATEGORY: [u8; 4] = *b"catg";
pub const EPISODE_GLOBAL_UNIQUE_ID: [u8; 4] = *b"egid";
pub const KEYWORD: [u8; 4] = *b"keyw";
pub const PODCAST: [u8; 4] = *b"pcst";
pub const PODCAST_URL: [u8; 4] = *b"purl";

// ITunes 5.0
pub const DESCRIPTION: [u8; 4] = *b"pcst";
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


/// List of standard genres found in the `gnre` `Atom`.
pub const GENRES: [(u16, &str); 80] = [
    (1, "Blues"),
    (2, "Classic rock"),
    (3, "Country"),
    (4, "Dance"),
    (5, "Disco"),
    (6, "Funk"),
    (7, "Grunge"),
    (8, "Hip,-Hop"),
    (9, "Jazz"),
    (10, "Metal"),
    (11, "New Age"),
    (12, "Oldies"),
    (13, "Other"),
    (14, "Pop"),
    (15, "Rhythm and Blues"),
    (16, "Rap"),
    (17, "Reggae"),
    (18, "Rock"),
    (19, "Techno"),
    (20, "Industrial"),
    (21, "Alternative"),
    (22, "Ska"),
    (23, "Death metal"),
    (24, "Pranks"),
    (25, "Soundtrack"),
    (26, "Euro-Techno"),
    (27, "Ambient"),
    (28, "Trip-Hop"),
    (29, "Vocal"),
    (30, "Jazz & Funk"),
    (31, "Fusion"),
    (32, "Trance"),
    (33, "Classical"),
    (34, "Instrumental"),
    (35, "Acid"),
    (36, "House"),
    (37, "Game"),
    (38, "Sound clip"),
    (39, "Gospel"),
    (40, "Noise"),
    (41, "Alternative Rock"),
    (42, "Bass"),
    (43, "Soul"),
    (44, "Punk"),
    (45, "Space"),
    (46, "Meditative"),
    (47, "Instrumental Pop"),
    (48, "Instrumental Rock"),
    (49, "Ethnic"),
    (50, "Gothic"),
    (51, "Darkwave"),
    (52, "Techno-Industrial"),
    (53, "Electronic"),
    (54, "Pop-Folk"),
    (55, "Eurodance"),
    (56, "Dream"),
    (57, "Southern Rock"),
    (58, "Comedy"),
    (59, "Cult"),
    (60, "Gangsta"),
    (61, "Top 41"),
    (62, "Christian Rap"),
    (63, "Pop/Funk"),
    (64, "Jungle"),
    (65, "Native US"),
    (66, "Cabaret"),
    (67, "New Wave"),
    (68, "Psychedelic"),
    (69, "Rave"),
    (70, "Show tunes"),
    (71, "Trailer"),
    (72, "Lo,-Fi"),
    (73, "Tribal"),
    (74, "Acid Punk"),
    (75, "Acid Jazz"),
    (76, "Polka"),
    (77, "Retro"),
    (78, "Musical"),
    (79, "Rock ’n’ Roll"),
    (80, "Hard Rock"),
];

/// A structure that represents a MPEG-4 metadata `Atom`.
pub struct Atom {
    /// The 4 byte identifier of the `Atom`.
    pub head: [u8; 4],
    /// The offset in bytes separating the head from the `Content`.
    pub offset: usize,
    /// The `Content` of an `Atom`.
    pub content: Content,
}

impl Atom {
    /// Creates a new empty `Atom`.
    pub fn new() -> Atom {
        Atom { head: *b"    ", offset: 0, content: Content::Empty }
    }

    /// Creates an `Atom` containing the provided `Content` at a n byte offset.
    pub fn with(head: [u8; 4], offset: usize, content: Content) -> Atom {
        Atom { head, offset, content }
    }

    /// Creates an `Atom` containing `Content::RawData` with the provided `Data`.
    pub fn with_raw_data(head: [u8; 4], offset: usize, data: Data) -> Atom {
        Atom::with(head, offset, Content::RawData(data))
    }

    pub fn data_atom() -> Atom {
        Atom::with(*b"data", 0, Content::TypedData(Data::Unparsed))
    }

    /// Attempts to read a `Tag` from the
    pub fn read_from(reader: &mut impl io::Read) -> crate::Result<Tag> {
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

        Ok(Tag::with(moov))
    }

    /// Attempts to recursively parse the `Atom` from the reader.
    pub fn parse(&mut self, reader: &mut impl io::Read) -> crate::Result<()> {
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
                Data::read_to_u8_vec(reader, length - 8)?;
            }
        }
    }

    /// Attempts to recursively parse the list of atoms from the reader.
    pub fn parse_atoms(atoms: &mut Vec<Atom>, reader: &mut impl io::Read, length: usize) -> crate::Result<()> {
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
                Data::read_to_u8_vec(reader, atom_length - 8)?;
            }

            parsed_bytes += atom_length;
        }

        Ok(())
    }

    /// Attempts to parse a 32 bit unsigned integer determining the size of the `Atom` in bytes and
    /// the following 4 byte head from the reader.
    pub fn parse_head(reader: &mut impl io::Read) -> crate::Result<(usize, [u8; 4])> {
        let length = match reader.read_u32::<BigEndian>() {
            Ok(l) => l as usize,
            Err(e) => return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length",
            )),
        };
        let mut f = [0_u8; 4];
        if let Err(e) = reader.read_exact(&mut f) {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading byte data",
            ));
        }

        Ok((length, f))
    }

    /// Attempts to parse the content of the provided length from the reader.
    pub fn parse_content(&mut self, reader: &mut impl io::Read, length: usize) -> crate::Result<()> {
        if length > 8 {
            if self.offset != 0 {
                Data::read_to_u8_vec(reader, self.offset)?;
            }
            self.content.parse(reader, length - 8)?;
        } else {
            self.content = Content::Empty;
        };

        Ok(())
    }

    /// Attempts to return the first children `Atom` if the `Content` is of type `Content::Atoms`.
    pub fn first_child(&self) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            return v.first();
        }

        None
    }

    /// Return true if the filetype specified in the `ftyp` atom is valid otherwise false.
    pub fn is_valid_filetype(&self) -> bool {
        if let Content::RawData(Data::UTF8(Ok(s))) = &self.content {
            for f in &VALID_FILE_TYPES {
                if s.starts_with(f) {
                    return true;
                }
            }
        }

        return false;
    }

    /// Returns a `Atom` hierarchy needed to parse the filetype.
    fn filetype_atom() -> Atom {
        Atom::with_raw_data(FILE_TYPE, 0, Data::empty_utf8())
    }

    /// Returns a `Atom` hierarchy needed to parse metadata.
    fn metadata_atom() -> Atom {
        Atom::with(
            MOVIE, 0, Content::with_atom(
                USER_DATA, 0, Content::with_atom(
                    METADATA, 4, Content::with_atom(
                        ITEM_LIST, 0, Content::atoms()
                            .add_atom_with(ALBUM, 0, Content::data_atom())
                            .add_atom_with(ALBUM_ARTIST, 0, Content::data_atom())
                            .add_atom_with(ARTIST, 0, Content::data_atom())
                            .add_atom_with(ARTWORK, 0, Content::data_atom())
                            .add_atom_with(BPM, 0, Content::data_atom())
                            .add_atom_with(COMMENT, 0, Content::data_atom())
                            .add_atom_with(COMPILATION, 0, Content::data_atom())
                            .add_atom_with(COMPOSER, 0, Content::data_atom())
                            .add_atom_with(COPYRIGHT, 0, Content::data_atom())
                            .add_atom_with(DISK_NUMBER, 0, Content::data_atom())
                            .add_atom_with(ENCODER, 0, Content::data_atom())
                            .add_atom_with(GENERIC_GENRE, 0, Content::data_atom())
                            .add_atom_with(GENRE, 0, Content::data_atom())
                            .add_atom_with(RATING, 0, Content::data_atom())
                            .add_atom_with(TITLE, 0, Content::data_atom())
                            .add_atom_with(TRACK_NUMBER, 0, Content::data_atom())
                            .add_atom_with(YEAR, 0, Content::data_atom())
                            .add_atom_with(GROUPING, 0, Content::data_atom())
                            .add_atom_with(CATEGORY, 0, Content::data_atom())
                            .add_atom_with(EPISODE_GLOBAL_UNIQUE_ID, 0, Content::data_atom())
                            .add_atom_with(KEYWORD, 0, Content::data_atom())
                            .add_atom_with(PODCAST, 0, Content::data_atom())
                            .add_atom_with(PODCAST_URL, 0, Content::data_atom())
                            .add_atom_with(DESCRIPTION, 0, Content::data_atom())
                            .add_atom_with(LYRICS, 0, Content::data_atom())
                            .add_atom_with(TV_EPISODE, 0, Content::data_atom())
                            .add_atom_with(TV_EPISODE_NUMBER, 0, Content::data_atom())
                            .add_atom_with(TV_NETWORK_NAME, 0, Content::data_atom())
                            .add_atom_with(TV_SEASON, 0, Content::data_atom())
                            .add_atom_with(TV_SHOW_NAME, 0, Content::data_atom())
                            .add_atom_with(PURCHASE_DATE, 0, Content::data_atom())
                            .add_atom_with(GAPLESS_PLAYBACK, 0, Content::data_atom()),
                    ),
                ),
            ),
        )
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head_string: String = self.head.iter().map(|b| char::from(*b)).collect();
        write!(f, "Atom{{ {}, {}: {:#?} }}", head_string, self.offset, self.content)
    }
}