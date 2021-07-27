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
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut moov = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MOVIE_HEADER => moov.mvhd = Some(Mvhd::parse(reader, cfg, head.size())?),
                TRACK if cfg.read_chapters || cfg.read_audio_info => {
                    moov.trak.push(Trak::parse(reader, cfg, head.size())?)
                }
                USER_DATA if cfg.read_item_list || cfg.read_chapters => {
                    moov.udta = Some(Udta::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(moov)
    }
}

impl WriteAtom for Moov<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.mvhd {
            a.write(writer)?;
        }
        for t in self.trak.iter() {
            t.write(writer)?;
        }
        if let Some(a) = &self.udta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.mvhd.len_or_zero()
            + self.trak.iter().map(Trak::len).sum::<u64>()
            + self.udta.len_or_zero();
        Size::from(content_len)
    }
}

#[derive(Default)]
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
        let mut moov = Self::Bounds { bounds, ..Default::default() };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                TRACK => moov.trak.push(Trak::find(reader, head.size())?),
                USER_DATA => moov.udta = Some(Udta::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(moov)
    }
}
