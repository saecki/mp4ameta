use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ftyp(pub String);

impl Ftyp {
    pub fn parse(reader: &mut (impl Read + Seek)) -> crate::Result<Self> {
        let head = parse_head(reader)?;
        if head.fourcc != FILETYPE {
            return Err(crate::Error::new(ErrorKind::NoTag, "No filetype atom found.".to_owned()));
        }

        let ftyp = read_utf8(reader, head.content_len())?;

        Ok(Ftyp(ftyp))
    }
}
