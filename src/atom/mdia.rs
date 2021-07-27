use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdia {
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
        let mut mdia = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA_HEADER if cfg.read_chapters => {
                    mdia.mdhd = Some(Mdhd::parse(reader, cfg, head.size())?)
                }
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

pub struct MdiaBounds {
    pub bounds: AtomBounds,
    pub minf: Option<MinfBounds>,
}

impl FindAtom for Mdia {
    type Bounds = MdiaBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut minf = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA_INFORMATION => minf = Some(Minf::find(reader, head.size())?),
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Bounds { bounds, minf })
    }
}
