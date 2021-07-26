use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tref {
    pub chap: Option<Chap>,
}

impl Atom for Tref {
    const FOURCC: Fourcc = TRACK_REFERENCE;
}

impl ParseAtom for Tref {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut tref = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                CHAPTER => tref.chap = Some(Chap::parse(reader, cfg, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(tref)
    }
}
