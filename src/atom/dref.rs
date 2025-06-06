use super::*;

pub const HEADER_SIZE: u64 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dref {
    pub state: State,
    pub url: Option<Url>,
}

impl Atom for Dref {
    const FOURCC: Fourcc = DATA_REFERENCE;
}

impl ParseAtom for Dref {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return unknown_version("data reference (dref)", version);
        }
        reader.skip(4)?; // number of entries

        expect_min_size("Data ", size, HEADER_SIZE)?;

        let mut dref = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = HEADER_SIZE;
        while parsed_bytes < size.content_len() {
            let remaining_bytes = size.content_len() - parsed_bytes;
            let head = head::parse(reader, remaining_bytes)?;

            match head.fourcc() {
                URL_MEDIA => dref.url = Some(Url::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(dref)
    }
}

impl AtomSize for Dref {
    fn size(&self) -> Size {
        let content_len = 8 + self.url.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Dref {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        if self.url.is_some() {
            writer.write_be_u32(1)?;
        } else {
            writer.write_be_u32(0)?;
        }

        if let Some(a) = &self.url {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Dref {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.url.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Dref(self)
    }
}
