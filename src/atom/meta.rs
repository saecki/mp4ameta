use super::*;

pub const HEADER_SIZE: u64 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta<'a> {
    pub state: State,
    pub hdlr: Option<Hdlr>,
    pub ilst: Option<Ilst<'a>>,
}

impl Atom for Meta<'_> {
    const FOURCC: Fourcc = METADATA;
}

impl ParseAtom for Meta<'_> {
    fn parse_atom(
        reader: &'_ mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown metadata (meta) version",
            ));
        }

        let mut meta = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = HEADER_SIZE;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                HANDLER_REFERENCE if cfg.write => {
                    meta.hdlr = Some(Hdlr::parse(reader, cfg, head.size())?)
                }
                ITEM_LIST => meta.ilst = Some(Ilst::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(meta)
    }
}

impl AtomSize for Meta<'_> {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + self.hdlr.len_or_zero() + self.ilst.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Meta<'_> {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;
        if let Some(a) = &self.hdlr {
            a.write(writer, changes)?;
        }
        if let Some(a) = &self.ilst {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl SimpleCollectChanges for Meta<'_> {
    fn state(&self) -> &State {
        &self.state
    }

    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.hdlr.collect_changes(bounds.content_pos() + HEADER_SIZE, level, changes)
            + self.ilst.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Meta(self)
    }
}
