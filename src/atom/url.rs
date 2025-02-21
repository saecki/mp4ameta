use std::borrow::Cow;

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Url {
    pub state: State,
    pub data: Cow<'static, [u8]>,
}

impl Atom for Url {
    const FOURCC: Fourcc = URL_MEDIA;
}

impl ParseAtom for Url {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let data = reader.read_u8_vec(size.content_len())?;
        Ok(Self {
            state: State::Existing(bounds),
            data: Cow::Owned(data),
        })
    }
}

impl AtomSize for Url {
    fn size(&self) -> Size {
        Size::from(self.data.len() as u64)
    }
}

impl WriteAtom for Url {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

impl Url {
    pub fn track() -> Self {
        Self {
            state: State::Insert,
            data: Cow::Borrowed(&[0x01, 0x00, 0x00, 0x00]),
        }
    }
}

impl LeafAtomCollectChanges for Url {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Url(self)
    }
}
