use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Udta<'a> {
    pub state: State,
    pub chpl: Option<Chpl<'a>>,
    pub meta: Option<Meta<'a>>,
}

impl Atom for Udta<'_> {
    const FOURCC: Fourcc = USER_DATA;
}

impl ParseAtom for Udta<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut udta = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                CHAPTER_LIST if cfg.cfg.read_chapter_list => {
                    udta.chpl = Some(Chpl::parse(reader, cfg, head.size())?);
                }
                METADATA if cfg.cfg.read_meta_items => {
                    udta.meta = Some(Meta::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(udta)
    }
}

impl AtomSize for Udta<'_> {
    fn size(&self) -> Size {
        let content_len = self.meta.len_or_zero() + self.chpl.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Udta<'_> {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.chpl {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.meta {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Udta<'_> {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.chpl.collect_changes(bounds.end(), level, changes)
            + self.meta.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Udta(self)
    }
}
