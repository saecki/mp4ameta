use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ftyp {
    pub size: Size,
    pub string: String,
}

impl Ftyp {
    pub fn parse(reader: &mut (impl Read + Seek), file_len: u64) -> crate::Result<Self> {
        let head = head::parse(reader, file_len)?;
        if head.fourcc() != FILETYPE {
            return Err(crate::Error::new(ErrorKind::NoFtyp, "No filetype atom found."));
        }

        let string = reader.read_utf8(head.content_len())?;

        Ok(Ftyp { size: head.size(), string })
    }
}
