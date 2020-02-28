use byteorder::{ReadBytesExt, BigEndian};
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use crate::{Data, Content, Tag, ErrorKind};

/// A list of valid filetypes defined by the "ftyp" atom
const VALID_FILE_TYPES: [&str; 2] = ["M4A ", "M4B "];

/// Byte values of Atom heads
const FILE_TYPE: [u8; 4] = *b"ftyp";
const MOOVE: [u8; 4] = *b"moov";
const USER_DATA: [u8; 4] = *b"udta";
const METADATA: [u8; 4] = *b"meta";
const LIST: [u8; 4] = *b"ilst";

const ALBUM: [u8; 4] = *b"\xa9alb";
const ARTIST: [u8; 4] = *b"\xa9ART";
const ALBUM_ARTIST: [u8; 4] = *b"aART";
const COMMENT: [u8; 4] = *b"\xa9cmt";
const COMPOSER: [u8; 4] = *b"\xa9wrt";
const COVER: [u8; 4] = *b"covr";
const DISK_NUMBER: [u8; 4] = *b"disk";
const GENRE: [u8; 4] = *b"\xa9gen";
const GENERIC_GENRE: [u8; 4] = *b"gnre";
const LYRICS: [u8; 4] = *b"\xa9lyr";
const TITLE: [u8; 4] = *b"\xa9nam";
const TRACK_NUMBER: [u8; 4] = *b"trkn";
const YEAR: [u8; 4] = *b"\xa9day";

/// A structure that represents a MPEG-4 metadata atom
pub struct Atom {
    /// The 4 byte identifier of the atom.
    head: [u8; 4],
    /// The offset in bytes from the head's end to the beginning of the content.
    offset: usize,
    /// The content of the atom
    content: Content,
}

impl Atom {
    pub fn read_from(reader: &mut BufReader<File>) -> crate::Result<Tag> {
        let mut ftyp = Atom::filetype_atom();
        ftyp.parse(reader)?;

        println!("{:#?}", ftyp);

        if !ftyp.is_valid_filetype() {
            return Err(crate::Error::new(
                ErrorKind::NoTag,
                "File does not contain MPEG-4 audio metadata",
            ));
        }

        let mut moov = Atom::metadata_atom();
        moov.parse(reader)?;

        if let Content::Atoms(v) = &mut moov.content {
            if let Some(udta) = v.first_mut() {
                if let Content::Atoms(v) = &mut udta.content {
                    if let Some(meta) = v.first_mut() {
                        if let Content::Atoms(v) = &mut meta.content {
                            if let Some(ilst) = v.first_mut() {
                                if let Content::Atoms(v) = &mut ilst.content {
                                    println!("{:#?}", v)
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(crate::Error::new(
            ErrorKind::NoTag,
            "Error parsing atoms",
        ))
    }

    pub fn parse(&mut self, reader: &mut BufReader<File>) -> crate::Result<()> {
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

    pub fn parse_atoms(atoms: &mut Vec<Atom>, reader: &mut BufReader<File>, length: usize) -> crate::Result<()> {
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

    pub fn parse_head(reader: &mut BufReader<File>) -> crate::Result<(usize, [u8; 4])> {
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

    pub fn parse_content(&mut self, reader: &mut BufReader<File>, length: usize) -> crate::Result<()> {
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

    pub fn is_valid_filetype(self) -> bool {
        if let Content::RawData(Data::UTF8(Ok(s))) = self.content {
            for f in &VALID_FILE_TYPES {
                if s.starts_with(f) {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn new() -> Atom {
        Atom { head: *b"    ", offset: 0, content: Content::RawData(Data::empty_unknown()) }
    }

    pub fn with(f: [u8; 4], offset: usize, content: Content) -> Atom {
        Atom { head: f, offset, content }
    }

    pub fn with_raw_data(f: [u8; 4], offset: usize, data: Data) -> Atom {
        Atom::with(f, offset, Content::RawData(data))
    }

    fn filetype_atom() -> Atom {
        Atom::with_raw_data(FILE_TYPE, 0, Data::empty_utf8())
    }

    fn metadata_atom() -> Atom {
        Atom::with(
            MOOVE, 0, Content::atom(
                USER_DATA, 0, Content::atom(
                    METADATA, 4, Content::atom(
                        LIST, 0, Content::atoms()
                            .add_atom(ALBUM, 0, Content::data_atom(0))
                            .add_atom(ALBUM_ARTIST, 0, Content::data_atom(0))
                            .add_atom(ARTIST, 0, Content::data_atom(0))
                            .add_atom(COMMENT, 0, Content::data_atom(0))
                            .add_atom(COMPOSER, 0, Content::data_atom(0))
                            .add_atom(COVER, 0, Content::data_atom(0))
                            .add_atom(DISK_NUMBER, 0, Content::data_atom(0))
                            .add_atom(GENRE, 0, Content::data_atom(0))
                            .add_atom(GENERIC_GENRE, 0, Content::data_atom(0))
                            .add_atom(LYRICS, 0, Content::data_atom(0))
                            .add_atom(TITLE, 0, Content::data_atom(0))
                            .add_atom(TRACK_NUMBER, 0, Content::data_atom(0))
                            .add_atom(YEAR, 0, Content::data_atom(0)),
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