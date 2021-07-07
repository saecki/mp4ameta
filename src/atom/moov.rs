use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Moov {
    pub mvhd: Option<Mvhd>,
    pub trak: Vec<Trak>,
    pub udta: Option<Udta>,
}

impl ParseAtom for Moov {
    const FOURCC: Fourcc = MOVIE;

    fn parse_atom(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let mut moov = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc {
                MOVIE_HEADER => moov.mvhd = Some(Mvhd::parse(reader, head.content_len())?),
                TRACK => moov.trak.push(Trak::parse(reader, head.content_len())?),
                USER_DATA => moov.udta = Some(Udta::parse(reader, head.content_len())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len;
        }

        Ok(moov)
    }
}
