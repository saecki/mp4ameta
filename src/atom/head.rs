use super::*;

/// A struct storing size of an atom and whether it is extended.
///
/// ```md
/// 4 bytes standard length
/// 4 bytes identifier
/// 8 bytes optional extended length
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size {
    ext: bool,
    len: u64,
}

impl Size {
    pub const fn new(ext: bool, content_len: u64) -> Self {
        let len = if ext { content_len + Head::EXT_SIZE } else { content_len + Head::NORMAL_SIZE };
        Self { ext, len }
    }

    pub const fn from(content_len: u64) -> Self {
        let ext = content_len + Head::NORMAL_SIZE > u32::MAX as u64;
        Self::new(ext, content_len)
    }

    /// Whether the head is of standard size (8 bytes) with a 32 bit length or extended (16 bytes)
    /// with a 64 bit length.
    pub const fn ext(&self) -> bool {
        self.ext
    }

    /// The length including the atom's head.
    pub const fn len(&self) -> u64 {
        self.len
    }

    /// The length of the atom's head.
    pub const fn head_len(&self) -> u64 {
        match self.ext {
            true => Head::EXT_SIZE,
            false => Head::NORMAL_SIZE,
        }
    }

    /// The length excluding the atom's head.
    pub const fn content_len(&self) -> u64 {
        self.len - self.head_len()
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
    fourcc: Fourcc,
}

impl Deref for Head {
    type Target = Size;

    fn deref(&self) -> &Self::Target {
        &self.size
    }
}

impl Head {
    pub const NORMAL_SIZE: u64 = 8;
    pub const EXT_SIZE: u64 = 16;

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
pub fn parse(reader: &mut impl Read) -> crate::Result<Head> {
    let len = match reader.read_be_u32() {
        Ok(l) => l as u64,
        Err(e) => {
            return Err(crate::Error::new(ErrorKind::Io(e), "Error reading atom length"));
        }
    };
    let mut fourcc = Fourcc::default();
    if let Err(e) = reader.read_exact(&mut *fourcc) {
        return Err(crate::Error::new(ErrorKind::Io(e), "Error reading atom identifier"));
    }

    if len == 1 {
        match reader.read_be_u64() {
            Ok(ext_len) if ext_len < 16 => Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                format!(
                    "Read extended length of '{fourcc}' which is less than 16 bytes: {ext_len}"
                ),
            )),
            Ok(ext_len) => Ok(Head::new(true, ext_len, fourcc)),
            Err(e) => {
                Err(crate::Error::new(ErrorKind::Io(e), "Error reading extended atom length"))
            }
        }
    } else if len < 8 {
        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            format!("Read length of '{fourcc}' which is less than 8 bytes: {len}"),
        ))
    } else {
        Ok(Head::new(false, len, fourcc))
    }
}

pub fn write(writer: &mut impl Write, head: Head) -> crate::Result<()> {
    if head.ext {
        writer.write_be_u32(1)?;
        writer.write_all(&*head.fourcc)?;
        writer.write_be_u64(head.len())?;
    } else {
        writer.write_be_u32(head.len() as u32)?;
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
pub fn parse_full(reader: &mut impl Read) -> crate::Result<(u8, [u8; 3])> {
    let version = match reader.read_u8() {
        Ok(v) => v,
        Err(e) => {
            return Err(crate::Error::new(
                crate::ErrorKind::Io(e),
                "Error reading version of full atom head",
            ));
        }
    };

    let mut flags = [0; 3];
    if let Err(e) = reader.read_exact(&mut flags) {
        return Err(crate::Error::new(
            crate::ErrorKind::Io(e),
            "Error reading flags of full atom head",
        ));
    };

    Ok((version, flags))
}

pub fn write_full(writer: &mut impl Write, version: u8, flags: [u8; 3]) -> crate::Result<()> {
    writer.write_all(&[version])?;
    writer.write_all(&flags)?;
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

    pub fn content_pos(&self) -> u64 {
        self.pos + self.head_len()
    }

    pub fn end(&self) -> u64 {
        self.pos + self.len()
    }
}

pub fn find_bounds(reader: &mut impl Seek, size: Size) -> crate::Result<AtomBounds> {
    let pos = reader.stream_position()? - size.head_len();
    Ok(AtomBounds { pos, size })
}

pub fn seek_to_end(reader: &mut impl Seek, bounds: &AtomBounds) -> crate::Result<()> {
    let current = reader.stream_position()?;
    let diff = bounds.end() - current;
    reader.skip(diff as i64)?;
    Ok(())
}
