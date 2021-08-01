use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Udta<'a> {
    pub chpl: Option<Chpl<'a>>,
    pub meta: Option<Meta<'a>>,
}

impl Atom for Udta<'_> {
    const FOURCC: Fourcc = USER_DATA;
}

impl ParseAtom for Udta<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut udta = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                CHAPTER_LIST if cfg.read_chapters => {
                    udta.chpl = Some(Chpl::parse(reader, cfg, head.size())?)
                }
                METADATA if cfg.read_item_list => {
                    udta.meta = Some(Meta::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(udta)
    }
}

impl WriteAtom for Udta<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.chpl {
            a.write(writer)?;
        }
        if let Some(a) = &self.meta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.meta.len_or_zero() + self.chpl.len_or_zero();
        Size::from(content_len)
    }
}

#[derive(Default)]
pub struct UdtaBounds {
    pub bounds: AtomBounds,
    pub chpl: Option<ChplBounds>,
    pub meta: Option<MetaBounds>,
}

impl Deref for UdtaBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Udta<'_> {
    type Bounds = UdtaBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut udta = Self::Bounds { bounds, ..Default::default() };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                METADATA => udta.meta = Some(Meta::find(reader, head.size())?),
                CHAPTER_LIST => udta.chpl = Some(Chpl::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(udta)
    }
}
