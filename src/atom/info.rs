use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use super::*;
use crate::{ChannelConfig, SampleRate};

/// Es descriptor  tag
const ELEMENTARY_STREAM_DESCRIPTOR: u8 = 0x03;
/// Decoder config descriptor tag
const DECODER_CONFIG_DESCRIPTOR: u8 = 0x04;
/// Decoder specific descriptor tag
const DECODER_SPECIFIC_DESCRIPTOR: u8 = 0x05;

/// A struct containing information about an MPEG-4 AAC track.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Mp4aInfo {
    /// The channel configuration of the track.
    pub channel_config: Option<ChannelConfig>,
    /// The sample rate of the track.
    pub sample_rate: Option<SampleRate>,
    /// The maximum bitrate of the track.
    pub max_bitrate: Option<u32>,
    /// The average bitrate of the track.
    pub avg_bitrate: Option<u32>,
}

// mp4a atom
// 4 bytes ?
// 2 bytes ?
// 2 bytes data reference index
// 8 bytes ?
// 2 bytes channel count
// 2 bytes sample size
// 4 bytes ?
// 4 bytes sample rate
//
//   esds atom
//   4 bytes len
//   4 bytes ident
//   1 byte version
//   3 bytes flags
//
//     elementary stream descriptor
//     1 byte tag (0x03)
//     1~4 bytes len
//     2 bytes id
//     1 byte flag
//
//       decoder config descriptor
//       1 byte tag (0x04)
//       1~4 bytes len
//       1 byte object type indication
//       1 byte stream type
//       3 bytes buffer size
//       4 bytes maximum bitrate
//       4 bytes average bitrate
//
//         decoder specific descriptor
//         1 byte tag (0x05)
//         1~4 bytes len
//         5 bits profile
//         4 bits frequency index
//         4 bits channel config
//         3 bits ?
//
//       sl config descriptor
//       1 byte tag (0x06)
//       1~4 bytes len
//       1 byte ?

impl Mp4aInfo {
    /// Attempts to parse audio information from the mp4 audio sample entry.
    pub(super) fn parse(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let mut info = Self::default();

        let start_pos = reader.seek(SeekFrom::Current(0))?;

        reader.seek(SeekFrom::Current(28))?;

        let head = parse_head(reader)?;
        if head.ident != ELEMENTARY_STREAM_DESCRIPTION {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(ELEMENTARY_STREAM_DESCRIPTION),
                "Missing esds atom".to_owned(),
            ));
        }

        parse_esds(reader, &mut info, head.content_len())?;

        let current_pos = reader.seek(SeekFrom::Current(0))?;
        let diff = current_pos - start_pos;
        reader.seek(SeekFrom::Current(len as i64 - diff as i64))?;

        Ok(info)
    }
}

fn parse_esds(reader: &mut (impl Read + Seek), info: &mut Mp4aInfo, len: u64) -> crate::Result<()> {
    let (version, _) = parse_ext_head(reader)?;

    if version != 0 {
        return Err(crate::Error::new(
            crate::ErrorKind::UnknownVersion(version),
            "Unknown MPEG-4 audio (mp4a) version".to_owned(),
        ));
    }

    let (tag, head_len, _) = parse_desc_head(reader)?;
    if tag != ELEMENTARY_STREAM_DESCRIPTOR {
        return Err(crate::Error::new(
            crate::ErrorKind::DescriptorNotFound(ELEMENTARY_STREAM_DESCRIPTOR),
            "Missing elementary stream descriptor".to_owned(),
        ));
    }

    parse_es_desc(reader, info, len - 4 - head_len)?;

    Ok(())
}

fn parse_es_desc(
    reader: &mut (impl Read + Seek),
    info: &mut Mp4aInfo,
    len: u64,
) -> crate::Result<()> {
    reader.seek(SeekFrom::Current(3))?;

    let mut pos = 3;
    while pos < len {
        let (tag, head_len, len) = parse_desc_head(reader)?;
        match tag {
            DECODER_CONFIG_DESCRIPTOR => {
                parse_dc_desc(reader, info)?;
            }
            _ => {
                reader.seek(SeekFrom::Current(len as i64))?;
            }
        }

        pos += head_len + len;
    }

    Ok(())
}

fn parse_dc_desc(reader: &mut (impl Read + Seek), audio_info: &mut Mp4aInfo) -> crate::Result<()> {
    reader.seek(SeekFrom::Current(5))?;
    audio_info.max_bitrate = Some(data::read_u32(reader)?);
    audio_info.avg_bitrate = Some(data::read_u32(reader)?);

    let (tag, _, _) = parse_desc_head(reader)?;
    if tag != DECODER_SPECIFIC_DESCRIPTOR {
        return Err(crate::Error::new(
            crate::ErrorKind::DescriptorNotFound(DECODER_SPECIFIC_DESCRIPTOR),
            "Missing decoder specific descriptor".to_owned(),
        ));
    }
    parse_ds_desc(reader, audio_info)?;

    Ok(())
}

