use std::io::{self, Read, Seek, SeekFrom};

pub trait ReadUtil: Read {
    /// Attempts to read an unsigned 8 bit integer from the reader.
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Attempts to read an unsigned 16 bit big endian integer from the reader.
    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Attempts to read an unsigned 32 bit big endian integer from the reader.
    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Attempts to read an unsigned 64 bit big endian integer from the reader.
    fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Attempts to read 8 bit unsigned integers from the reader to a vector of size length.
    fn read_u8_vec(&mut self, len: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len as usize];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Attempts to read a utf-8 string from the reader.
    fn read_utf8(&mut self, len: u64) -> crate::Result<String> {
        let data = self.read_u8_vec(len)?;

        Ok(String::from_utf8(data)?)
    }

    /// Attempts to read a utf-16 string from the reader.
    fn read_utf16(&mut self, len: u64) -> crate::Result<String> {
        let mut buf = vec![0u8; len as usize];

        self.read_exact(&mut buf)?;

        let data: Vec<u16> =
            buf.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();

        Ok(String::from_utf16(&data)?)
    }
}

impl<T: Read> ReadUtil for T {}

pub trait SeekUtil: Seek {
    /// Attempts to read the remaining stream length and returns to the starting position.
    fn remaining_stream_len(&mut self) -> io::Result<u64> {
        let current_pos = self.seek(SeekFrom::Current(0))?;
        let complete_len = self.seek(SeekFrom::End(0))?;
        let len = complete_len - current_pos;

        self.seek(SeekFrom::Start(current_pos))?;

        Ok(len)
    }
}

impl<T: Seek> SeekUtil for T {}

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

#[cfg(test)]
mod test {
    #[test]
    fn be_int() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00, 0x2D, 0x34, 0xD0, 0x5E];
        let int = be_int!(bytes, 4, u32);
        assert_eq!(int, Some(758435934u32));
    }

    #[test]
    fn set_be_int() {
        let mut bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0];
        set_be_int!(bytes, 4, 524, u16);
        assert_eq!(bytes[4], 2u8);
        assert_eq!(bytes[5], 12u8);
    }
}
