use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Text(pub Vec<u8>);

impl Deref for Text {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Atom for Text {
    const FOURCC: Fourcc = TEXT_MEDIA;
}

impl ParseAtom for Text {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        Ok(Self(reader.read_u8_vec(size.content_len())?))
    }
}

impl WriteAtom for Text {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self)?;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(self.0.len() as u64)
    }
}

impl Text {
    pub fn chapter() -> Self {
        Self(vec![
            // Text Sample Entry
            0x00, 0x00, 0x00, 0x01, // displayFlags
            0x00, // horizontal justification
            0x00, // vertical justification
            0x00, // bg color red
            0x00, // bg color green
            0x00, // bg color blue
            0x00, // bg color alpha
            // Box Record
            0x00, 0x00, // def text box top
            0x00, 0x00, // def text box left
            0x00, 0x00, // def text box bottom
            0x00, 0x00, // def text box right
            // Style Record
            0x00, 0x00, // start char
            0x00, 0x00, // end char
            0x00, 0x01, // font ID
            0x00, // font style flags
            0x00, // font size
            0x00, // fg color red
            0x00, // fg color green
            0x00, // fg color blue
            0x00, // fg color alpha
            // Font Table Box
            0x00, 0x00, 0x00, 0x0D, // box size
            b'f', b't', b'a', b'b', // box atom name
            0x00, 0x01, // entry count
            // Font Record
            0x00, 0x01, // font ID
            0x00, // font name length
        ])
    }
}
