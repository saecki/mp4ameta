use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ftyp(pub String);

impl Ftyp {
    pub fn parse(reader: &mut (impl Read + Seek)) -> crate::Result<Self> {
        let head = head::parse(reader)?;
        if head.fourcc() != FILETYPE {
            return Err(crate::Error::new(ErrorKind::NoFtyp, "No filetype atom found."));
        }

        let ftyp = reader.read_utf8(head.content_len())?;

        Ok(Ftyp(ftyp))
    }
}
