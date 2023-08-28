use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = parse_full_head(reader)?;

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
        let mut parsed_bytes = 4;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                ITEM_LIST => meta.ilst = Some(Ilst::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(meta)
    }
}

impl WriteAtom for Meta<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;
        if let Some(a) = &self.hdlr {
            a.write(writer)?;
        }
        if let Some(a) = &self.ilst {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.hdlr.len_or_zero() + self.ilst.len_or_zero();
        Size::from(content_len + 4)
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
        self.ilst.collect_changes(bounds.end(), level, changes)
            + self.hdlr.collect_changes(bounds.end(), level, changes)
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Meta(self)
    }
}
