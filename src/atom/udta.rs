use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Udta {
    pub meta: Option<Meta>,
}

impl ParseAtom for Udta {
    const FOURCC: Fourcc = USER_DATA;

    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let mut udta = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                METADATA => udta.meta = Some(Meta::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(udta)
    }
}
