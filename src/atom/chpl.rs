use super::*;

pub const DEFAULT_CHPL_TIMESCALE: u32 = 10_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Chpl<'a> {
    Owned(Vec<OwnedChplItem>),
    Borrowed(&'a [BorrowedChplItem<'a>]),
}

pub type OwnedChplItem = ChplItem<String>;
pub type BorrowedChplItem<'a> = ChplItem<&'a str>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChplItem<T: AsRef<str>> {
    pub start: u64,
    pub title: T,
}

impl Atom for Chpl<'_> {
    const FOURCC: Fourcc = CHAPTER_LIST;
}

impl ParseAtom for Chpl<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;
        let mut parsed_bytes = 4;

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

        let entries = reader.read_u8()?;
        parsed_bytes += 1;

        let mut chpl = Vec::with_capacity(entries as usize);
        while parsed_bytes < size.content_len() {
            let start = reader.read_be_u64()?;

            let str_len = reader.read_u8()?;
            let title = reader.read_utf8(str_len as u64)?;

            chpl.push(OwnedChplItem { start, title });

            parsed_bytes += 9 + str_len as u64;
        }

        Ok(Self::Owned(chpl))
    }
}

impl WriteAtom for Chpl<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        match self {
            Self::Owned(v) => {
                writer.write_u8(v.len() as u8)?;
                for c in v.iter() {
                    writer.write_be_u64(c.start)?;
                    writer.write_u8(c.title.len() as u8)?;
                    writer.write_utf8(&c.title)?;
                }
            }
            Self::Borrowed(v) => {
                writer.write_u8(v.len() as u8)?;
                for c in v.iter() {
                    writer.write_be_u64(c.start)?;
                    writer.write_u8(c.title.len() as u8)?;
                    writer.write_utf8(&c.title)?;
                }
            }
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 5 + match self {
            Chpl::Owned(v) => v.iter().map(|c| 9 + c.title.len() as u64).sum::<u64>(),
            Chpl::Borrowed(v) => v.iter().map(|c| 9 + c.title.len() as u64).sum::<u64>(),
        };
        Size::from(content_len)
    }
}

impl Chpl<'_> {
    pub fn owned(self) -> Option<Vec<OwnedChplItem>> {
        match self {
            Self::Owned(v) => Some(v),
            Self::Borrowed(_) => None,
        }
    }
}
