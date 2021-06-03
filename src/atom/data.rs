use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::{Img, ImgBuf, ImgFmt, ImgMut, ImgRef};

// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) codes
/// Reserved for use where no type needs to be indicated.
pub(crate) const RESERVED: u32 = 0;
/// UTF-8 without any count or NULL terminator.
pub(crate) const UTF8: u32 = 1;
/// UTF-16 also known as UTF-16BE.
pub(crate) const UTF16: u32 = 2;
/// UTF-8 variant storage of a string for sorting only.
#[allow(unused)]
pub(crate) const UTF8_SORT: u32 = 4;
/// UTF-16 variant storage of a string for sorting only.
#[allow(unused)]
pub(crate) const UTF16_SORT: u32 = 5;
/// JPEG in a JFIF wrapper.
pub(crate) const JPEG: u32 = 13;
/// PNG in a PNG wrapper.
pub(crate) const PNG: u32 = 14;
/// A big-endian signed integer in 1,2,3 or 4 bytes.
pub(crate) const BE_SIGNED: u32 = 21;
/// A big-endian unsigned integer in 1,2,3 or 4 bytes.
#[allow(unused)]
pub(crate) const BE_UNSIGNED: u32 = 22;
/// A big-endian 32-bit floating point value (`IEEE754`).
#[allow(unused)]
pub(crate) const BE_F32: u32 = 23;
/// A big-endian 64-bit floating point value (`IEEE754`).
#[allow(unused)]
pub(crate) const BE_F64: u32 = 24;
/// Windows bitmap format graphics.
#[allow(unused)]
pub(crate) const BMP: u32 = 27;
/// QuickTime Metadata atom.
#[allow(unused)]
pub(crate) const QT_META: u32 = 28;
/// An 8-bit signed integer.
#[allow(unused)]
pub(crate) const I8: u32 = 65;
/// A big-endian 16-bit signed integer.
#[allow(unused)]
pub(crate) const BE_I16: u32 = 66;
/// A big-endian 32-bit signed integer.
#[allow(unused)]
pub(crate) const BE_I32: u32 = 67;
/// A block of data representing a two dimensional (2D) point with 32-bit big-endian floating point
/// x and y coordinates. It has the structure:<br/> `{ BE_F32 x; BE_F32 y; }`
#[allow(unused)]
pub(crate) const BE_POINT_F32: u32 = 70;
/// A block of data representing 2D dimensions with 32-bit big-endian floating point width and
/// height. It has the structure:<br/>
/// `{ width: BE_F32, height: BE_F32 }`
#[allow(unused)]
pub(crate) const BE_DIMS_F32: u32 = 71;
/// A block of data representing a 2D rectangle with 32-bit big-endian floating point x and y
/// coordinates and a 32-bit big-endian floating point width and height size. It has the
/// structure:<br/>
/// `{ x: BE_F32, y: BE_F32, width: BE_F32, height: BE_F32 }`<br/>
/// or the equivalent structure:<br/>
/// `{ origin: BE_Point_F32, size: BE_DIMS_F32 }`
#[allow(unused)]
pub(crate) const BE_RECT_F32: u32 = 72;
/// A big-endian 64-bit signed integer.
#[allow(unused)]
pub(crate) const BE_I64: u32 = 74;
/// An 8-bit unsigned integer.
#[allow(unused)]
pub(crate) const U8: u32 = 75;
/// A big-endian 16-bit unsigned integer.
#[allow(unused)]
pub(crate) const BE_U16: u32 = 76;
/// A big-endian 32-bit unsigned integer.
#[allow(unused)]
pub(crate) const BE_U32: u32 = 77;
/// A big-endian 64-bit unsigned integer.
#[allow(unused)]
pub(crate) const BE_U64: u32 = 78;
/// A block of data representing a 3x3 transformation matrix. It has the structure:<br/>
/// `{ matrix: [[BE_F64; 3]; 3] }`
#[allow(unused)]
pub(crate) const AFFINE_TRANSFORM_F64: u32 = 79;

