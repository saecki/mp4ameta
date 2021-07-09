use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdia {
    pub minf: Option<Minf>,
}

impl TempAtom for Mdia {
    const FOURCC: Fourcc = MEDIA;
}

impl ParseAtom for Mdia {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut mdia = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA_INFORMATION => mdia.minf = Some(Minf::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(mdia)
    }
}
