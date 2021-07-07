use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsd {
    pub mp4a: Option<Mp4a>,
}

impl ParseAtom for Stsd {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_DESCRIPTION;

    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown sample table sample description (stsd) version".to_owned(),
            ));
        }

        reader.seek(SeekFrom::Current(4))?;

        let mut stsd = Self::default();
        let mut parsed_bytes = 8;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                MP4_AUDIO => stsd.mp4a = Some(Mp4a::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(stsd)
    }
}
