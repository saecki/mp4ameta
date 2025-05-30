use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
            let remaining_bytes = size.content_len() - parsed_bytes;
            let head = head::parse(reader, remaining_bytes)?;

            match head.fourcc() {
                MOVIE_HEADER => mvhd = Some(Mvhd::parse(reader, cfg, head.size())?),
                TRACK if cfg.write || cfg.cfg.read_chapter_track || cfg.cfg.read_audio_info => {
                    trak.push(Trak::parse(reader, cfg, head.size())?)
                }
                USER_DATA if cfg.cfg.read_meta_items || cfg.cfg.read_chapter_list => {
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

impl AtomSize for Moov<'_> {
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
