use super::*;

pub const HEADER_SIZE: u64 = 8;
pub const ENTRY_SIZE: u64 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stts {
    pub state: State,
    pub items: Table<SttsItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SttsItem {
    pub sample_count: u32,
    pub sample_duration: u32,
}

impl Atom for Stts {
    const FOURCC: Fourcc = SAMPLE_TABLE_TIME_TO_SAMPLE;
}

impl ParseAtom for Stts {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return unknown_version("sample table time to sample (stts)", version);
        }

        let num_entries = reader.read_be_u32()?;
        let table_size = ENTRY_SIZE * num_entries as u64;
        expect_size("Sample table time to sample (stts)", size, HEADER_SIZE + table_size)?;

        reader.skip(table_size as i64)?;
        let items = Table::Shallow {
            pos: bounds.content_pos() + HEADER_SIZE,
            num_entries,
        };

        Ok(Self { state: State::Existing(bounds), items })
    }
}

impl AtomSize for Stts {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + ENTRY_SIZE * self.items.len() as u64;
        Size::from(content_len)
    }
}

impl WriteAtom for Stts {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, Flags::ZERO)?;

        writer.write_be_u32(self.items.len() as u32)?;
        match &self.items {
            Table::Shallow { .. } => unreachable!(),
            Table::Full(items) => {
                for i in items.iter() {
                    writer.write_be_u32(i.sample_count)?;
                    writer.write_be_u32(i.sample_duration)?;
                }
            }
        }

        Ok(())
    }
}

impl LeafAtomCollectChanges for Stts {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stts(self)
    }
}
