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
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut stbl = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE_SAMPLE_DESCRIPTION => {
                    stbl.stsd = Some(Stsd::parse(reader, head.size())?)
                }
                SAMPLE_TABLE_TIME_TO_SAMPLE => stbl.stts = Some(Stts::parse(reader, head.size())?),
                SAMPLE_TABLE_SAMPLE_TO_COUNT => stbl.stsc = Some(Stsc::parse(reader, head.size())?),
                SAMPLE_TABLE_SAMPLE_SIZE => stbl.stsz = Some(Stsz::parse(reader, head.size())?),
                SAMPLE_TABLE_CHUNK_OFFSET => stbl.stco = Some(Stco::parse(reader, head.size())?),
                SAMPLE_TABLE_CHUNK_OFFSET_64 => stbl.co64 = Some(Co64::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(stbl)
    }
}

pub struct StblBounds {
    pub bounds: AtomBounds,
    pub stco: Option<StcoBounds>,
    pub co64: Option<Co64Bounds>,
}

impl FindAtom for Stbl {
    type Bounds = StblBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut stco = None;
        let mut co64 = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE_CHUNK_OFFSET => stco = Some(Stco::find(reader, head.size())?),
                SAMPLE_TABLE_CHUNK_OFFSET_64 => co64 = Some(Co64::find(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Bounds { bounds, stco, co64 })
    }
}
