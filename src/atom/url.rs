use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Url(pub Vec<u8>);

impl Deref for Url {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Atom for Url {
    const FOURCC: Fourcc = URL_MEDIA;
}

impl ParseAtom for Url {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        Ok(Self(reader.read_u8_vec(size.content_len())?))
    }
}

impl WriteAtom for Url {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(self)?;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(self.0.len() as u64)
    }
}
