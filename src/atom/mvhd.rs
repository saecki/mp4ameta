use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mvhd {
    pub state: State,
    pub version: u8,
    pub flags: [u8; 3],
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub preferred_rate: u32,
    pub preferred_volume: u16,
    pub matrix: [[u32; 3]; 3],
    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32,
}

impl Atom for Mvhd {
    const FOURCC: Fourcc = MOVIE_HEADER;
}

impl ParseAtom for Mvhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut mvhd = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };

        let (version, flags) = parse_full_head(reader)?;
        mvhd.version = version;
        mvhd.flags = flags;
        match version {
            0 => {
                mvhd.creation_time = reader.read_be_u32()? as u64;
                mvhd.modification_time = reader.read_be_u32()? as u64;
                mvhd.timescale = reader.read_be_u32()?;
                mvhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                mvhd.creation_time = reader.read_be_u64()?;
                mvhd.modification_time = reader.read_be_u64()?;
                mvhd.timescale = reader.read_be_u32()?;
                mvhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (mvhd) version {v}"),
                ))
            }
        }
        mvhd.preferred_rate = reader.read_be_u32()?;
        mvhd.preferred_volume = reader.read_be_u16()?;
        reader.skip(10)?; //reserved
        for row in mvhd.matrix.iter_mut() {
            for i in row.iter_mut() {
                *i = reader.read_be_u32()?;
            }
        }
        mvhd.preview_time = reader.read_be_u32()?;
        mvhd.preview_duration = reader.read_be_u32()?;
        mvhd.poster_time = reader.read_be_u32()?;
        mvhd.selection_time = reader.read_be_u32()?;
        mvhd.selection_duration = reader.read_be_u32()?;
        mvhd.current_time = reader.read_be_u32()?;
        mvhd.next_track_id = reader.read_be_u32()?;

        Ok(mvhd)
    }
}

impl WriteAtom for Mvhd {
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
                    format!("Unknown movie header (mvhd) version {v}"),
                ));
            }
        }
        writer.write_be_u32(self.preferred_rate)?;
        writer.write_be_u16(self.preferred_volume)?;
        writer.write_all(&[0; 10])?; //reserved
        for row in self.matrix {
            for i in row {
                writer.write_be_u32(i)?;
            }
        }
        writer.write_be_u32(self.preview_time)?;
        writer.write_be_u32(self.preview_duration)?;
        writer.write_be_u32(self.poster_time)?;
        writer.write_be_u32(self.selection_time)?;
        writer.write_be_u32(self.selection_duration)?;
        writer.write_be_u32(self.current_time)?;
        writer.write_be_u32(self.next_track_id)?;

        Ok(())
    }

    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(100),
            1 => Size::from(112),
            _ => Size::from(0),
        }
    }
}
