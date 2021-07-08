use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Udta<'a> {
    pub meta: Option<Meta<'a>>,
}

impl TempAtom for Udta<'_> {
    const FOURCC: Fourcc = USER_DATA;
}

impl ParseAtom for Udta<'_> {
    fn parse_atom(
        reader: &mut (impl std::io::Read + std::io::Seek),
        len: u64,
    ) -> crate::Result<Self> {
        let mut udta = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                METADATA => udta.meta = Some(Meta::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(udta)
    }
}

impl WriteAtom for Udta<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.meta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.meta.as_ref().map_or(0, |a| a.size().len());
        Size::from(content_len)
    }
}
