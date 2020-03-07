use std::{fmt, io};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{Atom, Error, ErrorKind};

/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
const UNKNOWN: u32 = 0;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
const UTF8: u32 = 1;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
const UTF16: u32 = 2;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
const JPEG: u32 = 13;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
const PNG: u32 = 14;

/// A structure representing the different types of content an Atom might have.
pub enum Content {
    /// A value containing an `Atom` `Vec`.
    Atoms(Vec<Atom>),
    /// A value containing raw `Data`.
    RawData(Data),
    /// A value containing structured `Data`.
    TypedData(Data),
    /// Empty `Content`.
    Empty,
}

impl Content {
    /// Creates a new `Content` of type `Content::Atoms` containing an empty `Vec`.
    pub fn atoms() -> Content {
        Content::Atoms(Vec::new())
    }

    /// Creates a new `Content` of type `Content::Atoms` containing the provided `Atom`.
    pub fn atom(atom: Atom) -> Content {
        Content::Atoms(vec![atom])
    }

    /// Creates a new `Content` of type `Content::Atoms` containing a data `Atom`.
    pub fn data_atom() -> Content {
        Content::atom(Atom::data_atom())
    }

    /// Creates a new `Content` of type `Content::Atoms` containing a new `Atom` with the provided
    /// head, offset and `Content`.
    pub fn with_atom(head: [u8; 4], offset: usize, content: Content) -> Content {
        Content::atom(Atom::with(head, offset, content))
    }

    /// Adds the provided `Atom` to the list of children if `self` is of type `Content::Atoms`.
    pub fn add_atom(self, atom: Atom) -> Content {
        if let Content::Atoms(mut atoms) = self {
            atoms.push(atom);
            Content::Atoms(atoms)
        } else {
            self
        }
    }

    /// Adds a data `Atom` to the list of children if `self` is of type `Content::Atoms`.
    pub fn add_data_atom(self) -> Content {
        self.add_atom(Atom::data_atom())
    }

    /// Adds a new `Atom` with the provided  head, offset and `Content` to the list of children if
    /// `self` is of type `Content::Atoms`.
    pub fn add_atom_with(self, f: [u8; 4], offset: usize, content: Content) -> Content {
        self.add_atom(Atom::with(f, offset, content))
    }

    /// Attempts to parse the `Content` from the reader.
    pub fn parse(&mut self, reader: &mut impl io::Read, length: usize) -> crate::Result<()> {
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

/// A struct that holds the different types of `Data` an `Atom` can contain following
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34).
pub enum Data {
    Unknown(crate::Result<Vec<u8>>),
    UTF8(crate::Result<String>),
    UTF16(crate::Result<String>),
    JPEG(crate::Result<Vec<u8>>),
    PNG(crate::Result<Vec<u8>>),
    UnknownCode(u32),
    Unparsed,
}

impl Data {
    /// Creates new `Data` of type `Data::Unknown` containing an `ErrorKind::EmptyData` Error.
    pub fn empty_unknown() -> Data {
        Data::Unknown(Err(Error::new(ErrorKind::EmptyData, "Empty data")))
    }

    /// Creates new `Data` of type `Data::UTF8` containing an `ErrorKind::EmptyData` Error.
    pub fn empty_utf8() -> Data {
        Data::UTF8(Err(Error::new(ErrorKind::EmptyData, "Empty uf8 data")))
    }

    /// Creates new `Data` of type `Data::UTF16` containing an `ErrorKind::EmptyData` Error.
    pub fn empty_utf16() -> Data {
        Data::UTF16(Err(Error::new(ErrorKind::EmptyData, "Empty uf16 data")))
    }

    /// Creates new `Data` of type `Data::JPEG` containing an `ErrorKind::EmptyData` Error.
    pub fn empty_jpeg() -> Data {
        Data::JPEG(Err(Error::new(ErrorKind::EmptyData, "Empty jpeg data")))
    }

    /// Creates new `Data` of type `Data::PNG` containing an `ErrorKind::EmptyData` Error.
    pub fn empty_png() -> Data {
        Data::JPEG(Err(Error::new(ErrorKind::EmptyData, "Empty png data")))
    }

    /// Attempts to parse the `Data` from the reader.
    pub fn parse(&mut self, reader: &mut impl io::Read, length: usize) -> crate::Result<()> {
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
                        _ => *self = Data::UnknownCode(datatype),
                    }
                }
            }
            Data::Unknown(_) => *self = Data::Unknown(Data::read_to_u8_vec(reader, length)),
            Data::UTF8(_) => *self = Data::UTF8(Data::read_utf8(reader, length)),
            Data::UTF16(_) => *self = Data::UTF16(Data::read_utf16(reader, length)),
            Data::JPEG(_) => *self = Data::JPEG(Data::read_to_u8_vec(reader, length)),
            Data::PNG(_) => *self = Data::PNG(Data::read_to_u8_vec(reader, length)),
            _ => (),
        }

        Ok(())
    }

    /// Attempts to read 8 bit unsigned integers from the reader to a `Vec` of size length.
    pub fn read_to_u8_vec(reader: &mut impl io::Read, length: usize) -> crate::Result<Vec<u8>> {
        let mut buff = vec![0u8; length];

        if let Err(e) = reader.read_exact(&mut buff) {
            return Err(Error::from(e));
        }

        Ok(buff)
    }

    /// Attempts to read 16 bit unsigned integers from the reader to a `Vec` of size length.
    pub fn read_to_u16_vec(reader: &mut impl io::Read, length: usize) -> crate::Result<Vec<u16>> {
        let mut buff = vec![0u16; length];

        if let Err(e) = reader.read_u16_into::<BigEndian>(&mut buff) {
            return Err(Error::from(e));
        }

        Ok(buff)
    }

    /// Attempts to read a utf-8 string from the reader.
    pub fn read_utf8(reader: &mut impl io::Read, length: usize) -> crate::Result<String> {
        let data = Data::read_to_u8_vec(reader, length)?;

        match String::from_utf8(data.clone()) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::from(e)),
        }
    }

    /// Attempts to read a utf-16 string from the reader.
    pub fn read_utf16(reader: &mut impl io::Read, length: usize) -> crate::Result<String> {
        let data = Data::read_to_u16_vec(reader, length / 2)?;

        if length % 2 == 1 {
            reader.read_u32::<BigEndian>()?;
        }

        match String::from_utf16(&data) {
            Ok(s) => Ok(s),
            Err(e) => Err(crate::Error::from(e)),
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Unknown(d) => write!(f, "Unknown{{ {:?} }}", d),
            Data::UTF8(d) => write!(f, "UTF8{{ {:?} }}", d),
            Data::UTF16(d) => write!(f, "UTF16{{ {:?} }}", d),
            Data::JPEG(_) => write!(f, "JPEG"),
            Data::PNG(_) => write!(f, "PNG"),
            Data::UnknownCode(c) => write!(f, "UnkownCode{{ {:?} }}", c),
            Data::Unparsed => write!(f, "Unparsed"),
        }
    }
}