use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tref {
    pub chap: Option<Chap>,
}

impl Atom for Tref {
    const FOURCC: Fourcc = TRACK_REFERENCE;
}

impl ParseAtom for Tref {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut tref = Self::default();
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

#[derive(Default)]
pub struct TrefBounds {
    pub bounds: AtomBounds,
    pub chap: Option<ChapBounds>,
}

impl FindAtom for Tref {
    type Bounds = TrefBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut tref = TrefBounds { bounds, ..Default::default() };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                CHAPTER => tref.chap = Some(Chap::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(tref)
    }
}
