use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdat {
    pub data: Vec<u8>,
}

impl Atom for Mdat {
    const FOURCC: Fourcc = MEDIA_DATA;
}

impl WriteAtom for Mdat {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.data.len() as u64;
        Size::new(true, content_len)
    }
}

impl Mdat {
    pub fn read_bounds(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<AtomBounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(bounds)
    }
}
