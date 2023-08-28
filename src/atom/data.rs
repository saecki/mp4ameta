use std::fmt;

use crate::{Img, ImgBuf, ImgFmt, ImgMut, ImgRef};

use super::*;

// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) codes
/// Reserved for use where no type needs to be indicated.
const RESERVED: u32 = 0;
/// UTF-8 without any count or NULL terminator.
const UTF8: u32 = 1;
/// UTF-16 also known as UTF-16BE.
const UTF16: u32 = 2;
/// UTF-8 variant storage of a string for sorting only.
#[allow(unused)]
const UTF8_SORT: u32 = 4;
/// UTF-16 variant storage of a string for sorting only.
#[allow(unused)]
const UTF16_SORT: u32 = 5;
/// JPEG in a JFIF wrapper.
const JPEG: u32 = 13;
/// PNG in a PNG wrapper.
const PNG: u32 = 14;
/// A big-endian signed integer in 1,2,3 or 4 bytes.
const BE_SIGNED: u32 = 21;
/// A big-endian unsigned integer in 1,2,3 or 4 bytes.
#[allow(unused)]
const BE_UNSIGNED: u32 = 22;
/// A big-endian 32-bit floating point value (`IEEE754`).
#[allow(unused)]
const BE_F32: u32 = 23;
/// A big-endian 64-bit floating point value (`IEEE754`).
#[allow(unused)]
const BE_F64: u32 = 24;
/// Windows bitmap format graphics.
#[allow(unused)]
const BMP: u32 = 27;
/// QuickTime Metadata atom.
#[allow(unused)]
const QT_META: u32 = 28;
/// An 8-bit signed integer.
#[allow(unused)]
const I8: u32 = 65;
/// A big-endian 16-bit signed integer.
#[allow(unused)]
const BE_I16: u32 = 66;
/// A big-endian 32-bit signed integer.
#[allow(unused)]
const BE_I32: u32 = 67;
/// A block of data representing a two dimensional (2D) point with 32-bit big-endian floating point
/// x and y coordinates. It has the structure:<br/> `{ BE_F32 x; BE_F32 y; }`
#[allow(unused)]
const BE_POINT_F32: u32 = 70;
/// A block of data representing 2D dimensions with 32-bit big-endian floating point width and
/// height. It has the structure:<br/>
/// `{ width: BE_F32, height: BE_F32 }`
#[allow(unused)]
const BE_DIMS_F32: u32 = 71;
/// A block of data representing a 2D rectangle with 32-bit big-endian floating point x and y
/// coordinates and a 32-bit big-endian floating point width and height size. It has the
/// structure:<br/>
/// `{ x: BE_F32, y: BE_F32, width: BE_F32, height: BE_F32 }`<br/>
/// or the equivalent structure:<br/>
/// `{ origin: BE_Point_F32, size: BE_DIMS_F32 }`
#[allow(unused)]
const BE_RECT_F32: u32 = 72;
/// A big-endian 64-bit signed integer.
#[allow(unused)]
const BE_I64: u32 = 74;
/// An 8-bit unsigned integer.
#[allow(unused)]
const U8: u32 = 75;
/// A big-endian 16-bit unsigned integer.
#[allow(unused)]
const BE_U16: u32 = 76;
/// A big-endian 32-bit unsigned integer.
#[allow(unused)]
const BE_U32: u32 = 77;
/// A big-endian 64-bit unsigned integer.
#[allow(unused)]
const BE_U64: u32 = 78;
/// A block of data representing a 3x3 transformation matrix. It has the structure:<br/>
/// `{ matrix: [[BE_F64; 3]; 3] }`
#[allow(unused)]
const AFFINE_TRANSFORM_F64: u32 = 79;

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
    /// A value containing an unknown data type code and data.
    Unknown {
        /// The data type code.
        code: u32,
        /// The data.
        data: Vec<u8>,
    },
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reserved(d) => write!(f, "Data::Reserved({d:?})"),
            Self::Utf8(d) => write!(f, "Data::Utf8({d:?})"),
            Self::Utf16(d) => write!(f, "Data::Utf16({d:?})"),
            Self::Jpeg(_) => write!(f, "Data::Jpeg"),
            Self::Png(_) => write!(f, "Data::Png"),
            Self::BeSigned(d) => write!(f, "Data::BeSigned({d:?})"),
            Self::Bmp(_) => write!(f, "Data::Bmp"),
            Self::Unknown { code, data } => {
                f.debug_struct("Data::Unknown").field("code", code).field("data", data).finish()
            }
        }
    }
}

