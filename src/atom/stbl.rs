use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stbl {
    pub state: State,
    pub stsd: Option<Stsd>,
    pub stts: Option<Stts>,
    pub stsc: Option<Stsc>,
    pub stsz: Option<Stsz>,
    pub stco: Option<Stco>,
    pub co64: Option<Co64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Table<T> {
    Shallow { pos: u64, num_entries: u32 },
    Full(Vec<T>),
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::Full(Vec::new())
    }
}

impl<T> Table<T> {
    pub fn len(&self) -> usize {
        match self {
            Table::Shallow { num_entries, .. } => *num_entries as usize,
            Table::Full(items) => items.len(),
        }
    }
}

impl<T: ReadItem> Table<T> {
    pub fn get_or_read<'a>(
        &'a self,
        reader: &mut (impl Read + Seek),
    ) -> Result<Cow<'a, [T]>, crate::Error> {
        match self {
            &Table::Shallow { pos, num_entries } => {
                reader.seek(SeekFrom::Start(pos))?;
                let items = Self::read_items(reader, num_entries)?;
                Ok(Cow::Owned(items))
            }
            Table::Full(items) => Ok(Cow::Borrowed(items)),
        }
    }

    pub fn read_items(reader: &mut impl Read, num_entries: u32) -> Result<Vec<T>, crate::Error> {
        let mut items = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            items.push(T::read_item(reader)?);
        }
        Ok(items)
    }
}

pub trait ReadItem: Sized + Clone {
    fn read_item(reader: &mut impl Read) -> std::io::Result<Self>;
}

impl ReadItem for u32 {
    fn read_item(reader: &mut impl Read) -> std::io::Result<Self> {
        reader.read_be_u32()
    }
}

impl ReadItem for u64 {
    fn read_item(reader: &mut impl Read) -> std::io::Result<Self> {
        reader.read_be_u64()
    }
}

impl ReadItem for SttsItem {
    fn read_item(reader: &mut impl Read) -> std::io::Result<Self> {
        Ok(SttsItem {
            sample_count: reader.read_be_u32()?,
            sample_duration: reader.read_be_u32()?,
        })
    }
}

impl ReadItem for StscItem {
    fn read_item(reader: &mut impl Read) -> std::io::Result<Self> {
        Ok(StscItem {
            first_chunk: reader.read_be_u32()?,
            samples_per_chunk: reader.read_be_u32()?,
            sample_description_id: reader.read_be_u32()?,
        })
    }
}

impl Atom for Stbl {
    const FOURCC: Fourcc = SAMPLE_TABLE;
}

impl ParseAtom for Stbl {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut stbl = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE_SAMPLE_DESCRIPTION if cfg.write || cfg.cfg.read_audio_info => {
                    stbl.stsd = Some(Stsd::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_TIME_TO_SAMPLE if cfg.cfg.read_chapter_track => {
                    stbl.stts = Some(Stts::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_SAMPLE_TO_CHUNK if cfg.cfg.read_chapter_track => {
                    stbl.stsc = Some(Stsc::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_SAMPLE_SIZE if cfg.cfg.read_chapter_track => {
                    stbl.stsz = Some(Stsz::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_CHUNK_OFFSET if cfg.write || cfg.cfg.read_chapter_track => {
                    stbl.stco = Some(Stco::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_CHUNK_OFFSET_64 if cfg.write || cfg.cfg.read_chapter_track => {
                    stbl.co64 = Some(Co64::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(stbl)
    }
}

impl AtomSize for Stbl {
    fn size(&self) -> Size {
        let content_len = self.stsd.len_or_zero()
            + self.stts.len_or_zero()
            + self.stsc.len_or_zero()
            + self.stsz.len_or_zero()
            + self.stco.len_or_zero()
            + self.co64.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Stbl {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.stsd {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.stts {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.stsc {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.stsz {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.stco {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.co64 {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Stbl {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.stsd.collect_changes(bounds.end(), level, changes)
            + self.stts.collect_changes(bounds.end(), level, changes)
            + self.stsc.collect_changes(bounds.end(), level, changes)
            + self.stsz.collect_changes(bounds.end(), level, changes)
            + self.stco.collect_changes(bounds.end(), level, changes)
            + self.co64.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stbl(self)
    }
}
