use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stbl {
    pub stsd: Option<Stsd>,
    pub stts: Option<Stts>,
    pub stsc: Option<Stsc>,
    pub stsz: Option<Stsz>,
    pub stco: Option<Stco>,
    pub co64: Option<Co64>,
}

impl Atom for Stbl {
    const FOURCC: Fourcc = SAMPLE_TABLE;
}

impl ParseAtom for Stbl {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut stbl = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE_SAMPLE_DESCRIPTION if cfg.read_audio_info => {
                    stbl.stsd = Some(Stsd::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_TIME_TO_SAMPLE if cfg.read_chapters => {
                    stbl.stts = Some(Stts::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_CHUNK_OFFSET if cfg.read_chapters => {
                    stbl.stco = Some(Stco::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE_CHUNK_OFFSET_64 if cfg.read_chapters => {
                    stbl.co64 = Some(Co64::parse(reader, cfg, head.size())?)
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(stbl)
    }
}

#[derive(Default)]
pub struct StblBounds {
    pub bounds: AtomBounds,
    pub stts: Option<SttsBounds>,
    pub stsc: Option<StscBounds>,
    pub stsz: Option<StszBounds>,
    pub stco: Option<StcoBounds>,
    pub co64: Option<Co64Bounds>,
}

impl FindAtom for Stbl {
    type Bounds = StblBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut stbl = Self::Bounds { bounds, ..Default::default() };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE_TIME_TO_SAMPLE => stbl.stts = Some(Stts::find(reader, head.size())?),
                SAMPLE_TABLE_SAMPLE_TO_COUNT => stbl.stsc = Some(Stsc::find(reader, head.size())?),
                SAMPLE_TABLE_SAMPLE_SIZE => stbl.stsz = Some(Stsz::find(reader, head.size())?),
                SAMPLE_TABLE_CHUNK_OFFSET => stbl.stco = Some(Stco::find(reader, head.size())?),
                SAMPLE_TABLE_CHUNK_OFFSET_64 => stbl.co64 = Some(Co64::find(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(stbl)
    }
}