impl<T: Into<Vec<u8>>> From<Img<T>> for Data {
    fn from(image: Img<T>) -> Self {
        match image.fmt {
            ImgFmt::Bmp => Self::Bmp(image.data.into()),
            ImgFmt::Jpeg => Self::Jpeg(image.data.into()),
            ImgFmt::Png => Self::Png(image.data.into()),
        }
    }
}

impl Atom for Data {
    const FOURCC: Fourcc = DATA;
}

impl ParseAtom for Data {
    /// Parses data based on [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34).
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Data> {
        let (version, flags) = parse_full_head(reader)?;
        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Error reading data atom (data)".to_owned(),
            ));
        }
        let [b2, b1, b0] = flags;
        let datatype = u32::from_be_bytes([0, b2, b1, b0]);

        // Skipping 4 byte locale indicator
        reader.seek(SeekFrom::Current(4))?;

        let data_len = size.content_len() - 8;

        Ok(match datatype {
            RESERVED => Data::Reserved(reader.read_u8_vec(data_len)?),
            UTF8 => Data::Utf8(reader.read_utf8(data_len)?),
            UTF16 => Data::Utf16(reader.read_utf16(data_len)?),
            JPEG => Data::Jpeg(reader.read_u8_vec(data_len)?),
            PNG => Data::Png(reader.read_u8_vec(data_len)?),
            BE_SIGNED => Data::BeSigned(reader.read_u8_vec(data_len)?),
            BMP => Data::Bmp(reader.read_u8_vec(data_len)?),
            _ => {
                // TODO: maybe log warning
                Data::Unknown { code: datatype, data: reader.read_u8_vec(data_len)? }
            }
        })
    }
}

impl WriteAtom for Data {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;

        let datatype = match self {
            Self::Reserved(_) => RESERVED,
            Self::Utf8(_) => UTF8,
            Self::Utf16(_) => UTF16,
            Self::Jpeg(_) => JPEG,
            Self::Png(_) => PNG,
            Self::BeSigned(_) => BE_SIGNED,
            Self::Bmp(_) => BMP,
            Self::Unknown { code, .. } => *code,
        };

        writer.write_all(&datatype.to_be_bytes())?;
        // Writing 4 byte locale indicator
        writer.write_all(&[0; 4])?;

        match self {
            Self::Reserved(v) => writer.write_all(v)?,
            Self::Utf8(s) => writer.write_all(s.as_bytes())?,
            Self::Utf16(s) => {
                for c in s.encode_utf16() {
                    writer.write_all(&c.to_be_bytes())?;
                }
            }
            Self::Jpeg(v) => writer.write_all(v)?,
            Self::Png(v) => writer.write_all(v)?,
            Self::BeSigned(v) => writer.write_all(v)?,
            Self::Bmp(v) => writer.write_all(v)?,
            Self::Unknown { data, .. } => writer.write_all(data)?,
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + self.data_len();
        Size::from(content_len)
    }
}

impl Data {
    /// Returns the length of the raw data (without version, datatype and locale header) in bytes.
    pub fn data_len(&self) -> u64 {
        (match self {
            Self::Reserved(v) => v.len(),
            Self::Utf8(s) => s.len(),
            Self::Utf16(s) => s.encode_utf16().count(),
            Self::Jpeg(v) => v.len(),
            Self::Png(v) => v.len(),
            Self::BeSigned(v) => v.len(),
            Self::Bmp(v) => v.len(),
            Self::Unknown { data, .. } => data.len(),
        }) as u64
    }

