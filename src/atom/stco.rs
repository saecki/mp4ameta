use std::io::{Read, Seek, SeekFrom};

use super::*;

/// A struct representing of a sample table chunk offset atom (`stco`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stco {
    pub table_pos: u64,
    pub offsets: Vec<u32>,
}

impl TempAtom for Stco {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET;
}

impl ParseAtom for Stco {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        match version {
            0 => {
                let entries = data::read_u32(reader)?;
                if 8 + 4 * entries as u64 != size.content_len() {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        "Sample table chunk offset (stco) offset table size doesn't match atom length".to_owned(),
                    ));
                }

                let table_pos = reader.seek(SeekFrom::Current(0))?;
                let mut offsets = Vec::with_capacity(entries as usize);
                for _ in 0..entries {
                    let offset = data::read_u32(reader)?;
                    offsets.push(offset);
                }

                Ok(Self { table_pos, offsets })
            }
            _ => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table chunk offset (stco) version".to_owned(),
            )),
        }
    }
}