fn parse_ds_desc(reader: &mut (impl Read + Seek), audio_info: &mut Mp4aInfo) -> crate::Result<()> {
    let num = data::read_u16(reader)?;

    let freq_index = ((num >> 7) & 0x0F) as u8;
    audio_info.sample_rate = SampleRate::try_from(freq_index).ok();

    let channel_config = ((num >> 3) & 0x0F) as u8;
    audio_info.channel_config = ChannelConfig::try_from(channel_config).ok();

    Ok(())
}

fn parse_desc_head(reader: &mut impl Read) -> crate::Result<(u8, u64, u64)> {
    let tag = data::read_u8(reader)?;

    let mut head_len = 1;
    let mut len = 0;
    while head_len < 5 {
        let b = data::read_u8(reader)?;
        len = (len << 7) | (b & 0x7F) as u64;
        head_len += 1;
        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((tag, head_len, len))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct MvhdInfo {
    /// The duration of the track.
    pub duration: Option<Duration>,
}

impl MvhdInfo {
    pub(super) fn parse(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let mut info = Self::default();

        let start_pos = reader.seek(SeekFrom::Current(0))?;

        let (version, _) = parse_ext_head(reader)?;
        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes motification time
                // 4 bytes time scale
                // 4 bytes duration
                // ...
                reader.seek(SeekFrom::Current(8))?;
                let timescale = read_u32(reader)? as u64;
                let duration = read_u32(reader)? as u64;

                info.duration = Some(Duration::from_nanos(duration * 1_000_000_000 / timescale));
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes motification time
                // 4 bytes time scale
                // 8 bytes duration
                // ...
                reader.seek(SeekFrom::Current(16))?;
                let timescale = read_u32(reader)? as u64;
                let duration = read_u64(reader)?;

                info.duration = Some(Duration::from_nanos(duration * 1_000_000_000 / timescale));
            }
            v => {
                return Err(crate::Error::new(
                    crate::ErrorKind::UnknownVersion(version),
                    format!("Error unknown movie header (mvhd) version {}", v),
                ))
            }
        }

        let current_pos = reader.seek(SeekFrom::Current(0))?;
        let diff = current_pos - start_pos;
        reader.seek(SeekFrom::Current(len as i64 - diff as i64))?;

        Ok(info)
    }
}

/// A struct representing of a sample table chunk offset atom (`stco`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ChunkOffsetInfo {
    pub table_pos: u64,
    pub offsets: Vec<u32>,
}

impl ChunkOffsetInfo {
    pub(super) fn parse(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let (version, _) = parse_ext_head(reader)?;

        match version {
            0 => {
                let entries = data::read_u32(reader)?;
                if 8 + 4 * entries as u64 != len {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        "Sample table chunk offset (stco) offset table size doesn't match atom length".to_owned(),
                    ));
                }

                let table_pos = reader.seek(SeekFrom::Current(0))?;
                let mut offsets = Vec::with_capacity(entries as usize);
                for _ in 0..entries {
                    let offset = data::read_u32(reader)?;
                    offsets.push(offset);
                }

                Ok(Self { table_pos, offsets })
            }
            _ => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown sample table chunk offset (stco) version".to_owned(),
            )),
        }
    }
}

/// A struct representing of a 64bit sample table chunk offset atom (`co64`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ChunkOffsetInfo64 {
    pub table_pos: u64,
    pub offsets: Vec<u64>,
}

impl ChunkOffsetInfo64 {
    pub(super) fn parse(reader: &mut (impl Read + Seek), len: u64) -> crate::Result<Self> {
        let (version, _) = parse_ext_head(reader)?;

        match version {
            0 => {
                let entries = data::read_u32(reader)?;
                if 8 + 8 * entries as u64 != len {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        "Sample table chunk offset 64 (co64) offset table size doesn't match atom length".to_owned(),
                    ));
                }

                let table_pos = reader.seek(SeekFrom::Current(0))?;
                let mut offsets = Vec::with_capacity(entries as usize);
                for _ in 0..entries {
                    let offset = data::read_u64(reader)?;
                    offsets.push(offset);
                }

                Ok(Self { table_pos, offsets })
            }
            _ => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                "Unknown 64bit sample table chunk offset (co64) version".to_owned(),
            )),
        }
    }
}
