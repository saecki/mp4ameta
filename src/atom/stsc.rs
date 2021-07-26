use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsc {
    pub table_pos: u64,
    pub items: Vec<StscItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StscItem {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_id: u32,
}

impl Atom for Stsc {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_TO_COUNT;
}

impl ParseAtom for Stsc {
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

        let entries = reader.read_be_u32()?;
        if 8 + 12 * entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table sample size (stsz) table size doesn't match atom length",
            ));
        }

        let table_pos = reader.seek(SeekFrom::Current(0))?;
        let mut items = Vec::with_capacity(entries as usize);
        for _ in 0..entries {
            items.push(StscItem {
                first_chunk: reader.read_be_u32()?,
                samples_per_chunk: reader.read_be_u32()?,
                sample_description_id: reader.read_be_u32()?,
            });
        }

        Ok(Self { table_pos, items })
    }
}

pub struct StscBounds {
    pub bounds: AtomBounds,
}

impl Deref for StscBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Stsc {
    type Bounds = StscBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
