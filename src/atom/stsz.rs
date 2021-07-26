use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsz {
    pub table_pos: u64,
    pub sample_size: u32,
    pub sizes: Vec<u32>,
}

impl Atom for Stsz {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_SIZE;
}

impl ParseAtom for Stsz {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table sample size (stsz) version",
            ));
        }

        let sample_size = reader.read_be_u32()?;
        let entries = reader.read_be_u32()?;
        if 12 + 4 * entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table sample size (stsz) table size doesn't match atom length",
            ));
        }

        let table_pos = reader.seek(SeekFrom::Current(0))?;
        let mut sizes = Vec::with_capacity(entries as usize);
        for _ in 0..entries {
            let offset = reader.read_be_u32()?;
            sizes.push(offset);
        }

        Ok(Self { table_pos, sample_size, sizes })
    }
}

pub struct StszBounds {
    pub bounds: AtomBounds,
}

impl Deref for StszBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Stsz {
    type Bounds = StszBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