    /// Returns true if the data is of length 0, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.data_len() == 0
    }

    /// Returns true if `self` is of type [`Self::Reserved`] or [`Self::BeSigned`], false otherwise.
    pub const fn is_bytes(&self) -> bool {
        matches!(self, Self::Reserved(_) | Self::BeSigned(_))
    }

    /// Returns true if `self` is of type [`Self::Utf8`] or [`Self::Utf16`], false otherwise.
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::Utf8(_) | Self::Utf16(_))
    }

    /// Returns true if `self` is of type [`Self::Jpeg`], [`Self::Png`] or [`Self::Bmp`] false
    /// otherwise.
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

    /// Returns a reference to byte data if `self` is of type [`Self::Reserved`] or
    /// [`Self::BeSigned`].
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a mutable reference to byte data if `self` is of type [`Self::Reserved`] or
    /// [`Self::BeSigned`].
    pub fn bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes `self` and returns byte data if `self` is of type [`Self::Reserved`] or
    /// [`Self::BeSigned`].
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self {
            Self::Reserved(v) => Some(v),
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to a string if `self` is of type [`Self::Utf8`] or [`Self::Utf16`].
    pub fn string(&self) -> Option<&str> {
        match self {
            Self::Utf8(s) => Some(s.as_str()),
            Self::Utf16(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns a mutable reference to a string if `self` is of type [`Self::Utf8`] or
    /// [`Self::Utf16`].
    pub fn string_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Utf8(s) => Some(s),
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Consumes `self` and returns a string if `self` is of type [`Self::Utf8`] or [`Self::Utf16`].
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::Utf8(s) => Some(s),
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to an image if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image(&self) -> Option<ImgRef<'_>> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Returns a mutable reference to an image if `self` is of type [`Self::Jpeg`], [`Self::Png`]
    /// or [`Self::Bmp`].
    pub fn image_mut(&mut self) -> Option<ImgMut<'_>> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Consumes `self` and returns an image if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn into_image(self) -> Option<ImgBuf> {
        match self {
            Self::Jpeg(v) => Some(Img::new(ImgFmt::Jpeg, v)),
            Self::Png(v) => Some(Img::new(ImgFmt::Png, v)),
            Self::Bmp(v) => Some(Img::new(ImgFmt::Bmp, v)),
            _ => None,
        }
    }

    /// Returns a reference to image data if `self` is of type [`Self::Jpeg`], [`Self::Png`] or
    /// [`Self::Bmp`].
    pub fn image_data(&self) -> Option<&[u8]> {
        self.image().map(|i| i.data)
    }

    /// Returns a mutable reference to image data if `self` is of type [`Self::Jpeg`], [`Self::Png`]
    /// or [`Self::Bmp`].
    pub fn image_data_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.image_mut().map(|i| i.data)
    }

    /// Consumes `self` and returns image data if `self` is of type [`Self::Jpeg`], [`Self::Png`]
    /// or [`Self::Bmp`].
    pub fn into_image_data(self) -> Option<Vec<u8>> {
        self.into_image().map(|i| i.data)
    }

    /// Returns a reference to byte data if `self` is of type [`Self::Reserved`].
    pub fn reserved(&self) -> Option<&[u8]> {
        match self {
            Self::Reserved(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to a string if `self` is of type [`Self::Utf8`].
    pub fn utf8(&self) -> Option<&str> {
        match self {
            Self::Utf8(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to a string if `self` is of type [`Self::Utf16`].
    pub fn utf16(&self) -> Option<&str> {
        match self {
            Self::Utf16(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to image data if `self` is of type [`Self::Jpeg`].
    pub fn jpeg(&self) -> Option<&[u8]> {
        match self {
            Self::Jpeg(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to image data if `self` is of type [`Self::Png`].
    pub fn png(&self) -> Option<&[u8]> {
        match self {
            Self::Png(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to byte data if `self` is of type [`Self::BeSigned`].
    pub fn be_signed(&self) -> Option<&[u8]> {
        match self {
            Self::BeSigned(v) => Some(v),
            _ => None,
        }
    }

    /// Returns a reference to byte data if `self` is of type [`Self::Bmp`].
    pub fn bmp(&self) -> Option<&[u8]> {
        match self {
            Self::Bmp(v) => Some(v),
            _ => None,
        }
    }
}
