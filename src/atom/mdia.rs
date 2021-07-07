use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdia {
    pub minf: Option<Minf>,
}

impl ParseAtom for Mdia {
    const FOURCC: Fourcc = MEDIA;

    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let mut mdia = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                MEDIA_INFORMATION => mdia.minf = Some(Minf::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(mdia)
    }
}
