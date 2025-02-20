use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub timescale: u32,
    pub duration: u64,
}

impl Atom for Mdhd {
    const FOURCC: Fourcc = MEDIA_HEADER;
}

impl ParseAtom for Mdhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut mdhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        mdhd.version = version;
        mdhd.flags = flags;
        match version {
            0 => {
                reader.skip(4)?; // creation time
                reader.skip(4)?; // modification time
                mdhd.timescale = reader.read_be_u32()?;
                mdhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                reader.skip(8)?; // creation time
                reader.skip(8)?; // modification time
                mdhd.timescale = reader.read_be_u32()?;
                mdhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown media header (mdhd) version {v}"),
                ));
            }
        }
        reader.skip(2)?; // language
        reader.skip(2)?; // quality

        Ok(mdhd)
    }
}

impl AtomSize for Mdhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(24),
            1 => Size::from(36),
            _ => Size::from(0),
        }
    }
}

impl WriteAtom for Mdhd {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                writer.write_be_u32(0)?; // creation time
                writer.write_be_u32(0)?; // modification time
                writer.write_be_u32(self.timescale)?;
                writer.write_be_u32(self.duration as u32)?;
            }
            1 => {
                writer.write_be_u64(0)?; // creation time
                writer.write_be_u64(0)?; // modification time
                writer.write_be_u32(self.timescale)?;
                writer.write_be_u64(self.duration)?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown media header (mdhd) version {v}"),
                ));
            }
        }
        writer.write_be_u16(0)?; // language
        writer.write_be_u16(0)?; // quality

        Ok(())
    }
}
