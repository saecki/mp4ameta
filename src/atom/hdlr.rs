use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hdlr {
    pub state: State,
    pub data: Vec<u8>,
}

impl Deref for Hdlr {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Hdlr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Atom for Hdlr {
    const FOURCC: Fourcc = HANDLER_REFERENCE;
}

impl ParseAtom for Hdlr {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        Ok(Self {
            state: State::Existing(bounds),
            data: reader.read_u8_vec(size.content_len())?,
        })
    }
}

impl AtomSize for Hdlr {
    fn size(&self) -> Size {
        Size::from(self.data.len() as u64)
    }
}

impl WriteAtom for Hdlr {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(self)?;
        Ok(())
    }
}

impl LeafAtomCollectChanges for Hdlr {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Hdlr(self)
    }
}

impl Hdlr {
    pub fn meta() -> Self {
        Self {
            state: State::Insert,
            data: vec![
                0x00, 0x00, 0x00, 0x00, // version + flags
                0x00, 0x00, 0x00, 0x00, // component type
                0x6d, 0x64, 0x69, 0x72, // component subtype
                0x61, 0x70, 0x70, 0x6c, // component manufacturer
                0x00, 0x00, 0x00, 0x00, // component flags
                0x00, 0x00, 0x00, 0x00, // component flags mask
                0x00, // component name
            ],
        }
    }

    pub fn text_mdia() -> Self {
        Self {
            state: State::Insert,
            data: vec![
                0x00, 0x00, 0x00, 0x00, // version + flags
                0x00, 0x00, 0x00, 0x00, // component type
                0x74, 0x65, 0x78, 0x74, // component subtype
                0x00, 0x00, 0x00, 0x00, // component manufacturer
                0x00, 0x00, 0x00, 0x00, // component flags
                0x00, 0x00, 0x00, 0x00, // component flags mask
                0x00, // component name
            ],
        }
    }
}
