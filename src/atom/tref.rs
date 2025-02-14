use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tref {
    pub state: State,
    pub chap: Option<Chap>,
}

impl Atom for Tref {
    const FOURCC: Fourcc = TRACK_REFERENCE;
}

impl ParseAtom for Tref {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut tref = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                CHAPTER => tref.chap = Some(Chap::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(tref)
    }
}

impl WriteAtom for Tref {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.chap {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.chap.len_or_zero();
        Size::from(content_len)
    }
}
