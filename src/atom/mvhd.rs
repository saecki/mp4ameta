use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mvhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub timescale: u32,
    pub duration: u64,
}

impl Atom for Mvhd {
    const FOURCC: Fourcc = MOVIE_HEADER;
}

impl ParseAtom for Mvhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut mvhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        mvhd.version = version;
        mvhd.flags = flags;
        match version {
            0 => {
                reader.skip(4)?; // creation time
                reader.skip(4)?; // modification time
                mvhd.timescale = reader.read_be_u32()?;
                mvhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                reader.skip(8)?; // creation time
                reader.skip(8)?; // modification time
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
        reader.skip(4)?; // preferred rate
        reader.skip(2)?; // preferred volume
        reader.skip(10)?; // reserved
        reader.skip(4 * 9)?; // matrix
        reader.skip(4)?; // preview time
        reader.skip(4)?; // preview duration
        reader.skip(4)?; // poster time
        reader.skip(4)?; // selection time
        reader.skip(4)?; // selection duration
        reader.skip(4)?; // current time
        reader.skip(4)?; // next track id

        Ok(mvhd)
    }
}

impl AtomSize for Mvhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(100),
            1 => Size::from(112),
            _ => Size::from(0),
        }
    }
}
