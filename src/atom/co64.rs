use std::io::{Read, Seek, SeekFrom};

use super::*;

/// A struct representing of a 64bit sample table chunk offset atom (`co64`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Co64 {
    pub table_pos: u64,
    pub offsets: Vec<u64>,
}

impl TempAtom for Co64 {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET_64;
}

impl ParseAtom for Co64 {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        match version {
            0 => {
                let entries = data::read_u32(reader)?;
                if 8 + 8 * entries as u64 != size.content_len() {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        "Sample table chunk offset 64 (co64) offset table size doesn't match atom length".to_owned(),
                    ));
                }

                let table_pos = reader.seek(SeekFrom::Current(0))?;
                let mut offsets = Vec::with_capacity(entries as usize);
                for _ in 0..entries {
                    let offset = data::read_u64(reader)?;
                    offsets.push(offset);
                }

                Ok(Self { table_pos, offsets })
            }
            _ => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown 64bit sample table chunk offset (co64) version".to_owned(),
            )),
        }
    }
}
