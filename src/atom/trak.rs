use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Trak {
    pub mdia: Option<Mdia>,
}

impl TempAtom for Trak {
    const FOURCC: Fourcc = TRACK;
}

impl ParseAtom for Trak {
    fn parse_atom(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let mut trak = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA => trak.mdia = Some(Mdia::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(trak)
    }
}
