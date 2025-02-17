use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Dref {
    pub state: State,
    pub url: Option<Url>,
}

impl Atom for Dref {
    const FOURCC: Fourcc = DATA_REFERENCE;
}

impl ParseAtom for Dref {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown data reference (dref) atom version",
            ));
        }

        reader.skip(4)?; // number of entries

        let mut dref = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };
        let mut parsed_bytes = 8;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                URL_MEDIA => dref.url = Some(Url::parse(reader, cfg, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(dref)
    }
}

impl WriteAtom for Dref {
    fn write_atom(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, 0, [0; 3])?;

        if self.url.is_some() {
            writer.write_be_u32(1)?;
        } else {
            writer.write_be_u32(0)?;
        }

        if let Some(a) = &self.url {
            a.write(writer, changes)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = 8 + self.url.len_or_zero();
        Size::from(content_len)
    }
}
