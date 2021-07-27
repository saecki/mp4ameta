use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub language: u16,
    pub quality: u16,
}

impl Atom for Mdhd {
    const FOURCC: Fourcc = MEDIA_HEADER;
}

impl ParseAtom for Mdhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut mdhd = Self::default();

        let (version, flags) = parse_full_head(reader)?;
        mdhd.version = version;
        mdhd.flags = flags;
        match version {
            0 => {
                mdhd.creation_time = reader.read_be_u32()? as u64;
                mdhd.modification_time = reader.read_be_u32()? as u64;
                mdhd.timescale = reader.read_be_u32()?;
                mdhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                mdhd.creation_time = reader.read_be_u64()?;
                mdhd.modification_time = reader.read_be_u64()?;
                mdhd.timescale = reader.read_be_u32()?;
                mdhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown media header (mdhd) version {}", v),
                ));
            }
        }
        mdhd.language = reader.read_be_u16()?;
        mdhd.quality = reader.read_be_u16()?;

        Ok(mdhd)
    }
}

impl WriteAtom for Mdhd {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                writer.write_be_u32(self.creation_time as u32)?;
                writer.write_be_u32(self.modification_time as u32)?;
                writer.write_be_u32(self.timescale)?;
                writer.write_be_u32(self.duration as u32)?;
            }
            1 => {
                writer.write_be_u64(self.creation_time)?;
                writer.write_be_u64(self.modification_time)?;
                writer.write_be_u32(self.timescale)?;
                writer.write_be_u64(self.duration)?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown media header (mdhd) version {}", v),
                ));
            }
        }
        writer.write_be_u16(self.language)?;
        writer.write_be_u16(self.quality)?;

        Ok(())
    }

    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(24),
            1 => Size::from(36),
            _ => Size::from(0),
        }
    }
}
