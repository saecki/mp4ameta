use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Trak {
    pub tkhd: Option<Tkhd>,
    pub tref: Option<Tref>,
    pub mdia: Option<Mdia>,
}

impl Atom for Trak {
    const FOURCC: Fourcc = TRACK;
}

impl ParseAtom for Trak {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        cfg: &ReadConfig,
        size: Size,
    ) -> crate::Result<Self> {
        let mut trak = Self::default();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                TRACK_HEADER if cfg.read_chapters => {
                    trak.tkhd = Some(Tkhd::parse(reader, cfg, head.size())?)
                }
                TRACK_REFERENCE if cfg.read_chapters => {
                    trak.tref = Some(Tref::parse(reader, cfg, head.size())?)
                }
                MEDIA if cfg.read_chapters || cfg.read_audio_info => {
                    trak.mdia = Some(Mdia::parse(reader, cfg, head.size())?)
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(trak)
    }
}

pub struct TrakBounds {
    pub bounds: AtomBounds,
    pub mdia: Option<MdiaBounds>,
}

impl FindAtom for Trak {
    type Bounds = TrakBounds;

    fn find_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self::Bounds> {
        let bounds = find_bounds(reader, size)?;
        let mut mdia = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                MEDIA => mdia = Some(Mdia::find(reader, head.size())?),
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Bounds { bounds, mdia })
    }
}
