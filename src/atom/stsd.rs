use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsd {
    pub mp4a: Option<Mp4a>,
    pub text: Option<Text>,
}

impl Atom for Stsd {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_DESCRIPTION;
}

impl ParseAtom for Stsd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown sample table sample description (stsd) version",
            ));
        }

        reader.skip(4)?; // number of entries

        let mut stsd = Self::default();
        let mut parsed_bytes = 8;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MP4_AUDIO => stsd.mp4a = Some(Mp4a::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(stsd)
    }
}

impl WriteAtom for Stsd {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_all(&[0; 4])?; // reserved
        if let Some(a) = &self.text {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + self.text.len_or_zero();
        Size::from(content_len)
    }
}