/// An enum that holds different types of data defined by
/// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34).
#[derive(Clone, Eq, PartialEq)]
pub enum Data {
    /// A value containing reserved type data inside a `Vec<u8>`.
    Reserved(Vec<u8>),
    /// A value containing a `String` decoded from, or to be encoded to utf-8.
    Utf8(String),
    /// A value containing a `String` decoded from, or to be encoded to utf-16.
    Utf16(String),
    /// A value containing jpeg byte data inside a `Vec<u8>`.
    Jpeg(Vec<u8>),
    /// A value containing png byte data inside a `Vec<u8>`.
    Png(Vec<u8>),
    /// A value containing big endian signed integer inside a `Vec<u8>`.
    BeSigned(Vec<u8>),
    /// A value containing bmp byte data inside a `Vec<u8>`.
    Bmp(Vec<u8>),
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reserved(d) => write!(f, "Data::Reserved({:?})", d),
            Self::Utf8(d) => write!(f, "Data::Utf8({:?})", d),
            Self::Utf16(d) => write!(f, "Data::Utf16({:?})", d),
            Self::Jpeg(_) => write!(f, "Data::Jpeg"),
            Self::Png(_) => write!(f, "Data::Png"),
            Self::BeSigned(d) => write!(f, "Data::BeSigned({:?})", d),
            Self::Bmp(_) => write!(f, "Data::Bmp"),
        }
    }
}

impl From<ImgBuf> for Data {
    fn from(image: ImgBuf) -> Self {
        match image.fmt {
            ImgFmt::Bmp => Self::Bmp(image.data),
            ImgFmt::Jpeg => Self::Jpeg(image.data),
            ImgFmt::Png => Self::Png(image.data),
        }
    }
}

impl Data {
    /// Returns the length in bytes.
    pub fn len(&self) -> u64 {
        (match self {
            Self::Reserved(v) => v.len(),
            Self::Utf8(s) => s.len(),
            Self::Utf16(s) => s.encode_utf16().count(),
            Self::Jpeg(v) => v.len(),
            Self::Png(v) => v.len(),
            Self::BeSigned(v) => v.len(),
            Self::Bmp(v) => v.len(),
        }) as u64
    }

    /// Returns true if the data is empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if `self` is of type [`Self::Reserved`] or [`Self::BeSigned`], false otherwise.
    pub const fn is_bytes(&self) -> bool {
        matches!(self, Self::Reserved(_) | Self::BeSigned(_))
    }

