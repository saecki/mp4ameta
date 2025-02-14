use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Url {
    pub state: State,
    pub data: Vec<u8>,
}

impl Deref for Url {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
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
        Ok(Self {
            state: State::Existing(bounds),
            data: reader.read_u8_vec(size.content_len())?,
        })
    }
}

impl WriteAtom for Url {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(self)?;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(self.data.len() as u64)
    }
}

impl Url {
    pub fn track() -> Self {
        Self {
            state: State::Insert,
            data: vec![0x01, 0x00, 0x00, 0x00],
        }
    }
}
