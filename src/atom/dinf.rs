use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Dinf {
    pub state: State,
    pub dref: Option<Dref>,
}

impl Atom for Dinf {
    const FOURCC: Fourcc = DATA_INFORMATION;
}

impl ParseAtom for Dinf {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut dinf = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                DATA_REFERENCE => dinf.dref = Some(Dref::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(dinf)
    }
}

impl WriteAtom for Dinf {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.dref {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.dref.len_or_zero();
        Size::from(content_len)
    }
}
