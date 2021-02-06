use std::io::{Read, Seek, SeekFrom};

use super::*;
use crate::{ChannelConfig, SampleRate};

/// A struct containing information about an MPEG-4 AAC track.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioInfo {
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

/// Attempts to parse audio information from the mp4 audio sample entry.
pub(crate) fn parse_mp4a(reader: &mut (impl Read + Seek), len: usize) -> crate::Result<AudioInfo> {
    let mut audio_info = AudioInfo::default();

    let start_pos = reader.seek(SeekFrom::Current(0))?;

    reader.seek(SeekFrom::Current(28))?;

    let (_, ident) = parse_head(reader)?;
    if ident != ELEMENTARY_STREAM_DESCRIPTION {
        return Err(crate::Error::new(
            crate::ErrorKind::AtomNotFound(ELEMENTARY_STREAM_DESCRIPTION),
            "Missing esds atom".to_owned(),
        ));
    }

    parse_esds(reader, &mut audio_info, len - 36)?;

    let current_pos = reader.seek(SeekFrom::Current(0))?;
    let diff = current_pos - start_pos;
    reader.seek(SeekFrom::Current(len as i64 - diff as i64))?;

    Ok(audio_info)
}

fn parse_esds(
    reader: &mut (impl Read + Seek),
    audio_info: &mut AudioInfo,
    len: usize,
) -> crate::Result<()> {
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

    parse_es_desc(reader, audio_info, len - 4 - head_len)?;

    Ok(())
}

fn parse_es_desc(
    reader: &mut (impl Read + Seek),
    audio_info: &mut AudioInfo,
    len: usize,
) -> crate::Result<()> {
    reader.seek(SeekFrom::Current(3))?;

    let mut pos = 3;
    while pos < len {
        let (tag, head_len, len) = parse_desc_head(reader)?;
        match tag {
            DECODER_CONFIG_DESCRIPTOR => {
                parse_dc_desc(reader, audio_info)?;
            }
            _ => {
                reader.seek(SeekFrom::Current(len as i64))?;
            }
        }

        pos += head_len + len;
    }

    Ok(())
}

fn parse_dc_desc(reader: &mut (impl Read + Seek), audio_info: &mut AudioInfo) -> crate::Result<()> {
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

fn parse_ds_desc(reader: &mut (impl Read + Seek), audio_info: &mut AudioInfo) -> crate::Result<()> {
    let num = data::read_u16(reader)?;

    let freq_index = ((num >> 7) & 0x0F) as u8;
    audio_info.sample_rate = SampleRate::try_from(freq_index).ok();

    let channel_config = ((num >> 3) & 0x0F) as u8;
    audio_info.channel_config = ChannelConfig::try_from(channel_config).ok();

    Ok(())
}
