use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Minf {
    pub state: State,
    pub gmhd: Option<Gmhd>,
    pub dinf: Option<Dinf>,
    pub stbl: Option<Stbl>,
}

impl Atom for Minf {
    const FOURCC: Fourcc = MEDIA_INFORMATION;
}

impl ParseAtom for Minf {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut minf = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                BASE_MEDIA_INFORMATION_HEADER if cfg.write => {
                    minf.gmhd = Some(Gmhd::parse(reader, cfg, head.size())?)
                }
                DATA_INFORMATION if cfg.write => {
                    minf.dinf = Some(Dinf::parse(reader, cfg, head.size())?)
                }
                SAMPLE_TABLE => minf.stbl = Some(Stbl::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(minf)
    }
}

impl WriteAtom for Minf {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        if let Some(a) = &self.gmhd {
            a.write(writer)?;
        }
        if let Some(a) = &self.dinf {
            a.write(writer)?;
        }
        if let Some(a) = &self.stbl {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len =
            self.gmhd.len_or_zero() + self.dinf.len_or_zero() + self.stbl.len_or_zero();
        Size::from(content_len)
    }
}

impl SimpleCollectChanges for Minf {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        // TODO: check other child atoms
        self.stbl.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Minf(self)
    }
}
