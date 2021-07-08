use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Minf {
    pub stbl: Option<Stbl>,
}

impl TempAtom for Minf {
    const FOURCC: Fourcc = MEDIA_INFORMATION;
}

impl ParseAtom for Minf {
    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let mut minf = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE => minf.stbl = Some(Stbl::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(minf)
    }
}
