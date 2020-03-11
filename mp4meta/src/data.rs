use core::fmt;
use std::io;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Error, ErrorKind};

pub const TYPED: i32 = -1;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
pub const RESERVED: i32 = 0;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
pub const UTF8: i32 = 1;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
pub const UTF16: i32 = 2;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
pub const JPEG: i32 = 13;
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code
pub const PNG: i32 = 14;

/// A struct that holds the different types of data an `Atom` can contain following
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34).
#[derive(Clone)]
pub enum Data {
    /// A value containing reserved type data inside a `Option<Vec<u8>>`.
    Reserved(Vec<u8>),
    /// A value containing a `Option<String>` decoded from utf-8.
    Utf8(String),
    /// A value containing a `Option<String>` decoded from utf-16.
    Utf16(String),
    /// A value containing jpeg byte data inside a `Option<Vec<u8>>`.
    Jpeg(Vec<u8>),
    /// A value containing png byte data inside a `Option<Vec<u8>>`.
    Png(Vec<u8>),
    /// A value containing a `u32` determining the datatype of the data that is yet to be parsed.
    Unparsed(i32),
}

impl Data {
    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        match self {
            Data::Reserved(v) => v.len(),
            Data::Utf8(s) => s.len(),
            Data::Utf16(s) => s.len() * 2,
            Data::Jpeg(v) => v.len(),
            Data::Png(v) => v.len(),
            _ => 0,
        }
    }

    /// Attempts to parse itself from the reader.
    pub fn parse(&mut self, reader: &mut impl io::Read, length: usize) -> crate::Result<()> {
        if let Data::Unparsed(d) = *self {
            let mut datatype = d;
            let mut l = length;

            if d == TYPED {
                if length > 8 {
                    datatype = match reader.read_i32::<BigEndian>() {
                        Ok(d) => d,
                        Err(e) => return Err(crate::Error::from(e)),
                    };

                    // consuming 4 byte data offset
                    if let Err(e) = reader.read_u32::<BigEndian>() {
                        return Err(crate::Error::from(e));
                    }

                    l -= 8;
                } else {
                    return Err(crate::Error::new(
                        ErrorKind::Parsing,
                        "Typed data header to short",
                    ));
                }
            }

            match datatype {
                RESERVED => *self = Data::Reserved(Data::read_to_u8_vec(reader, l)?),
                UTF8 => *self = Data::Utf8(Data::read_utf8(reader, l)?),
                UTF16 => *self = Data::Utf16(Data::read_utf16(reader, l)?),
                JPEG => *self = Data::Jpeg(Data::read_to_u8_vec(reader, l)?),
                PNG => *self = Data::Png(Data::read_to_u8_vec(reader, l)?),
                _ => return Err(crate::Error::new(
                    ErrorKind::UnknownDataType(datatype),
                    "Unknown datatype code",
                )),
            }

            Ok(())
        } else {
            Err(crate::Error::new(
                ErrorKind::Parsing,
                "Data already parsed",
            ))
        }
    }

    pub fn write_typed(&self, writer: &mut impl io::Write) -> crate::Result<()> {
        let datatype = match self {
            Data::Reserved(_) => RESERVED,
            Data::Utf8(_) => UTF8,
            Data::Utf16(_) => UTF16,
            Data::Jpeg(_) => JPEG,
            Data::Png(_) => PNG,
            Data::Unparsed(_) => return Err(crate::Error::new(
                ErrorKind::UnWritableDataType,
                "Data of type Data::Unparsed can't be written.",
            )),
        };

        writer.write_i32::<BigEndian>(datatype)?;
        writer.write_u32::<BigEndian>(0)?;

        self.write_raw(writer)?;

        Ok(())
    }

    pub fn write_raw(&self, writer: &mut impl io::Write) -> crate::Result<()> {
        match self {
            Data::Reserved(v) => {
                writer.write(v)?;
            }
            Data::Utf8(s) => {
                writer.write(s.as_bytes())?;
            }
            Data::Utf16(s) => {
                for c in s.encode_utf16() {
                    writer.write_u16::<BigEndian>(c)?;
                }
            }
            Data::Jpeg(v) => {
                writer.write(v)?;
            }
            Data::Png(v) => {
                writer.write(v)?;
            }
            Data::Unparsed(_) => return Err(crate::Error::new(
                ErrorKind::UnWritableDataType,
                "Data of type Data::Unparsed can't be written.",
            )),
        }

        Ok(())
    }

    /// Attempts to read 8 bit unsigned integers from the reader to a vector of size length.
    pub fn read_to_u8_vec(reader: &mut impl io::Read, length: usize) -> crate::Result<Vec<u8>> {
        let mut buff = vec![0u8; length];

        if let Err(e) = reader.read_exact(&mut buff) {
            return Err(Error::from(e));
        }

        Ok(buff)
    }

    /// Attempts to read 16 bit unsigned integers from the reader to a vector of size length.
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

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Data::Reserved(v) => if let Data::Reserved(ov) = other { return v == ov; }
            Data::Utf8(s) => if let Data::Utf8(os) = other { return s == os; }
            Data::Utf16(s) => if let Data::Utf16(os) = other { return s == os; }
            Data::Jpeg(v) => if let Data::Jpeg(ov) = other { return v == ov; }
            Data::Png(v) => if let Data::Png(ov) = other { return v == ov; }
            Data::Unparsed(d) => if let Data::Unparsed(od) = other { return d == od; }
        }

        false
    }

    fn ne(&self, other: &Self) -> bool {
        match self {
            Data::Reserved(v) => if let Data::Reserved(ov) = other { return v != ov; }
            Data::Utf8(s) => if let Data::Utf8(os) = other { return s != os; }
            Data::Utf16(s) => if let Data::Utf16(os) = other { return s != os; }
            Data::Jpeg(v) => if let Data::Jpeg(ov) = other { return v != ov; }
            Data::Png(v) => if let Data::Png(ov) = other { return v != ov; }
            Data::Unparsed(d) => if let Data::Unparsed(od) = other { return d != od; }
        }

        true
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Data::Reserved(d) => write!(f, "Reserved{{ {:?} }}", d),
            Data::Utf8(d) => write!(f, "UTF8{{ {:?} }}", d),
            Data::Utf16(d) => write!(f, "UTF16{{ {:?} }}", d),
            Data::Jpeg(_) => write!(f, "JPEG"),
            Data::Png(_) => write!(f, "PNG"),
            Data::Unparsed(d) => write!(f, "Unparsed{{ {:?} }}", d),
        }
    }
}