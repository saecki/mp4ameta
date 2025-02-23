use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mdia {
    pub state: State,
    pub mdhd: Mdhd,
    pub hdlr: Option<Hdlr>,
    pub minf: Option<Minf>,
}

impl Atom for Mdia {
    const FOURCC: Fourcc = MEDIA;
}

impl ParseAtom for Mdia {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut parsed_bytes = 0;
        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;

        while parsed_bytes < size.content_len() {
            let remaining_bytes = size.content_len() - parsed_bytes;
            let head = head::parse(reader, remaining_bytes)?;

            match head.fourcc() {
                MEDIA_HEADER => mdhd = Some(Mdhd::parse(reader, cfg, head.size())?),
                HANDLER_REFERENCE if cfg.write => {
                    hdlr = Some(Hdlr::parse(reader, cfg, head.size())?)
                }
                MEDIA_INFORMATION => minf = Some(Minf::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        let mdhd = mdhd.ok_or_else(|| {
            crate::Error::new(
                ErrorKind::AtomNotFound(MEDIA_HEADER),
                "Missing necessary data, no media header (mdhd) atom found",
            )
        })?;

        let mdia = Self { state: State::Existing(bounds), mdhd, hdlr, minf };

        Ok(mdia)
    }
}

impl AtomSize for Mdia {
    fn size(&self) -> Size {
        let content_len = self.mdhd.len() + self.hdlr.len_or_zero() + self.minf.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Mdia {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        self.mdhd.write(writer, changes)?;
        if let Some(a) = &self.hdlr {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.minf {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Mdia {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.hdlr.collect_changes(bounds.end(), level, changes)
            + self.minf.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Mdia(self)
    }
}
