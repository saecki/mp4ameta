use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Moov<'a> {
    pub mvhd: Option<Mvhd>,
    pub trak: Vec<Trak>,
    pub udta: Option<Udta<'a>>,
}

impl Atom for Moov<'_> {
    const FOURCC: Fourcc = MOVIE;
}

impl ParseAtom for Moov<'_> {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut moov = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MOVIE_HEADER => moov.mvhd = Some(Mvhd::parse(reader, head.size())?),
                TRACK => moov.trak.push(Trak::parse(reader, head.size())?),
                USER_DATA => moov.udta = Some(Udta::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(moov)
    }
}

impl WriteAtom for Moov<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.udta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(self.udta.len_or_zero())
    }
}

pub struct MoovBounds {
    pub bounds: AtomBounds,
    pub trak: Vec<TrakBounds>,
    pub udta: Option<UdtaBounds>,
}

impl Deref for MoovBounds {
    type Target = AtomBounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}

impl FindAtom for Moov<'_> {
    type Bounds = MoovBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut trak = Vec::new();
        let mut udta = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                TRACK => trak.push(Trak::find(reader, head.size())?),
                USER_DATA => udta = Some(Udta::find(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Bounds { bounds, trak, udta })
    }
}
