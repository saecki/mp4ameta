use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdia {
    pub state: State,
    pub mdhd: Option<Mdhd>,
    pub hdlr: Option<Hdlr>,
    pub minf: Option<Minf>,
}

impl Atom for Mdia {
    const FOURCC: Fourcc = MEDIA;
}

impl ParseAtom for Mdia {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut mdia = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA_HEADER if cfg.read_chapters => {
                    mdia.mdhd = Some(Mdhd::parse(reader, cfg, head.size())?)
                }
                HANDLER_REFERENCE => mdia.hdlr = Some(Hdlr::parse(reader, cfg, head.size())?),
                MEDIA_INFORMATION => mdia.minf = Some(Minf::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(mdia)
    }
}

impl WriteAtom for Mdia {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.mdhd {
            a.write(writer)?;
        }
        if let Some(a) = &self.hdlr {
            a.write(writer)?;
        }
        if let Some(a) = &self.minf {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len =
            self.mdhd.len_or_zero() + self.hdlr.len_or_zero() + self.minf.len_or_zero();
        Size::from(content_len)
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
        self.minf.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef {
        AtomRef::Mdia(self)
    }
}
