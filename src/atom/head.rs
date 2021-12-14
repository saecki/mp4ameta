use super::*;

/// A struct storing size of an atom and whether it is extended.
///
/// ```md
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional extended length
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    /// Whether the head is of standard size (8 bytes) with a 32 bit length or extended (16 bytes)
    /// with a 64 bit length.
    ext: bool,
    /// The length including this head.
    len: u64,
}

impl Size {
    pub const fn from(content_len: u64) -> Self {
        let mut len = content_len + 8;
        let ext = len > u32::MAX as u64;
        if ext {
            len += 8;
        }
        Self { ext, len }
    }

    pub const fn ext(&self) -> bool {
        self.ext
    }

    pub const fn len(&self) -> u64 {
        self.len
    }

    pub const fn head_len(&self) -> u64 {
        match self.ext {
            true => 16,
            false => 8,
        }
    }

    pub const fn content_len(&self) -> u64 {
        match self.ext {
            true => self.len - 16,
            false => self.len - 8,
        }
    }
}

/// A head specifying the size and type of an atom.
///
/// ```md
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional extended length
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Head {
    size: Size,
    /// The identifier.
    fourcc: Fourcc,
}

impl Deref for Head {
    type Target = Size;

    fn deref(&self) -> &Self::Target {
        &self.size
    }
}

impl Head {
    pub const fn new(ext: bool, len: u64, fourcc: Fourcc) -> Self {
        Self { size: Size { ext, len }, fourcc }
    }

    pub const fn from(size: Size, fourcc: Fourcc) -> Self {
        Self { size, fourcc }
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub const fn fourcc(&self) -> Fourcc {
        self.fourcc
    }
}

/// Attempts to parse the atom's head containing a 32 bit unsigned integer determining the size of
/// the atom in bytes and the following 4 byte identifier from the reader. If the 32 bit length is
/// set to 1 an extended 64 bit length is read.
///
/// ```md
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional extended length
/// ```
pub fn parse_head(reader: &mut impl Read) -> crate::Result<Head> {
    let len = match reader.read_u32() {
        Ok(l) => l as u64,
        Err(e) => {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length".to_owned(),
            ));
        }
    };
    let mut ident = Fourcc([0u8; 4]);
    if let Err(e) = reader.read_exact(&mut *ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".to_owned(),
        ));
    }

    if len == 1 {
        match reader.read_u64() {
            Ok(l) => Ok(Head::new(true, l, ident)),
            Err(e) => Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading extended atom length".to_owned(),
            )),
        }
    } else if len < 8 {
        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            format!("Read length of '{}' which is less than 8 bytes: {}", ident, len),
        ))
    } else {
        Ok(Head::new(false, len, ident))
    }
}

pub fn write_head(writer: &mut impl Write, head: Head) -> crate::Result<()> {
    if head.ext {
        writer.write_all(&u32::to_be_bytes(1))?;
        writer.write_all(&*head.fourcc)?;
        writer.write_all(&u64::to_be_bytes(head.len()))?;
    } else {
        writer.write_all(&u32::to_be_bytes(head.len() as u32))?;
        writer.write_all(&*head.fourcc)?;
    }
    Ok(())
}

/// Attempts to parse a full atom head.
///
/// ```md
/// 1 byte version
/// 3 bytes flags
/// ```
pub fn parse_full_head(reader: &mut impl Read) -> crate::Result<(u8, [u8; 3])> {
    let version = match reader.read_u8() {
        Ok(v) => v,
        Err(e) => {
            return Err(crate::Error::new(
                crate::ErrorKind::Io(e),
                "Error reading version of full atom head".to_owned(),
            ));
        }
    };

    let mut flags = [0u8; 3];
    if let Err(e) = reader.read_exact(&mut flags) {
        return Err(crate::Error::new(
            crate::ErrorKind::Io(e),
            "Error reading flags of full atom head".to_owned(),
        ));
    };

    Ok((version, flags))
}

pub fn write_full_head(writer: &mut impl Write, version: u8, flags: [u8; 3]) -> crate::Result<()> {
    writer.write_all(&[version])?;
    writer.write_all(&flags)?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomBounds {
    pos: u64,
    size: Size,
}

impl Deref for AtomBounds {
    type Target = Size;

    fn deref(&self) -> &Self::Target {
        &self.size
    }
}

impl AtomBounds {
    pub const fn pos(&self) -> u64 {
        self.pos
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn content_pos(&self) -> u64 {
        self.pos + self.head_len()
    }

    pub fn end(&self) -> u64 {
        self.pos + self.len()
    }
}

pub fn find_bounds(reader: &mut impl Seek, size: Size) -> crate::Result<AtomBounds> {
    let pos = reader.seek(SeekFrom::Current(0))? - size.head_len();
    Ok(AtomBounds { pos, size })
}

pub fn seek_to_end(reader: &mut impl Seek, bounds: &AtomBounds) -> crate::Result<()> {
    let current = reader.seek(SeekFrom::Current(0))?;
    let diff = bounds.end() - current;
    reader.seek(SeekFrom::Current(diff as i64))?;
    Ok(())
}
