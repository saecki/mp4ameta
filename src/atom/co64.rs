use super::*;

/// A struct representing of a 64bit sample table chunk offset atom (`co64`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Co64 {
    pub offsets: Vec<u64>,
}

impl Atom for Co64 {
    const FOURCC: Fourcc = SAMPLE_TABLE_CHUNK_OFFSET_64;
}

impl ParseAtom for Co64 {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        match version {
            0 => {
                let entries = reader.read_be_u32()?;
                if 8 + 8 * entries as u64 != size.content_len() {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        "Sample table chunk offset 64 (co64) offset table size doesn't match atom length",
                    ));
                }

                let mut offsets = Vec::with_capacity(entries as usize);
                for _ in 0..entries {
                    let offset = reader.read_be_u64()?;
                    offsets.push(offset);
                }

                Ok(Self { offsets })
            }
            _ => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown 64bit sample table chunk offset (co64) version",
            )),
        }
    }
}

impl WriteAtom for Co64 {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        writer.write_be_u32(self.offsets.len() as u32)?;
        for o in self.offsets.iter() {
            writer.write_be_u64(*o)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + 8 * self.offsets.len() as u64;
        Size::from(content_len)
    }
}

pub struct Co64Bounds {
    pub bounds: AtomBounds,
}

impl Deref for Co64Bounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Co64 {
    type Bounds = Co64Bounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
