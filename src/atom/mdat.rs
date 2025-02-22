use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mdat;

impl Atom for Mdat {
    const FOURCC: Fourcc = MEDIA_DATA;
}

impl Mdat {
    pub fn read_bounds(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<AtomBounds> {
        let bounds = find_bounds(reader, size)?;
        reader.seek(SeekFrom::Start(bounds.end()))?;
        Ok(bounds)
    }
}
