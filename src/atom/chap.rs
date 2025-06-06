use super::*;

pub const ENTRY_SIZE: u64 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chap {
    pub state: State,
    pub chapter_ids: Vec<u32>,
}

impl Atom for Chap {
    const FOURCC: Fourcc = CHAPTER_REFERENCE;
}

impl ParseAtom for Chap {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        if size.content_len() % 4 != 0 {
            return Err(crate::Error::new(
                ErrorKind::InvalidAtomSize,
                "Chapter reference (chap) atom size is not a multiple of 4",
            ));
        }

        let num_entries = size.content_len() / ENTRY_SIZE;
        let mut chapter_ids = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            chapter_ids.push(reader.read_be_u32()?);
        }

        Ok(Self { state: State::Existing(bounds), chapter_ids })
    }
}

impl AtomSize for Chap {
    fn size(&self) -> Size {
        let content_len = ENTRY_SIZE * self.chapter_ids.len() as u64;
        Size::from(content_len)
    }
}

impl WriteAtom for Chap {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        for c in self.chapter_ids.iter() {
            writer.write_be_u32(*c)?;
        }
        Ok(())
    }
}

impl LeafAtomCollectChanges for Chap {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Chap(self)
    }
}
