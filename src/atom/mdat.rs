use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdat {
    pub data: Vec<u8>,
}

impl Atom for Mdat {
    const FOURCC: Fourcc = MEDIA_DATA;
}

impl WriteAtom for Mdat {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.data.len() as u64;
        Size::from(content_len)
    }
}

pub struct MdatBounds {
    pub bounds: AtomBounds,
}

impl Deref for MdatBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Mdat {
    type Bounds = MdatBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
