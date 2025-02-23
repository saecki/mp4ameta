use std::io::{self, Read, Seek, SeekFrom, Write};
use std::time::Duration;

use crate::ErrorKind;
use crate::atom::head::Size;

pub trait ReadUtil: Read {
    /// Attempts to read an unsigned 8 bit integer from the reader.
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Attempts to read an unsigned 16 bit big endian integer from the reader.
    fn read_be_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Attempts to read an unsigned 32 bit big endian integer from the reader.
    fn read_be_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Attempts to read an unsigned 64 bit big endian integer from the reader.
    fn read_be_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Attempts to read 8 bit unsigned integers from the reader to a vector of size length.
    fn read_u8_vec(&mut self, len: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; len as usize];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Attempts to read a utf-8 string from the reader.
    fn read_utf8(&mut self, len: u64) -> crate::Result<String> {
        let data = self.read_u8_vec(len)?;

        Ok(String::from_utf8(data)?)
    }

    /// Attempts to read a big endian utf-16 string from the reader.
    fn read_be_utf16(&mut self, len: u64) -> crate::Result<String> {
        let mut buf = vec![0; len as usize];

        self.read_exact(&mut buf)?;

        let data: Vec<u16> =
            buf.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();

        Ok(String::from_utf16(&data)?)
    }

    /// Attempts to read a little endian utf-16 string from the reader.
    fn read_le_utf16(&mut self, len: u64) -> crate::Result<String> {
        let mut buf = vec![0; len as usize];

        self.read_exact(&mut buf)?;

        let data: Vec<u16> =
            buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();

        Ok(String::from_utf16(&data)?)
    }
}

impl<T: Read> ReadUtil for T {}

pub trait SeekUtil: Seek {
    fn skip(&mut self, offset: i64) -> io::Result<()> {
        self.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}

impl<T: Seek> SeekUtil for T {}

pub trait WriteUtil: Write {
    fn write_u8(&mut self, val: u8) -> io::Result<()> {
        self.write_all(&[val])
    }

    fn write_be_u16(&mut self, val: u16) -> io::Result<()> {
        self.write_all(&val.to_be_bytes())
    }

    fn write_be_u32(&mut self, val: u32) -> io::Result<()> {
        self.write_all(&val.to_be_bytes())
    }

    fn write_be_u64(&mut self, val: u64) -> io::Result<()> {
        self.write_all(&val.to_be_bytes())
    }

    fn write_utf8(&mut self, string: &str) -> io::Result<()> {
        self.write_all(string.as_bytes())
    }

    fn write_be_utf16(&mut self, string: &str) -> io::Result<()> {
        for c in string.encode_utf16() {
            self.write_be_u16(c)?;
        }
        Ok(())
    }
}

pub fn expect_size(name: &str, head_size: Size, content_size: u64) -> crate::Result<()> {
    let head_content_size = head_size.content_len();
    if content_size != head_content_size {
        return Err(crate::Error::new(
            ErrorKind::SizeMismatch,
            format!(
                "{name} size from atom head {head_content_size} differs from the content size {content_size}",
            ),
        ));
    }
    Ok(())
}

pub fn expect_min_size(name: &str, head_size: Size, min_size: u64) -> crate::Result<()> {
    let head_content_size = head_size.content_len();
    if head_content_size < min_size {
        return Err(crate::Error::new(
            ErrorKind::InvalidAtomSize,
            format!(
                "{name} size from atom head {head_content_size} is smaller than the minimum size {min_size}",
            ),
        ));
    }
    Ok(())
}

pub fn unknown_version<T>(name: &str, version: u8) -> crate::Result<T> {
    Err(crate::Error::new(
        crate::ErrorKind::UnknownVersion(version),
        format!("Unknown {name} atom version {version}"),
    ))
}

impl<T: Write> WriteUtil for T {}

pub fn scale_duration(timescale: u32, duration: u64) -> Duration {
    let secs = duration / timescale as u64;
    let nanos = (duration % timescale as u64) * 1_000_000_000 / timescale as u64;
    Duration::new(secs, nanos as u32)
}

pub fn unscale_duration(timescale: u32, duration: Duration) -> u64 {
    let secs = duration.as_secs() * timescale as u64;
    let nanos = duration.subsec_nanos() as u64 * timescale as u64 / 1_000_000_000;
    secs + nanos
}

/// Attempts to read a big endian integer at the specified index from a byte slice.
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

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, clippy::eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn be_int() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x2D, 0x34, 0xD0, 0x5E];
        let int = be_int!(bytes, 4, u32);
        assert_eq!(int, Some(758435934u32));
    }

    #[test]
    fn set_be_int() {
        let mut bytes = vec![0, 0, 0, 0, 0, 0, 0, 0];
        set_be_int!(bytes, 4, 524, u16);
        assert_eq!(bytes[4], 2);
        assert_eq!(bytes[5], 12);
    }
}
