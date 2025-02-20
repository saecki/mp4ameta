use super::*;

pub const HEADER_SIZE: u64 = 8;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stsd {
    pub state: State,
    pub mp4a: Option<Mp4a>,
    pub text: Option<Text>,
}

impl Atom for Stsd {
    const FOURCC: Fourcc = SAMPLE_TABLE_SAMPLE_DESCRIPTION;
}

impl ParseAtom for Stsd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = head::parse_full(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown sample table sample description (stsd) version",
            ));
        }

        reader.skip(4)?; // number of entries

        let mut stsd = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = HEADER_SIZE;

        while parsed_bytes < size.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                MP4_AUDIO if !cfg.write => stsd.mp4a = Some(Mp4a::parse(reader, cfg, head.size())?),
                TEXT_MEDIA if cfg.write => stsd.text = Some(Text::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(stsd)
    }
}

impl AtomSize for Stsd {
    fn size(&self) -> Size {
        let content_len = HEADER_SIZE + self.text.len_or_zero();
        Size::from(content_len)
    }
}

impl WriteAtom for Stsd {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, 0, [0; 3])?;

        if self.text.is_some() {
            writer.write_be_u32(1)?;
        } else {
            writer.write_be_u32(0)?;
        }

        if let Some(a) = &self.text {
            a.write(writer, changes)?;
        }
        Ok(())
    }
}

impl LeafAtomCollectChanges for Stsd {
    fn state(&self) -> &State {
        &self.state
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        AtomRef::Stsd(self)
    }
}
