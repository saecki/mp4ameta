use std::io::{Read, Seek, SeekFrom};

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tkhd {
    /// The duration of the track.
    pub id: u32,
}

impl Atom for Tkhd {
    const FOURCC: Fourcc = TRACK_HEADER;
}

impl ParseAtom for Tkhd {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut tkhd = Self::default();

        let (version, _) = parse_full_head(reader)?;
        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes motification time
                // 4 bytes track id
                // ...
                reader.seek(SeekFrom::Current(8))?;
                tkhd.id = reader.read_u32()?;
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes motification time
                // 4 bytes track id
                // ...
                reader.seek(SeekFrom::Current(16))?;

                tkhd.id = reader.read_u32()?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (mvhd) version {}", v),
                ))
            }
        }

        seek_to_end(reader, &bounds)?;

        println!("track id: {}", tkhd.id);

        Ok(tkhd)
    }
}
