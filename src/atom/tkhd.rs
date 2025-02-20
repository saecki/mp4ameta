use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tkhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub id: u32,
    /// The duration in mvhd timescale units
    pub duration: u64,
}

impl Atom for Tkhd {
    const FOURCC: Fourcc = TRACK_HEADER;
}

impl ParseAtom for Tkhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut tkhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        tkhd.version = version;
        tkhd.flags = flags;
        match version {
            0 => {
                reader.skip(4)?; // creation time
                reader.skip(4)?; // modification time
                tkhd.id = reader.read_be_u32()?;
                reader.skip(4)?; // reserved
                tkhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                reader.skip(8)?; // creation time
                reader.skip(8)?; // modification time
                tkhd.id = reader.read_be_u32()?;
                reader.skip(4)?; // reserved
                tkhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown track header (tkhd) version {v}"),
                ));
            }
        }
        reader.skip(8)?; // reserved
        reader.skip(2)?; // layer
        reader.skip(2)?; // alternate group
        reader.skip(2)?; // volume
        reader.skip(2)?; // reserved
        reader.skip(4 * 9)?; // matrix
        reader.skip(4)?; // track width
        reader.skip(4)?; // track width

        Ok(tkhd)
    }
}

impl AtomSize for Tkhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(84),
            1 => Size::from(96),
            _ => Size::from(0),
        }
    }
}

impl WriteAtom for Tkhd {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                writer.write_be_u32(0)?; // creation time
                writer.write_be_u32(0)?; // modification time
                writer.write_be_u32(self.id)?;
                writer.write_all(&[0; 4])?; // reserved
                writer.write_be_u32(self.duration as u32)?;
            }
            1 => {
                writer.write_be_u64(0)?; // creation time
                writer.write_be_u64(0)?; // modification time
                writer.write_be_u32(self.id)?;
                writer.write_all(&[0; 4])?; // reserved
                writer.write_be_u64(self.duration)?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown track header (tkhd) version {v}"),
                ));
            }
        }
        writer.write_all(&[0; 8])?; // reserved
        writer.write_be_u16(0)?; // layer
        writer.write_be_u16(0)?; // alternate group
        writer.write_be_u16(0)?; // volume
        writer.write_all(&[0; 2])?; // reserved
        writer.write_all(&[0; 4 * 9])?; // matrix
        writer.write_be_u32(0)?; // track width
        writer.write_be_u32(0)?; // track height

        Ok(())
    }
}
