use super::*;

pub const DEFAULT_TIMESCALE: NonZeroU32 = NonZeroU32::new(10_000_000).unwrap();

pub const HEADER_SIZE_V0: u64 = 5;
pub const HEADER_SIZE_V1: u64 = 9;
pub const ITEM_HEADER_SIZE: u64 = 9;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chpl<'a> {
    pub state: State,
    pub data: ChplData<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChplData<'a> {
    Owned(Vec<ChplItem>),
    Borrowed(u32, &'a [Chapter]),
}

impl Default for ChplData<'_> {
    fn default() -> Self {
        ChplData::Owned(Vec::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChplItem {
    pub start: u64,
    pub title: String,
}

impl Atom for Chpl<'_> {
    const FOURCC: Fourcc = CHAPTER_LIST;
}

impl ParseAtom for Chpl<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;
        let header_size = match version {
            0 => HEADER_SIZE_V0,
            1 => {
                reader.skip(4)?; // ???
                HEADER_SIZE_V1
            }
            _ => {
                return unknown_version("chapter list (chpl)", version);
            }
        };

        expect_min_size("Chapter list (chpl)", size, header_size)?;

        let num_entries = reader.read_u8()?;
        let table_size = size.content_len() - header_size;
        let mut buf = vec![0; table_size as usize];
        reader.read_exact(&mut buf)?;

        let mut cursor = std::io::Cursor::new(buf);

        let mut chpl = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let start = cursor.read_be_u64()?;

            let str_len = cursor.read_u8()?;
            let title = cursor.read_utf8(str_len as u64)?;

            chpl.push(ChplItem { start, title });
        }

        Ok(Self {
            state: State::Existing(bounds),
            data: ChplData::Owned(chpl),
        })
    }
}

impl AtomSize for Chpl<'_> {
    fn size(&self) -> Size {
        let data_len = match &self.data {
            ChplData::Owned(v) => {
                v.iter().map(|c| ITEM_HEADER_SIZE + title_len(&c.title) as u64).sum::<u64>()
            }
            ChplData::Borrowed(_, v) => {
                v.iter().map(|c| ITEM_HEADER_SIZE + title_len(&c.title) as u64).sum::<u64>()
            }
        };
        let content_len = HEADER_SIZE_V0 + data_len;
        Size::from(content_len)
    }
}

impl WriteAtom for Chpl<'_> {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, Flags::ZERO)?;

        match &self.data {
            ChplData::Owned(v) => {
                writer.write_u8(v.len() as u8)?;
                for c in v.iter() {
                    writer.write_be_u64(c.start)?;

                    let title_len = title_len(&c.title);
                    writer.write_u8(title_len as u8)?;
                    writer.write_utf8(&c.title[..title_len])?;
                }
            }
            ChplData::Borrowed(timescale, chapters) => {
                writer.write_u8(chapters.len() as u8)?;
                for c in chapters.iter() {
                    let start = unscale_duration(*timescale, c.start);
                    writer.write_be_u64(start)?;

                    let title_len = title_len(&c.title);
                    writer.write_u8(title_len as u8)?;
                    writer.write_utf8(&c.title[..title_len])?;
                }
            }
        }

        Ok(())
    }
}

impl LeafAtomCollectChanges for Chpl<'_> {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Chpl(self)
    }
}

impl Chpl<'_> {
    pub fn into_owned(self) -> Option<Vec<ChplItem>> {
        match self.data {
            ChplData::Owned(v) => Some(v),
            ChplData::Borrowed(_, _) => None,
        }
    }
}

fn title_len(title: &str) -> usize {
    title.len().min(u8::MAX as usize)
}
