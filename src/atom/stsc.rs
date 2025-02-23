use super::*;

pub const HEADER_SIZE: u64 = 8;
pub const ENTRY_SIZE: u64 = 12;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stsc {
    pub state: State,
    pub items: Table<StscItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StscItem {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_id: u32,
}

impl Atom for Stsc {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_TO_CHUNK;
}

impl ParseAtom for Stsc {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return unknown_version("sample table sample size (stsz)", version);
        }

        let num_entries = reader.read_be_u32()?;
        let table_size = ENTRY_SIZE * num_entries as u64;
        expect_size("Sample table sample to chunk (stsc)", size, HEADER_SIZE + table_size)?;

        reader.skip(table_size as i64)?;
        let items = Table::Shallow {
            pos: bounds.content_pos() + HEADER_SIZE,
            num_entries,
        };

        Ok(Self { state: State::Existing(bounds), items })
    }
}

impl AtomSize for Stsc {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + ENTRY_SIZE * self.items.len() as u64;
        Size::from(content_len)
    }
}

impl WriteAtom for Stsc {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        writer.write_be_u32(self.items.len() as u32)?;
        match &self.items {
            Table::Shallow { .. } => unreachable!(),
            Table::Full(items) => {
                for i in items.iter() {
                    writer.write_be_u32(i.first_chunk)?;
                    writer.write_be_u32(i.samples_per_chunk)?;
                    writer.write_be_u32(i.sample_description_id)?;
                }
            }
        }

        Ok(())
    }
}

impl LeafAtomCollectChanges for Stsc {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stsc(self)
    }
}
