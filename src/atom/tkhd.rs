use std::io::{Read, Seek, SeekFrom};

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tkhd {
    /// The duration of the track.
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
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut tkhd = Self::default();

        let (version, _) = parse_full_head(reader)?;
        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes modification time
                // 4 bytes track id
                // 4 bytes reserved
                // 4 bytes duration
                // ...
                reader.seek(SeekFrom::Current(8))?;
                tkhd.id = reader.read_be_u32()?;
                reader.seek(SeekFrom::Current(4))?;
                tkhd.duration = reader.read_be_u32()? as u64;
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes modification time
                // 4 bytes track id
                // 4 bytes reserved
                // 8 bytes duration
                // ...
                reader.seek(SeekFrom::Current(16))?;
                tkhd.id = reader.read_be_u32()?;
                reader.seek(SeekFrom::Current(4))?;
                tkhd.duration = reader.read_be_u64()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (tkhd) version {}", v),
                ))
            }
        }

        seek_to_end(reader, &bounds)?;

        Ok(tkhd)
    }
}