    /// Returns true if `self` is of type [`Self::Utf8`] or [`Self::Utf16`], false otherwise.
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::Utf8(_) | Self::Utf16(_))
    }

    /// Returns true if `self` is of type [`Self::Jpeg`], [`Self::Png`] or [`Self::Bmp`] false otherwise.
    pub const fn is_image(&self) -> bool {
        matches!(self, Self::Jpeg(_) | Self::Png(_) | Self::Bmp(_))
    }

    /// Returns true if `self` is of type [`Self::Reserved`] false otherwise.
    pub const fn is_reserved(&self) -> bool {
        matches!(self, Self::Reserved(_))
    }

    /// Returns true if `self` is of type [`Self::Utf8`] false otherwise.
    pub const fn is_utf8(&self) -> bool {
        matches!(self, Self::Utf8(_))
    }

    /// Returns true if `self` is of type [`Self::Utf16`] false otherwise.
    pub const fn is_utf16(&self) -> bool {
        matches!(self, Self::Utf16(_))
    }

    /// Returns true if `self` is of type [`Self::Jpeg`] false otherwise.
    pub const fn is_jpeg(&self) -> bool {
        matches!(self, Self::Jpeg(_))
    }

    /// Returns true if `self` is of type [`Self::Png`] false otherwise.
    pub const fn is_png(&self) -> bool {
        matches!(self, Self::Png(_))
    }

    /// Returns true if `self` is of type [`Self::BeSigned`] false otherwise.
    pub const fn is_be_signed(&self) -> bool {
        matches!(self, Self::BeSigned(_))
    }

    /// Returns true if `self` is of type [`Self::Bmp`] false otherwise.
    pub const fn is_bmp(&self) -> bool {
        matches!(self, Self::Bmp(_))
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Reserved`] or [`Self::BeSigned`].
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a mutable byte vec reference if `self` is of type [`Self::Reserved`] or
    /// [`Self::BeSigned`].
    pub fn bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes `self` and returns a byte vec if `self` is of type [`Self::Reserved`] or
    /// [`Self::BeSigned`].
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a string reference if `self` is either of type [`Self::Utf8`] or [`Self::Utf16`].
    pub fn string(&self) -> Option<&str> {
        match self {
            Self::Utf8(s) => Some(s.as_str()),
            Self::Utf16(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns a mutable string reference if `self` is either of type [`Self::Utf8`] or
    /// [`Self::Utf16`].
    pub fn string_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Utf8(s) => Some(s),
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Consumes `self` and returns a string if `self` is either of type [`Self::Utf8`] or
    /// [`Self::Utf16`].
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::Utf8(s) => Some(s),
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a data reference if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image(&self) -> Option<ImgRef> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Returns a data reference if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image_mut(&mut self) -> Option<ImgMut> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Consumes `self` and returns data if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn into_image(self) -> Option<ImgBuf> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image_data(&self) -> Option<&[u8]> {
        self.image().map(|i| i.data)
    }

    /// Returns a mutable byte vec reference if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image_data_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.image_mut().map(|i| i.data)
    }

    /// Consumes `self` and returns a byte vec if `self` is of type [`Self::Jpeg`], [`Self::Png`]
    /// or [`Self::Bmp`].
    pub fn into_image_data(self) -> Option<Vec<u8>> {
        self.into_image().map(|i| i.data)
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Reserved`].
    pub const fn reserved(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Reserved(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a string reference if `self` is of type [`Self::Utf8`].
    pub const fn utf8(&self) -> Option<&String> {
        match self {
            Self::Utf8(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a string reference if `self` is of type [`Self::Utf16`].
    pub const fn utf16(&self) -> Option<&String> {
        match self {
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Jpeg`].
    pub const fn jpeg(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Jpeg(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Png`].
    pub const fn png(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Png(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a byte vec reference if `self` is of type [`Self::BeSigned`].
    pub const fn be_signed(&self) -> Option<&Vec<u8>> {
        match self {
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a byte vec reference if `self` is of type [`Self::Bmp`].
    pub const fn bmp(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Bmp(v) => Some(v),
            _ => None,
        }
    }

    /// Attempts to write the typed data to the writer.
    pub fn write_typed(&self, writer: &mut impl Write) -> crate::Result<()> {
        let datatype = match self {
            Self::Reserved(_) => RESERVED,
            Self::Utf8(_) => UTF8,
            Self::Utf16(_) => UTF16,
            Self::Jpeg(_) => JPEG,
            Self::Png(_) => PNG,
            Self::BeSigned(_) => BE_SIGNED,
            Self::Bmp(_) => BMP,
        };

        writer.write_all(&datatype.to_be_bytes())?;
        // Writing 4 byte locale indicator
        writer.write_all(&[0u8; 4])?;

        self.write_raw(writer)?;

        Ok(())
    }

    /// Attempts to write the raw data to the writer.
    pub fn write_raw(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self {
            Self::Reserved(v) => {
                writer.write_all(v)?;
            }
            Self::Utf8(s) => {
                writer.write_all(s.as_bytes())?;
            }
            Self::Utf16(s) => {
                for c in s.encode_utf16() {
                    writer.write_all(&c.to_be_bytes())?;
                }
            }
            Self::Jpeg(v) => {
                writer.write_all(v)?;
            }
            Self::Png(v) => {
                writer.write_all(v)?;
            }
            Self::BeSigned(v) => {
                writer.write_all(v)?;
            }
            Self::Bmp(v) => {
                writer.write_all(v)?;
            }
        }

        Ok(())
    }
}

/// Parses data based on [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34).
pub(crate) fn parse_data(reader: &mut impl Read, datatype: u32, len: u64) -> crate::Result<Data> {
    Ok(match datatype {
        RESERVED => Data::Reserved(read_u8_vec(reader, len)?),
        UTF8 => Data::Utf8(read_utf8(reader, len)?),
        UTF16 => Data::Utf16(read_utf16(reader, len)?),
        JPEG => Data::Jpeg(read_u8_vec(reader, len)?),
        PNG => Data::Png(read_u8_vec(reader, len)?),
        BE_SIGNED => Data::BeSigned(read_u8_vec(reader, len)?),
        BMP => Data::Bmp(read_u8_vec(reader, len)?),
        _ => {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownDataType(datatype),
                "Unknown datatype code".to_owned(),
            ));
        }
    })
}

/// Attempts to read an unsigned 8 bit integer from the reader.
pub(crate) fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0u8];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Attempts to read an unsigned 16 bit big endian integer from the reader.
pub(crate) fn read_u16(reader: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

/// Attempts to read an unsigned 32 bit big endian integer from the reader.
pub(crate) fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Attempts to read an unsigned 64 bit big endian integer from the reader.
pub(crate) fn read_u64(reader: &mut impl Read) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

/// Attempts to read 8 bit unsigned integers from the reader to a vector of size length.
pub(crate) fn read_u8_vec(reader: &mut impl Read, len: u64) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Attempts to read a utf-8 string from the reader.
pub(crate) fn read_utf8(reader: &mut impl Read, len: u64) -> crate::Result<String> {
    let data = read_u8_vec(reader, len)?;

    Ok(String::from_utf8(data)?)
}

/// Attempts to read a utf-16 string from the reader.
pub(crate) fn read_utf16(reader: &mut impl Read, len: u64) -> crate::Result<String> {
    let mut buf = vec![0u8; len as usize];

    reader.read_exact(&mut buf)?;

    let data: Vec<u16> = buf.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();

    Ok(String::from_utf16(&data)?)
}

/// Attempts to read the remaining stream length and returns to the starting position.
pub(crate) fn remaining_stream_len(reader: &mut impl Seek) -> io::Result<u64> {
    let current_pos = reader.seek(SeekFrom::Current(0))?;
    let complete_len = reader.seek(SeekFrom::End(0))?;
    let len = complete_len - current_pos;

    reader.seek(SeekFrom::Start(current_pos))?;

    Ok(len)
}

/// Attempts to read a big endian integer at the specified index from a byte slice.
///
/// # Example
/// ```
/// # #[macro_use] extern crate mp4ameta; fn main() {
/// let bytes = vec![0u8, 0, 0, 0, 0, 0, 1, 3];
/// let int = be_int!(bytes, 4, u32);
/// assert_eq!(int, Some(259u32));
/// # }
/// ```
#[macro_export]
macro_rules! be_int {
    ($bytes:expr, $index:expr, $type:ty) => {{
        use std::convert::TryFrom;

        const SIZE: usize = std::mem::size_of::<$type>();
        let bytes_start = ($index);
        let bytes_end = ($index) + SIZE;

        if $bytes.len() < bytes_end {
            None
        } else {
            let be_bytes = <[u8; SIZE]>::try_from(&$bytes[bytes_start..bytes_end]);

            match be_bytes {
                Ok(b) => Some(<$type>::from_be_bytes(b)),
                Err(_) => None,
            }
        }
    }};
}

/// Attempts to write a big endian integer at the specified index to a byte vector.
///
/// # Example
/// ```
/// # #[macro_use] extern crate mp4ameta; fn main() {
/// let mut bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0];
/// set_be_int!(bytes, 4, 524, u16);
/// assert_eq!(bytes[4], 2u8);
/// assert_eq!(bytes[5], 12u8);
/// # }
/// ```
#[macro_export]
macro_rules! set_be_int {
    ($bytes:expr, $index:expr, $value:expr, $type:ty) => {{
        const SIZE: usize = std::mem::size_of::<$type>();
        let bytes_start = ($index);
        let bytes_end = ($index) + SIZE;

        let be_bytes = <$type>::to_be_bytes($value);

        if $bytes.len() < bytes_end {
            $bytes.resize(bytes_end, 0);
        }

        for i in 0..SIZE {
            $bytes[bytes_start + i] = be_bytes[i];
        }
    }};
}
