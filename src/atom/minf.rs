use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Minf {
    pub stbl: Option<Stbl>,
}

impl Atom for Minf {
    const FOURCC: Fourcc = MEDIA_INFORMATION;
}

impl ParseAtom for Minf {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut minf = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE => minf.stbl = Some(Stbl::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(minf)
    }
}

pub struct MinfBounds {
    pub bounds: AtomBounds,
    pub stbl: Option<StblBounds>,
}

impl FindAtom for Minf {
    type Bounds = MinfBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut stbl = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                SAMPLE_TABLE => stbl = Some(Stbl::find(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Bounds { bounds, stbl })
    }
}
