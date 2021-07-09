use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsd {
    pub mp4a: Option<Mp4a>,
}

impl TempAtom for Stsd {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_DESCRIPTION;
}

impl ParseAtom for Stsd {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
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

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MP4_AUDIO => stsd.mp4a = Some(Mp4a::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(stsd)
    }
}
