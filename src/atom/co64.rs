use super::*;

pub const HEADER_SIZE: u64 = 8;
pub const ENTRY_SIZE: u64 = 8;

/// A struct representing of a 64bit sample table chunk offset atom (`co64`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Co64 {
    pub state: State,
    pub offsets: Table<u64>,
}

impl Atom for Co64 {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET_64;
}

impl ParseAtom for Co64 {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown 64bit sample table chunk offset (co64) version",
            ));
        }

        let num_entries = reader.read_be_u32()?;
        let table_size = ENTRY_SIZE * num_entries as u64;
        if HEADER_SIZE + table_size != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::SizeMismatch,
                "Sample table chunk offset 64 (co64) offset table size doesn't match atom length",
            ));
        }

        let offsets = if cfg.write {
            let offsets = Table::read_items(reader, num_entries)?;
            Table::Full(offsets)
        } else {
            reader.skip(table_size as i64)?;
            Table::Shallow {
                pos: bounds.content_pos() + HEADER_SIZE,
                num_entries,
            }
        };

        Ok(Self { state: State::Existing(bounds), offsets })
    }
}

impl AtomSize for Co64 {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + ENTRY_SIZE * self.offsets.len() as u64;
        Size::from(content_len)
    }
}

impl WriteAtom for Co64 {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        writer.write_be_u32(self.offsets.len() as u32)?;
        match &self.offsets {
            Table::Shallow { .. } => unreachable!(),
            Table::Full(offsets) => {
                change::write_shifted_offsets(writer, offsets, changes)?;
            }
        }

        Ok(())
    }
}

impl LeafAtomCollectChanges for Co64 {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Co64(self)
    }
}
