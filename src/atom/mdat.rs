use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdat;

impl Atom for Mdat {
    const FOURCC: Fourcc = MEDIA_DATA;
}

impl Mdat {
    pub fn read_bounds(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<AtomBounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(bounds)
    }
}
