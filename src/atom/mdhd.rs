use super::*;

pub const HEADER_SIZE_V0: usize = 24;
pub const HEADER_SIZE_V1: usize = 36;
const BUF_SIZE_V0: usize = HEADER_SIZE_V0 - 4;
const BUF_SIZE_V1: usize = HEADER_SIZE_V1 - 4;

const_assert!(std::mem::size_of::<MdhdBufV0>() == BUF_SIZE_V0);
const_assert!(std::mem::size_of::<MdhdBufV1>() == BUF_SIZE_V1);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mdhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub timescale: u32,
    pub duration: u64,
}

#[derive(Default)]
#[repr(C)]
struct MdhdBufV0 {
    creation_time: [u8; 4],
    modification_time: [u8; 4],
    timescale: [u8; 4],
    duration: [u8; 4],
    language: [u8; 2],
    quality: [u8; 2],
}

#[derive(Default)]
#[repr(C)]
struct MdhdBufV1 {
    creation_time: [u8; 8],
    modification_time: [u8; 8],
    timescale: [u8; 4],
    duration: [u8; 8],
    language: [u8; 2],
    quality: [u8; 2],
}

impl Atom for Mdhd {
    const FOURCC: Fourcc = MEDIA_HEADER;
}

impl ParseAtom for Mdhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut mdhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        mdhd.version = version;
        mdhd.flags = flags;

        match version {
            0 => {
                let mut buf = MdhdBufV0::default();

                // SAFETY: alignment and size match because all fields are byte arrays
                let byte_buf: &mut [u8; BUF_SIZE_V0] = unsafe { std::mem::transmute(&mut buf) };
                reader.read_exact(byte_buf)?;

                mdhd.timescale = u32::from_be_bytes(buf.timescale);
                mdhd.duration = u32::from_be_bytes(buf.duration) as u64;
            }
            1 => {
                let mut buf = MdhdBufV1::default();

                // SAFETY: alignment and size match because all fields are byte arrays
                let byte_buf: &mut [u8; BUF_SIZE_V1] = unsafe { std::mem::transmute(&mut buf) };
                reader.read_exact(byte_buf)?;

                mdhd.timescale = u32::from_be_bytes(buf.timescale);
                mdhd.duration = u64::from_be_bytes(buf.duration);
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown media header (mdhd) version {v}"),
                ));
            }
        }

        Ok(mdhd)
    }
}

impl AtomSize for Mdhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(HEADER_SIZE_V0 as u64),
            1 => Size::from(HEADER_SIZE_V1 as u64),
            _ => Size::from(0),
        }
    }
}

impl WriteAtom for Mdhd {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                let mut buf = MdhdBufV0::default();
                buf.timescale = u32::to_be_bytes(self.timescale);
                buf.duration = u32::to_be_bytes(self.duration as u32);

                // SAFETY: alignment and size match because all fields are byte arrays
                let byte_buf: &[u8; BUF_SIZE_V0] = unsafe { std::mem::transmute(&buf) };
                writer.write_all(byte_buf)?;
            }
            1 => {
                let mut buf = MdhdBufV1::default();
                buf.timescale = u32::to_be_bytes(self.timescale);
                buf.duration = u64::to_be_bytes(self.duration);

                // SAFETY: alignment and size match because all fields are byte arrays
                let byte_buf: &[u8; BUF_SIZE_V1] = unsafe { std::mem::transmute(&buf) };
                writer.write_all(byte_buf)?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown media header (mdhd) version {v}"),
                ));
            }
        }

        Ok(())
    }
}
