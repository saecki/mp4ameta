//! mp4a atom
//!
//! ```md
//! 4 bytes ?
//! 2 bytes ?
//! 2 bytes data reference index
//! 8 bytes ?
//! 2 bytes channel count
//! 2 bytes sample size
//! 4 bytes ?
//! 4 bytes sample rate
//! │
//! └─ esds atom
//!    4 bytes len
//!    4 bytes ident
//!    1 byte version
//!    3 bytes flags
//!    │
//!    └─ elementary stream descriptor
//!       1 byte tag (0x03)
//!       1~4 bytes len
//!       2 bytes id
//!       1 byte flag
//!       │
//!       ├─ decoder config descriptor
//!       │  1 byte tag (0x04)
//!       │  1~4 bytes len
//!       │  1 byte object type indication
//!       │  1 byte stream type
//!       │  3 bytes buffer size
//!       │  4 bytes maximum bitrate
//!       │  4 bytes average bitrate
//!       │  │
//!       │  └─ decoder specific descriptor
//!       │     1 byte tag (0x05)
//!       │     1~4 bytes len
//!       │     5 bits profile
//!       │     4 bits frequency index
//!       │     4 bits channel config
//!       │     3 bits ?
//!       │
//!       └─ sl config descriptor
//!          1 byte tag (0x06)
//!          1~4 bytes len
//!          1 byte ?
//! ```

use std::cmp::min;

use crate::{ChannelConfig, SampleRate};

use super::*;

/// Es descriptor  tag
const ELEMENTARY_STREAM_DESCRIPTOR: u8 = 0x03;
/// Decoder config descriptor tag
const DECODER_CONFIG_DESCRIPTOR: u8 = 0x04;
/// Decoder specific descriptor tag
const DECODER_SPECIFIC_DESCRIPTOR: u8 = 0x05;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mp4a {
    pub state: State,
    pub channel_config: Option<ChannelConfig>,
    pub sample_rate: Option<SampleRate>,
    pub max_bitrate: Option<u32>,
    pub avg_bitrate: Option<u32>,
}

impl Atom for Mp4a {
    const FOURCC: Fourcc = MP4_AUDIO;
}

impl ParseAtom for Mp4a {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut mp4a = Self::default();

        reader.skip(28)?;

        let head = parse_head(reader)?;
        if head.fourcc() != ELEMENTARY_STREAM_DESCRIPTION {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(ELEMENTARY_STREAM_DESCRIPTION),
                "Missing esds atom",
            ));
        }

        parse_esds(reader, &mut mp4a, head.size())?;

        seek_to_end(reader, &bounds)?;

        mp4a.state = State::Existing(bounds);

        Ok(mp4a)
    }
}

/// esds atom
///
/// ```md
/// 4 bytes len
/// 4 bytes ident
/// 1 byte version
/// 3 bytes flags
/// │
/// └──elementary stream descriptor
///    │
///    ├──decoder config descriptor
///    │  │
///    │  └──decoder specific descriptor
///    │
///    └──sl config descriptor
/// ```
fn parse_esds(reader: &mut (impl Read + Seek), info: &mut Mp4a, size: Size) -> crate::Result<()> {
    let (version, _) = parse_full_head(reader)?;

    if version != 0 {
        return Err(crate::Error::new(
            crate::ErrorKind::UnknownVersion(version),
            "Unknown MPEG-4 audio (mp4a) version",
        ));
    }

    let (tag, head_len, desc_len) = parse_desc_head(reader)?;
    if tag != ELEMENTARY_STREAM_DESCRIPTOR {
        return Err(crate::Error::new(
            crate::ErrorKind::DescriptorNotFound(ELEMENTARY_STREAM_DESCRIPTOR),
            "Missing elementary stream descriptor",
        ));
    }

    let max_len = size.content_len() - 4 - head_len;
    parse_es_desc(reader, info, min(desc_len, max_len))?;

    Ok(())
}

/// elementary stream descriptor
///
/// ```md
/// 1 byte tag (0x03)
/// 1~4 bytes len
/// 2 bytes id
/// 1 byte flag
/// │
/// ├──decoder config descriptor
/// │  │
/// │  └──decoder specific descriptor
/// │
/// └──sl config descriptor
/// ```
fn parse_es_desc(reader: &mut (impl Read + Seek), info: &mut Mp4a, len: u64) -> crate::Result<()> {
    reader.skip(3)?;

    let mut parsed_bytes = 3;
    while parsed_bytes < len {
        let (tag, head_len, desc_len) = parse_desc_head(reader)?;

        match tag {
            DECODER_CONFIG_DESCRIPTOR => parse_dc_desc(reader, info, desc_len)?,
            _ => reader.skip(desc_len as i64)?,
        }

        parsed_bytes += head_len + desc_len;
    }

    Ok(())
}

/// decoder config descriptor
///
/// ```md
/// 1 byte tag (0x04)
/// 1~4 bytes len
/// 1 byte object type indication
/// 1 byte stream type
/// 3 bytes buffer size
/// 4 bytes maximum bitrate
/// 4 bytes average bitrate
/// │
/// └──decoder specific descriptor
/// ```
fn parse_dc_desc(reader: &mut (impl Read + Seek), info: &mut Mp4a, len: u64) -> crate::Result<()> {
    reader.skip(5)?;
    info.max_bitrate = Some(reader.read_be_u32()?);
    info.avg_bitrate = Some(reader.read_be_u32()?);

    let mut parsed_bytes = 13;
    while parsed_bytes < len {
        let (tag, head_len, desc_len) = parse_desc_head(reader)?;

        match tag {
            DECODER_SPECIFIC_DESCRIPTOR => parse_ds_desc(reader, info, desc_len)?,
            _ => {
                reader.skip(desc_len as i64)?;
            }
        }

        parsed_bytes += head_len + desc_len;
    }

    Ok(())
}

/// decoder specific descriptor
///
/// ```md
/// 1 byte tag (0x05)
/// 1~4 bytes len
/// 5 bits profile
/// 4 bits frequency index
/// 4 bits channel config
/// 3 bits ?
/// ```
fn parse_ds_desc(reader: &mut (impl Read + Seek), info: &mut Mp4a, len: u64) -> crate::Result<()> {
    let num = reader.read_be_u16()?;

    let freq_index = ((num >> 7) & 0x0F) as u8;
    info.sample_rate = SampleRate::try_from(freq_index).ok();

    let channel_config = ((num >> 3) & 0x0F) as u8;
    info.channel_config = ChannelConfig::try_from(channel_config).ok();

    reader.skip((len - 2) as i64)?;
    Ok(())
}

fn parse_desc_head(reader: &mut impl Read) -> crate::Result<(u8, u64, u64)> {
    let tag = reader.read_u8()?;

    let mut head_len = 1;
    let mut len = 0;
    while head_len < 5 {
        let b = reader.read_u8()?;
        len = (len << 7) | (b & 0x7F) as u64;
        head_len += 1;
        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((tag, head_len, len))
}
