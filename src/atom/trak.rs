use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trak {
    pub state: State,
    pub tkhd: Tkhd,
    pub tref: Option<Tref>,
    pub mdia: Option<Mdia>,
}

impl Atom for Trak {
    const FOURCC: Fourcc = TRACK;
}

impl ParseAtom for Trak {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut parsed_bytes = 0;
        let mut tkhd = None;
        let mut tref = None;
        let mut mdia = None;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                TRACK_HEADER => tkhd = Some(Tkhd::parse(reader, cfg, head.size())?),
                TRACK_REFERENCE if cfg.cfg.read_chapter_track => {
                    tref = Some(Tref::parse(reader, cfg, head.size())?)
                }
                MEDIA if cfg.write || cfg.cfg.read_chapter_track || cfg.cfg.read_audio_info => {
                    mdia = Some(Mdia::parse(reader, cfg, head.size())?)
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        let tkhd = tkhd.ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::AtomNotFound(TRACK_HEADER),
                "Missing necessary data, no track header (tkhd) atom found",
            )
        })?;

        let trak = Self { state: State::Existing(bounds), tkhd, tref, mdia };

        Ok(trak)
    }
}

impl AtomSize for Trak {
    fn size(&self) -> Size {
        let content_len = self.tkhd.len() + self.tref.len_or_zero() + self.mdia.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Trak {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        self.tkhd.write(writer, changes)?;
        if let Some(a) = &self.tref {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.mdia {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Trak {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.tref.collect_changes(bounds.end(), level, changes)
            + self.mdia.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Trak(self)
    }
}
