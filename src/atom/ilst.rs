use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ilst<'a> {
    pub state: State,
    pub data: IlstData<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IlstData<'a> {
    Owned(Vec<MetaItem>),
    Borrowed(&'a [MetaItem]),
}

impl Default for IlstData<'_> {
    fn default() -> Self {
        IlstData::Owned(Vec::new())
    }
}

impl Deref for Ilst<'_> {
    type Target = [MetaItem];

    fn deref(&self) -> &Self::Target {
        match &self.data {
            IlstData::Owned(a) => a,
            IlstData::Borrowed(a) => a,
        }
    }
}

impl Atom for Ilst<'_> {
    const FOURCC: Fourcc = ITEM_LIST;
}

impl ParseAtom for Ilst<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut ilst = Vec::<MetaItem>::new();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                FREE => reader.skip(head.content_len() as i64)?,
                _ => {
                    let atom = MetaItem::parse(reader, cfg, head)?;
                    let other = ilst.iter_mut().find(|o| atom.ident == o.ident);

                    match other {
                        Some(other) => other.data.extend(atom.data),
                        None => ilst.push(atom),
                    }
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self {
            state: State::Existing(bounds),
            data: IlstData::Owned(ilst),
        })
    }
}

impl AtomSize for Ilst<'_> {
    fn size(&self) -> Size {
        let content_len = self.iter().map(|a| a.len()).sum();
        Size::from(content_len)
    }
}

impl WriteAtom for Ilst<'_> {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        for a in self.iter() {
            a.write(writer)?;
        }
        Ok(())
    }
}

// Not really a leaf atom, but it is treated like one.
impl LeafAtomCollectChanges for Ilst<'_> {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Ilst(self)
    }
}

impl Ilst<'_> {
    pub fn owned(self) -> Option<Vec<MetaItem>> {
        match self.data {
            IlstData::Owned(a) => Some(a),
            IlstData::Borrowed(_) => None,
        }
    }
}
