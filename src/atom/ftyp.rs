use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ftyp(pub String);

impl Ftyp {
    pub fn parse(reader: &mut (impl Read + Seek)) -> crate::Result<Self> {
        let head = parse_head(reader)?;
        if head.fourcc() != FILETYPE {
            return Err(crate::Error::new(ErrorKind::NoTag, "No filetype atom found."));
        }

        let ftyp = reader.read_utf8(head.content_len())?;

        Ok(Ftyp(ftyp))
    }

    pub fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        let head = Head::new(false, self.len(), FILETYPE);
        write_head(writer, head)?;
        writer.write_utf8(&self.0)?;
        Ok(())
    }

    pub fn len(&self) -> u64 {
        Head::NORMAL_SIZE + self.0.len() as u64
    }
}
