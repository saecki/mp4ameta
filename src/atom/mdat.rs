use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdat;

impl Atom for Mdat {
    const FOURCC: Fourcc = MEDIA_DATA;
}

pub struct MdatBounds {
    pub bounds: AtomBounds,
}

impl Deref for MdatBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Mdat {
    type Bounds = MdatBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        seek_to_end(reader, &bounds)?;
        Ok(Self::Bounds { bounds })
    }
}
