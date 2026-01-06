use super::*;

pub const HEADER_SIZE_V0: usize = 100;
pub const HEADER_SIZE_V1: usize = 112;
const BUF_SIZE_V0: usize = HEADER_SIZE_V0 - 4;
const BUF_SIZE_V1: usize = HEADER_SIZE_V1 - 4;

const_assert!(std::mem::size_of::<MvhdBufV0>() == BUF_SIZE_V0);
const_assert!(std::mem::size_of::<MvhdBufV1>() == BUF_SIZE_V1);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mvhd {
    pub version: u8,
    pub flags: Flags,
    pub timescale: u32,
    pub duration: u64,
}

#[derive(Default)]
#[repr(C)]
struct MvhdBufV0 {
    creation_time: [u8; 4],
    modification_time: [u8; 4],
    timescale: [u8; 4],
    duration: [u8; 4],
    preferred_rate: [u8; 4],
    preferred_volume: [u8; 2],
    reserved: [u8; 10],
    matrix: [[[u8; 4]; 3]; 3],
    preview_time: [u8; 4],
    preview_duration: [u8; 4],
    poster_time: [u8; 4],
    selection_time: [u8; 4],
    selection_duration: [u8; 4],
    current_time: [u8; 4],
    next_track_id: [u8; 4],
}

impl MvhdBufV0 {
    fn bytes_mut(&mut self) -> &mut [u8; BUF_SIZE_V0] {
        // SAFETY: alignment and size match because all fields are byte arrays
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Default)]
#[repr(C)]
struct MvhdBufV1 {
    creation_time: [u8; 8],
    modification_time: [u8; 8],
    timescale: [u8; 4],
    duration: [u8; 8],
    preferred_rate: [u8; 4],
    preferred_volume: [u8; 2],
    reserved: [u8; 10],
    matrix: [[[u8; 4]; 3]; 3],
    preview_time: [u8; 4],
    preview_duration: [u8; 4],
    poster_time: [u8; 4],
    selection_time: [u8; 4],
    selection_duration: [u8; 4],
    current_time: [u8; 4],
    next_track_id: [u8; 4],
}

impl MvhdBufV1 {
    fn bytes_mut(&mut self) -> &mut [u8; BUF_SIZE_V1] {
        // SAFETY: alignment and size match because all fields are byte arrays
        unsafe { std::mem::transmute(self) }
    }
}

impl Atom for Mvhd {
    const FOURCC: Fourcc = MOVIE_HEADER;
}

impl ParseAtom for Mvhd {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let mut mvhd = Self::default();

        let (version, flags) = head::parse_full(reader)?;
        mvhd.version = version;
        mvhd.flags = flags;

        match version {
            0 => {
                expect_size("Movie header (mvhd) version 0", size, HEADER_SIZE_V0 as u64)?;

                let mut buf = MvhdBufV0::default();
                reader.read_exact(buf.bytes_mut())?;
                mvhd.timescale = u32::from_be_bytes(buf.timescale);
                mvhd.duration = u32::from_be_bytes(buf.duration) as u64;
            }
            1 => {
                expect_size("Movie header (mvhd) version 1", size, HEADER_SIZE_V1 as u64)?;

                let mut buf = MvhdBufV1::default();
                reader.read_exact(buf.bytes_mut())?;
                mvhd.timescale = u32::from_be_bytes(buf.timescale);
                mvhd.duration = u64::from_be_bytes(buf.duration);
            }
            _ => {
                return unknown_version("movie header (mvhd)", version);
            }
        }

        Ok(mvhd)
    }
}

impl AtomSize for Mvhd {
    fn size(&self) -> Size {
        match self.version {
            0 => Size::from(HEADER_SIZE_V0 as u64),
            1 => Size::from(HEADER_SIZE_V1 as u64),
            _ => Size::from(0),
        }
    }
}
