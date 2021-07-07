use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Meta {
    pub ilst: Option<Ilst>,
}

impl ParseAtom for Meta {
    const FOURCC: Fourcc = METADATA;

    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown metadata (meta) version".to_owned(),
            ));
        }

        let mut meta = Self::default();
        let mut parsed_bytes = 4;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                ITEM_LIST => meta.ilst = Some(Ilst::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(meta)
    }
}
