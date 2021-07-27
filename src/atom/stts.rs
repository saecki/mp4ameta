use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stts {
    pub items: Vec<SttsItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table time to sample (stts) version",
            ));
        }

        let entries = reader.read_be_u32()?;
        if 8 + 8 * entries as u64 != size.content_len() {
            return Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Sample table time to sample (stts) table size doesn't match atom length",
            ));
        }

        let mut items = Vec::with_capacity(entries as usize);
        for _ in 0..entries {
            items.push(SttsItem {
                sample_count: reader.read_be_u32()?,
                sample_duration: reader.read_be_u32()?,
            });
        }

        Ok(Self { items })
    }
}

impl WriteAtom for Stts {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_be_u32(self.items.len() as u32)?;
        for i in self.items.iter() {
            writer.write_be_u32(i.sample_count)?;
            writer.write_be_u32(i.sample_duration)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + 8 * self.items.len() as u64;
        Size::from(content_len)
    }
}

pub struct SttsBounds {
    pub bounds: AtomBounds,
}

impl Deref for SttsBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Stts {
    type Bounds = SttsBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
