use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hdlr(pub Vec<u8>);

impl Deref for Hdlr {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hdlr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TempAtom for Hdlr {
    const FOURCC: Fourcc = HANDLER_REFERENCE;
}

impl ParseAtom for Hdlr {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        Ok(Self(data::read_u8_vec(reader, size.content_len())?))
    }
}

impl WriteAtom for Hdlr {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self)?;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(self.0.len() as u64)
    }
}
