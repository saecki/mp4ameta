use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mvhd {
    /// The duration of the track.
    pub duration: Duration,
}

impl Atom for Mvhd {
    const FOURCC: Fourcc = MOVIE_HEADER;
}

impl ParseAtom for Mvhd {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut mvhd = Self::default();

        let (version, _) = parse_full_head(reader)?;
        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes motification time
                // 4 bytes time scale
                // 4 bytes duration
                // ...
                reader.seek(SeekFrom::Current(8))?;
                let timescale = reader.read_u32()? as u64;
                let duration = reader.read_u32()? as u64;

                mvhd.duration = Duration::from_nanos(duration * 1_000_000_000 / timescale);
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes motification time
                // 4 bytes time scale
                // 8 bytes duration
                // ...
                reader.seek(SeekFrom::Current(16))?;
                let timescale = reader.read_u32()? as u64;
                let duration = reader.read_u64()?;

                mvhd.duration = Duration::from_nanos(duration * 1_000_000_000 / timescale);
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (mvhd) version {v}"),
                ))
            }
        }

        seek_to_end(reader, &bounds)?;

        Ok(mvhd)
    }
}
