use super::*;

pub const HEADER_SIZE_V0: usize = 84;
pub const HEADER_SIZE_V1: usize = 96;
const BUF_SIZE_V0: usize = HEADER_SIZE_V0 - 4;
const BUF_SIZE_V1: usize = HEADER_SIZE_V1 - 4;

const_assert!(std::mem::size_of::<TkhdBufV0>() == BUF_SIZE_V0);
const_assert!(std::mem::size_of::<TkhdBufV1>() == BUF_SIZE_V1);

const MATRIX: [[[u8; 4]; 3]; 3] = [
    [u32::to_be_bytes(1 << 16), u32::to_be_bytes(0), u32::to_be_bytes(0)],
    [u32::to_be_bytes(0), u32::to_be_bytes(1 << 16), u32::to_be_bytes(0)],
    [u32::to_be_bytes(0), u32::to_be_bytes(0), u32::to_be_bytes(1 << 30)],
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tkhd {
    pub version: u8,
    pub flags: [u8; 3],
    pub id: u32,
    /// The duration in mvhd timescale units
    pub duration: u64,
}

#[derive(Default)]
#[repr(C)]
struct TkhdBufV0 {
    creation_time: [u8; 4],
    modification_time: [u8; 4],
    id: [u8; 4],
    reserved0: [u8; 4],
    duration: [u8; 4],
    reserved1: [u8; 8],
    layer: [u8; 2],
    alternate_group: [u8; 2],
    volume: [u8; 2],
    reserved2: [u8; 2],
    matrix: [[[u8; 4]; 3]; 3],
    track_width: [u8; 4],
    track_height: [u8; 4],
}

impl TkhdBufV0 {
    fn bytes_mut(&mut self) -> &mut [u8; BUF_SIZE_V0] {
        // SAFETY: alignment and size match because all fields are byte arrays
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Default)]
#[repr(C)]
struct TkhdBufV1 {
    creation_time: [u8; 8],
    modification_time: [u8; 8],
    id: [u8; 4],
    reserved0: [u8; 4],
    duration: [u8; 8],
    reserved1: [u8; 8],
    layer: [u8; 2],
    alternate_group: [u8; 2],
    volume: [u8; 2],
    reserved2: [u8; 2],
    matrix: [[[u8; 4]; 3]; 3],
    track_width: [u8; 4],
    track_height: [u8; 4],
}

impl TkhdBufV1 {
    fn bytes_mut(&mut self) -> &mut [u8; BUF_SIZE_V1] {
        // SAFETY: alignment and size match because all fields are byte arrays
        unsafe { std::mem::transmute(self) }
    }
}

impl Atom for Tkhd {
    const FOURCC: Fourcc = TRACK_HEADER;
}

impl ParseAtom for Tkhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        _size: Size,
    ) -> crate::Result<Self> {
        let mut tkhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        tkhd.version = version;
        tkhd.flags = flags;

        match version {
            0 => {
                let mut buf = TkhdBufV0::default();
                reader.read_exact(buf.bytes_mut())?;
                tkhd.id = u32::from_be_bytes(buf.id);
                tkhd.duration = u32::from_be_bytes(buf.duration) as u64;
            }
            1 => {
                let mut buf = TkhdBufV1::default();
                reader.read_exact(buf.bytes_mut())?;
                tkhd.id = u32::from_be_bytes(buf.id);
                tkhd.duration = u64::from_be_bytes(buf.duration);
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Unknown track header (tkhd) version {v}"),
                ));
            }
        }

        Ok(tkhd)
    }
}

impl AtomSize for Tkhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(HEADER_SIZE_V0 as u64),
            1 => Size::from(HEADER_SIZE_V1 as u64),
            _ => Size::from(0),
        }
    }
}

impl WriteAtom for Tkhd {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        head::write_full(writer, self.version, self.flags)?;

        match self.version {
            0 => {
                let mut buf = TkhdBufV0::default();
                buf.id = u32::to_be_bytes(self.id);
                buf.duration = u32::to_be_bytes(self.duration as u32);
                buf.matrix = MATRIX;
                writer.write_all(buf.bytes_mut())?;
            }
            1 => {
                let mut buf = TkhdBufV1::default();
                buf.id = u32::to_be_bytes(self.id);
                buf.duration = u64::to_be_bytes(self.duration);
                buf.matrix = MATRIX;
                writer.write_all(buf.bytes_mut())?;
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(self.version),
                    format!("Unknown track header (tkhd) version {v}"),
                ));
            }
        }

        Ok(())
    }
}
