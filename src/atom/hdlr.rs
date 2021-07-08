use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hdlr(Vec<u8>);

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
    fn parse_atom(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        Ok(Self(data::read_u8_vec(reader, len)?))
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

impl Hdlr {
    pub fn meta() -> Self {
        Self(vec![
            0x00, 0x00, 0x00, 0x00, // version + flags
            0x00, 0x00, 0x00, 0x00, // component type
            0x6d, 0x64, 0x69, 0x72, // component subtype
            0x61, 0x70, 0x70, 0x6c, // component manufacturer
            0x00, 0x00, 0x00, 0x00, // component flags
            0x00, 0x00, 0x00, 0x00, // component flags mask
            0x00, // component name
        ])
    }
}
