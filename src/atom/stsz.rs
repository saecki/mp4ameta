use super::*;

pub const HEADER_SIZE: u64 = 12;
pub const ENTRY_SIZE: u64 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stsz {
    pub state: State,
    /// If this field is set to zero, a list of sizes is read instead.
    pub uniform_sample_size: u32,
    pub sizes: Table<u32>,
}

impl Atom for Stsz {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_SIZE;
}

impl ParseAtom for Stsz {
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
                "Unknown sample table sample size (stsz) version",
            ));
        }

        let uniform_sample_size = reader.read_be_u32()?;

        let num_entries = reader.read_be_u32()?;
        let sizes = if uniform_sample_size == 0 {
            let table_size = ENTRY_SIZE * num_entries as u64;
            let content_size = HEADER_SIZE + table_size;
            if content_size != size.content_len() {
                return Err(crate::Error::new(
                    crate::ErrorKind::SizeMismatch,
                    format!(
                        "Sample table sample size (stsz) table size {} doesn't match atom content length {}",
                        content_size,
                        size.content_len(),
                    ),
                ));
            }

            reader.skip(table_size as i64)?;

            Table::Shallow {
                pos: bounds.content_pos() + HEADER_SIZE,
                num_entries,
            }
        } else {
            if size.content_len() != HEADER_SIZE {
                return Err(crate::Error::new(
                    crate::ErrorKind::SizeMismatch,
                    format!(
                        "Sample table sample size (stsz) uniform sample size set, but content length {} doesn't match",
                        size.content_len(),
                    ),
                ));
            }

            Table::Full(Vec::new())
        };

        Ok(Self {
            state: State::Existing(bounds),
            uniform_sample_size,
            sizes,
        })
    }
}

impl AtomSize for Stsz {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + ENTRY_SIZE * self.sizes.len() as u64;
        Size::from(content_len)
    }
}

impl WriteAtom for Stsz {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        writer.write_be_u32(self.uniform_sample_size)?;
        writer.write_be_u32(self.sizes.len() as u32)?;

        match &self.sizes {
            Table::Shallow { .. } => unreachable!(),
            Table::Full(sizes) => {
                for s in sizes.iter() {
                    writer.write_be_u32(*s)?;
                }
            }
        }

        Ok(())
    }
}

impl LeafAtomCollectChanges for Stsz {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stsz(self)
    }
}
