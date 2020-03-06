use byteorder::{ReadBytesExt, BigEndian};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::Atom;
use crate::{Error, ErrorKind};

const UNKNOWN: u32 = 0;
const UTF8: u32 = 1;
const UTF16: u32 = 2;
const JPEG: u32 = 613;
const PNG: u32 = 614;

/// A structure representing the different types of content an Atom might have.
pub enum Content {
    /// A list of children Atoms
    Atoms(Vec<Atom>),
    /// Raw Data
    RawData(Data),
    /// A value containing Data in a structure defined by a datatype code
    TypedData(Data),
    /// Nothing
    Empty,
}

/// A struct that holds the different types of data an atom can contain. More at:
/// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34
pub enum Data {
    Unknown(crate::Result<Vec<u8>>),
    UTF8(crate::Result<String>),
    UTF16(crate::Result<String>),
    JPEG(crate::Result<Vec<u8>>),
    PNG(crate::Result<Vec<u8>>),
    Unparsed,
}

impl Content {
    pub fn parse(&mut self, reader: &mut BufReader<File>, length: usize) -> crate::Result<()> {
        match self {
            Content::Atoms(v) => {
                Atom::parse_atoms(v, reader, length)?
            }
            Content::RawData(d) => d.parse(reader, length)?,
            Content::TypedData(d) => d.parse(reader, length)?,
            Content::Empty => (),
        }

        Ok(())
    }

    pub fn atoms() -> Content {
        Content::Atoms(Vec::new())
    }

    pub fn atom(head: [u8; 4], offset: usize, content: Content) -> Content {
        let mut v = Vec::new();
        v.push(Atom::with(head, offset, content));
        Content::Atoms(v)
    }

    pub fn data_atom(offset: usize) -> Content {
        Content::atom(*b"data", offset, Content::TypedData(Data::Unparsed))
    }

    pub fn add_atom(self, f: [u8; 4], offset: usize, content: Content) -> Content {
        if let Content::Atoms(mut atoms) = self {
            atoms.push(Atom::with(f, offset, content));
            Content::Atoms(atoms)
        } else {
            self
        }
    }

    pub fn add_data_atom(self, offset: usize) -> Content {
        self.add_atom(*b"data", offset, Content::TypedData(Data::Unparsed))
    }
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Content::Atoms(a) => write!(f, "Content::Atoms{{ {:#?} }}", a),
            Content::TypedData(d) => write!(f, "{:?}", d),
            Content::RawData(d) => write!(f, "Content::RawData{{ {:?} }}", d),
            Content::Empty => write!(f, "Content::Empty")
        }
    }
}

impl Data {
    pub fn parse(&mut self, reader: &mut BufReader<File>, length: usize) -> crate::Result<()> {
        match *self {
            Data::Unparsed => {
                if length > 8 {
                    let datatype = match reader.read_u32::<BigEndian>() {
                        Ok(d) => d,
                        Err(e) => return Err(crate::Error::from(e)),
                    };

                    // consuming 4 byte data offset
                    if let Err(e) = reader.read_u32::<BigEndian>() {
                        return Err(crate::Error::from(e));
                    }

                    match datatype {
                        UNKNOWN => *self = Data::Unknown(Data::read_to_u8_vec(reader, length - 8)),
                        UTF8 => *self = Data::UTF8(Data::read_utf8(reader, length - 8)),
                        UTF16 => *self = Data::UTF16(Data::read_utf16(reader, length - 8)),
                        JPEG => *self = Data::JPEG(Data::read_to_u8_vec(reader, length - 8)),
                        PNG => *self = Data::PNG(Data::read_to_u8_vec(reader, length - 8)),
                        _ => *self = Data::Unparsed,
                    }
                }
            }
            Data::Unknown(_) => *self = Data::Unknown(Data::read_to_u8_vec(reader, length)),
            Data::UTF8(_) => *self = Data::UTF8(Data::read_utf8(reader, length)),
            Data::UTF16(_) => *self = Data::UTF16(Data::read_utf16(reader, length)),
            Data::JPEG(_) => *self = Data::JPEG(Data::read_to_u8_vec(reader, length)),
            Data::PNG(_) => *self = Data::PNG(Data::read_to_u8_vec(reader, length)),
        }

        Ok(())
    }

    pub fn read_to_u8_vec(reader: &mut BufReader<File>, length: usize) -> crate::Result<Vec<u8>> {
        let mut buff = vec![0u8; length];

        if let Err(e) = reader.read_exact(&mut buff) {
            return Err(Error::from(e));
        }

        Ok(buff)
    }

    pub fn read_to_u16_vec(reader: &mut BufReader<File>, length: usize) -> crate::Result<Vec<u16>> {
        let mut buff = vec![0u16; length];

        if let Err(e) = reader.read_u16_into::<BigEndian>(&mut buff) {
            return Err(Error::from(e));
        }

        Ok(buff)
    }

    pub fn read_utf8(reader: &mut BufReader<File>, length: usize) -> crate::Result<String> {
        let data = Data::read_to_u8_vec(reader, length)?;

        match String::from_utf8(data.clone()) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::from(e)),
        }
    }

    pub fn read_utf16(reader: &mut BufReader<File>, length: usize) -> crate::Result<String> {
        let data = Data::read_to_u16_vec(reader, length / 2)?;

        if length % 2 == 1 {
            reader.read_u32::<BigEndian>()?;
        }

        match String::from_utf16(&data) {
            Ok(s) => Ok(s),
            Err(e) => Err(crate::Error::from(e)),
        }
    }

    pub fn empty_utf8() -> Data {
        Data::UTF8(Err(Error::new(ErrorKind::EmptyData, "Empty uf8 data")))
    }

    pub fn empty_unknown() -> Data {
        Data::Unknown(Err(Error::new(ErrorKind::EmptyData, "Empty data")))
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Unknown(d) => write!(f, "Unknown{{ {:?} }}", d),
            Data::UTF8(d) => write!(f, "UTF8{{ {:?} }}", d),
            Data::UTF16(d) => write!(f, "UTF16{{ {:?} }}", d),
            Data::JPEG(d) => write!(f, "JPEG{{ {:?} }}", d),
            Data::PNG(d) => write!(f, "PNG{{ {:?} }}", d),
            Data::Unparsed => write!(f, "Unparsed"),
        }
    }
}