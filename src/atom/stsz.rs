use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsz {
    pub state: State,
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
        let bounds = find_bounds(reader, size)?;
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table sample size (stsz) version",
            ));
        }

        let sample_size = reader.read_be_u32()?;
        let entries = reader.read_be_u32()?;
        let table_size = 12 + 4 * entries as u64;

        let sizes = if sample_size == 0 {
            if table_size != size.content_len() {
                return Err(crate::Error::new(
                    crate::ErrorKind::Parsing,
                    format!(
                        "Sample table sample size (stsz) table size {} doesn't match atom content length {}",
                        table_size,
                        size.content_len(),
                    ),
                ));
            }

            let mut sizes = Vec::with_capacity(entries as usize);
            for _ in 0..entries {
                let offset = reader.read_be_u32()?;
                sizes.push(offset);
            }
            sizes
        } else {
            if size.content_len() != 12 {
                return Err(crate::Error::new(
                    crate::ErrorKind::Parsing,
                    format!(
                        "Sample table sample size (stsz) uniform sample size set, but content length {} doesn't match",
                        size.content_len(),
                    ),
                ));
            }

            Vec::new()
        };

        Ok(Self { state: State::Existing(bounds), sample_size, sizes })
    }
}

impl WriteAtom for Stsz {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_be_u32(self.sample_size)?;
        writer.write_be_u32(self.sizes.len() as u32)?;
        for s in self.sizes.iter() {
            writer.write_be_u32(*s)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 12 + 4 * self.sizes.len() as u64;
        Size::from(content_len)
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
