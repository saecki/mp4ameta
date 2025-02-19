use super::*;

pub const DEFAULT_TIMESCALE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(10_000_000) };

pub const HEADER_SIZE: u64 = 5;
pub const ITEM_HEADER_SIZE: u64 = 9;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chpl<'a> {
    pub state: State,
    pub data: ChplData<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChplData<'a> {
    Owned(Vec<ChplItem>),
    Borrowed(u32, &'a [Chapter]),
}

impl Default for ChplData<'_> {
    fn default() -> Self {
        ChplData::Owned(Vec::new())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
        let mut parsed_bytes = HEADER_SIZE;

        match version {
            0 => (),
            1 => {
                reader.skip(4)?; // ???
                parsed_bytes += 4;
            }
            _ => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    "Unknown chapter list (chpl) version",
                ));
            }
        }

        let num_entries = reader.read_u8()?;

        let mut chpl = Vec::with_capacity(num_entries as usize);
        while parsed_bytes < size.content_len() {
            let start = reader.read_be_u64()?;

            let str_len = reader.read_u8()?;
            let title = reader.read_utf8(str_len as u64)?;

            chpl.push(ChplItem { start, title });

            parsed_bytes += ITEM_HEADER_SIZE + str_len as u64;
        }

        Ok(Self {
            state: State::Existing(bounds),
            data: ChplData::Owned(chpl),
        })
    }
}

impl WriteAtom for Chpl<'_> {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        match &self.data {
            ChplData::Owned(v) => {
                writer.write_u8(v.len() as u8)?;
                for c in v.iter() {
                    writer.write_be_u64(c.start)?;

                    let title_len = c.title.len().min(u8::MAX as usize);
                    writer.write_u8(title_len as u8)?;
                    writer.write_utf8(&c.title[..title_len])?;
                }
            }
            ChplData::Borrowed(timescale, chapters) => {
                writer.write_u8(chapters.len() as u8)?;
                for c in chapters.iter() {
                    let start = unscale_duration(*timescale, c.start);
                    writer.write_be_u64(start)?;
                    writer.write_u8(c.title.len() as u8)?;
                    writer.write_utf8(&c.title)?;
                }
            }
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let data_len = match &self.data {
            ChplData::Owned(v) => {
                v.iter().map(|c| ITEM_HEADER_SIZE + c.title.len() as u64).sum::<u64>()
            }
            ChplData::Borrowed(_, v) => {
                v.iter().map(|c| ITEM_HEADER_SIZE + c.title.len() as u64).sum::<u64>()
            }
        };
        let content_len = HEADER_SIZE + data_len;
        Size::from(content_len)
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
