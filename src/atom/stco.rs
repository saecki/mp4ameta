use super::*;

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
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table chunk offset (stco) version",
            ));
        }

        let entries = reader.read_be_u32()?;
        if 8 + 4 * entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table chunk offset (stco) table size doesn't match atom length",
            ));
        }

        let mut offsets = Vec::with_capacity(entries as usize);
        for _ in 0..entries {
            let offset = reader.read_be_u32()?;
            offsets.push(offset);
        }

        Ok(Self { state: State::Existing(bounds), offsets })
    }
}

impl WriteAtom for Stco {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_be_u32(self.offsets.len() as u32)?;
        for o in self.offsets.iter() {
            writer.write_be_u32(*o)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + 4 * self.offsets.len() as u64;
        Size::from(content_len)
    }
}

pub struct StcoBounds {
    pub bounds: AtomBounds,
}

impl Deref for StcoBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Stco {
    type Bounds = StcoBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
