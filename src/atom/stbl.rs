use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stbl {
    pub stsd: Option<Stsd>,
}

impl ParseAtom for Stbl {
    const FOURCC: Fourcc = SAMPLE_TABLE;

    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let mut stbl = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                SAMPLE_TABLE_SAMPLE_DESCRIPTION => {
                    stbl.stsd = Some(Stsd::parse(reader, head.content_len())?)
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(stbl)
    }
}
