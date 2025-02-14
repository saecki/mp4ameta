use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Gmhd {
    pub state: State,
    pub gmin: Option<Gmin>,
    pub text: Option<Text>,
}

impl Atom for Gmhd {
    const FOURCC: Fourcc = BASE_MEDIA_INFORMATION_HEADER;
}

impl ParseAtom for Gmhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut gmhd = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                BASE_MEDIA_INFORMATION => gmhd.gmin = Some(Gmin::parse(reader, cfg, head.size())?),
                TEXT_MEDIA => gmhd.text = Some(Text::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(gmhd)
    }
}

impl WriteAtom for Gmhd {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;

        if let Some(a) = &self.gmin {
            a.write(writer)?;
        }
        if let Some(a) = &self.text {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.gmin.len_or_zero() + self.text.len_or_zero();
        Size::from(content_len)
    }
}
