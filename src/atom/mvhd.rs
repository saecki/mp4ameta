use std::io::{Read, Seek, SeekFrom};

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mvhd {
    pub timescale: u32,
    /// The duration in timescale units
    pub duration: u64,
}

impl Atom for Mvhd {
    const FOURCC: Fourcc = MOVIE_HEADER;
}

impl ParseAtom for Mvhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut mvhd = Self::default();

        let (version, _) = parse_full_head(reader)?;
        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes modification time
                // 4 bytes time scale
                // 4 bytes duration
                // ...
                reader.seek(SeekFrom::Current(8))?;
                mvhd.timescale = reader.read_be_u32()?;
                mvhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes modification time
                // 4 bytes time scale
                // 8 bytes duration
                // ...
                reader.seek(SeekFrom::Current(16))?;
                mvhd.timescale = reader.read_be_u32()?;
                mvhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (mvhd) version {}", v),
                ))
            }
        }

        seek_to_end(reader, &bounds)?;

        Ok(mvhd)
    }
}
