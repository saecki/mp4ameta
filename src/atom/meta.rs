use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Meta<'a> {
    pub hdlr: Option<Hdlr>,
    pub ilst: Option<Ilst<'a>>,
}

impl TempAtom for Meta<'_> {
    const FOURCC: Fourcc = METADATA;
}

impl ParseAtom for Meta<'_> {
    fn parse_atom(reader: &'_ mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let (version, _) = parse_full_head(reader)?;

        if version != 0 {
            return Err(crate::Error::new(
                ErrorKind::UnknownVersion(version),
                "Unknown metadata (meta) version".to_owned(),
            ));
        }

        let mut meta = Self::default();
        let mut parsed_bytes = 4;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                ITEM_LIST => meta.ilst = Some(Ilst::parse(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
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
        let content_len = self.hdlr.as_ref().map_or(0, |a| a.size().len())
            + self.ilst.as_ref().map_or(0, |a| a.size().len());
        Size::from(content_len + 4)
    }
}

impl Meta<'_> {
    pub fn hdlr() -> Hdlr {
        Hdlr(vec![
            0x00, 0x00, 0x00, 0x00, // version + flags
            0x00, 0x00, 0x00, 0x00, // component type
            0x6d, 0x64, 0x69, 0x72, // component subtype
            0x61, 0x70, 0x70, 0x6c, // component manufacturer
            0x00, 0x00, 0x00, 0x00, // component flags
            0x00, 0x00, 0x00, 0x00, // component flags mask
            0x00, // component name
        ])
    }
}
