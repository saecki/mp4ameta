use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
            let remaining_bytes = size.content_len() - parsed_bytes;
            let head = head::parse(reader, remaining_bytes)?;

            match head.fourcc() {
                CHAPTER_REFERENCE => tref.chap = Some(Chap::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(tref)
    }
}

impl AtomSize for Tref {
    fn size(&self) -> Size {
        let content_len = self.chap.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Tref {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.chap {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Tref {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.chap.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Tref(self)
    }
}
