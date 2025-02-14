use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Moov<'a> {
    pub state: State,
    pub mvhd: Mvhd,
    pub trak: Vec<Trak>,
    pub udta: Option<Udta<'a>>,
}

impl Atom for Moov<'_> {
    const FOURCC: Fourcc = MOVIE;
}

impl ParseAtom for Moov<'_> {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut parsed_bytes = 0;
        let mut mvhd = None;
        let mut trak = Vec::new();
        let mut udta = None;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MOVIE_HEADER => mvhd = Some(Mvhd::parse(reader, cfg, head.size())?),
                TRACK if cfg.write || cfg.cfg.read_chapter_track || cfg.cfg.read_audio_info => {
                    trak.push(Trak::parse(reader, cfg, head.size())?)
                }
                USER_DATA if cfg.cfg.read_item_list || cfg.cfg.read_chapter_list => {
                    udta = Some(Udta::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        let mvhd = mvhd.ok_or_else(|| {
            crate::Error::new(
                ErrorKind::AtomNotFound(MOVIE_HEADER),
                "Missing necessary data, no movie header (mvhd) atom found",
            )
        })?;

        let moov = Self { state: State::Existing(bounds), mvhd, trak, udta };

        Ok(moov)
    }
}

impl WriteAtom for Moov<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        self.mvhd.write(writer)?;
        for t in self.trak.iter() {
            t.write(writer)?;
        }
        if let Some(a) = &self.udta {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.mvhd.len()
            + self.trak.iter().map(Trak::len).sum::<u64>()
            + self.udta.len_or_zero();
        Size::from(content_len)
    }
}

impl SimpleCollectChanges for Moov<'_> {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.trak.iter().map(|a| a.collect_changes(bounds.end(), level, changes)).sum::<i64>()
            + self.udta.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Moov(self)
    }
}
