use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Text {
    pub state: State,
    pub data: Cow<'static, [u8]>,
}

impl Atom for Text {
    const FOURCC: Fourcc = TEXT_MEDIA;
}

impl ParseAtom for Text {
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

impl AtomSize for Text {
    fn size(&self) -> Size {
        Size::from(self.data.len() as u64)
    }
}

impl WriteAtom for Text {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

impl LeafAtomCollectChanges for Text {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Text(self)
    }
}

impl Text {
    pub fn media_chapter() -> Self {
        Self {
            state: State::Insert,
            data: Cow::Borrowed(&[
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
            ]),
        }
    }

    pub fn media_information_chapter() -> Self {
        Self {
            state: State::Insert,
            data: Cow::Borrowed(&[
                0x00, 0x01, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x01, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, //
            ]),
        }
    }
}
