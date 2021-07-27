use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chap {
    pub chapter_ids: Vec<u32>,
}

impl Atom for Chap {
    const FOURCC: Fourcc = CHAPTER;
}

impl ParseAtom for Chap {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let count = size.content_len() as usize / 4;
        let mut chapter_ids = Vec::with_capacity(count);

        for _ in 0..count {
            chapter_ids.push(reader.read_be_u32()?);
        }

        Ok(Self { chapter_ids })
    }
}

impl WriteAtom for Chap {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        for c in self.chapter_ids.iter() {
            writer.write_be_u32(*c)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 4 * self.chapter_ids.len() as u64;
        Size::from(content_len)
    }
}
