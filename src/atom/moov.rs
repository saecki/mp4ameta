use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Moov<'a> {
    pub mvhd: Option<Mvhd>,
    pub trak: Vec<Trak>,
    pub udta: Option<Udta<'a>>,
}

impl TempAtom for Moov<'_> {
    const FOURCC: Fourcc = MOVIE;
}

impl ParseAtom for Moov<'_> {
    fn parse_atom(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let mut moov = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MOVIE_HEADER => moov.mvhd = Some(Mvhd::parse(reader, head.content_len())?),
                TRACK => moov.trak.push(Trak::parse(reader, head.content_len())?),
                USER_DATA => moov.udta = Some(Udta::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(moov)
    }
}

impl WriteAtom for Moov<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.udta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.udta.as_ref().map_or(0, |a| a.size().len());
        Size::from(content_len)
    }
}
