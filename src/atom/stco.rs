use super::*;

pub const HEADER_SIZE: u64 = 8;
pub const ENTRY_SIZE: u64 = 4;

/// A struct representing of a sample table chunk offset atom (`stco`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stco {
    pub state: State,
    pub offsets: Vec<u32>,
}

impl Atom for Stco {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET;
}

impl ParseAtom for Stco {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table chunk offset (stco) version",
            ));
        }

        let num_entries = reader.read_be_u32()?;
        if HEADER_SIZE + ENTRY_SIZE * num_entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table chunk offset (stco) table size doesn't match atom length",
            ));
        }

        let mut offsets = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let offset = reader.read_be_u32()?;
            offsets.push(offset);
        }

        Ok(Self { state: State::Existing(bounds), offsets })
    }
}

impl WriteAtom for Stco {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        writer.write_be_u32(self.offsets.len() as u32)?;
        change::write_shifted_offsets(writer, &self.offsets, changes)?;

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + ENTRY_SIZE * self.offsets.len() as u64;
        Size::from(content_len)
    }
}

impl SimpleCollectChanges for Stco {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        _level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        changes.push(Change::UpdateChunkOffset(UpdateChunkOffsets {
            bounds,
            offsets: ChunkOffsets::Stco(&self.offsets),
        }));
        0
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stco(self)
    }
}
