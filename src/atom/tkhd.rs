use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tkhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub creation_time: u64,
    pub modification_time: u64,
    pub id: u32,
    /// The duration in mvhd timescale units
    pub duration: u64,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: u16,
    pub matrix: [[u32; 3]; 3],
    pub track_width: u32,
    pub track_height: u32,
}

impl Atom for Tkhd {
    const FOURCC: Fourcc = TRACK_HEADER;
}

impl ParseAtom for Tkhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut tkhd = Self::default();

        let (version, flags) = parse_full_head(reader)?;
        tkhd.version = version;
        tkhd.flags = flags;
        match version {
            0 => {
                tkhd.creation_time = reader.read_be_u32()? as u64;
                tkhd.modification_time = reader.read_be_u32()? as u64;
                tkhd.id = reader.read_be_u32()?;
                reader.skip(4)?; // reserved
                tkhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                tkhd.creation_time = reader.read_be_u64()?;
                tkhd.modification_time = reader.read_be_u64()?;
                tkhd.id = reader.read_be_u32()?;
                reader.skip(4)?; // reserved
                tkhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown track header (tkhd) version {}", v),
                ));
            }
        }
        reader.skip(8)?; // reserved
        tkhd.layer = reader.read_be_u16()?;
        tkhd.alternate_group = reader.read_be_u16()?;
        tkhd.volume = reader.read_be_u16()?;
        reader.skip(2)?; // reserved
        for i in 0..3 {
            tkhd.matrix[i][0] = reader.read_be_u32()?;
            tkhd.matrix[i][1] = reader.read_be_u32()?;
            tkhd.matrix[i][2] = reader.read_be_u32()?;
        }
        tkhd.track_width = reader.read_be_u32()?;
        tkhd.track_height = reader.read_be_u32()?;

        Ok(tkhd)
    }
}

impl WriteAtom for Tkhd {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                writer.write_be_u32(self.creation_time as u32)?;
                writer.write_be_u32(self.modification_time as u32)?;
                writer.write_be_u32(self.id)?;
                writer.write_all(&[0; 4])?; // reserved
                writer.write_be_u32(self.duration as u32)?;
            }
            1 => {
                writer.write_be_u64(self.creation_time)?;
                writer.write_be_u64(self.modification_time)?;
                writer.write_be_u32(self.id)?;
                writer.write_all(&[0; 4])?; // reserved
                writer.write_be_u64(self.duration)?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown track header (tkhd) version {}", v),
                ));
            }
        }
        writer.write_all(&[0; 8])?; // reserved
        writer.write_be_u16(self.layer)?;
        writer.write_be_u16(self.alternate_group)?;
        writer.write_be_u16(self.volume)?;
        writer.write_all(&[0; 2])?; // reserved
        for i in 0..3 {
            writer.write_be_u32(self.matrix[i][0])?;
            writer.write_be_u32(self.matrix[i][1])?;
            writer.write_be_u32(self.matrix[i][2])?;
        }
        writer.write_be_u32(self.track_width)?;
        writer.write_be_u32(self.track_height)?;

        Ok(())
    }

    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(84),
            1 => Size::from(96),
            _ => Size::from(0),
        }
    }
}
