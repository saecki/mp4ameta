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
    pub channel_config: Option<ChannelConfig>,
    pub sample_rate: Option<SampleRate>,
    pub max_bitrate: Option<u32>,
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
// │
// └──esds atom
//    4 bytes len
//    4 bytes ident
//    1 byte version
//    3 bytes flags
//    │
//    └──elementary stream descriptor
//       1 byte tag (0x03)
//       1~4 bytes len
//       2 bytes id
//       1 byte flag
//       │
//       ├──decoder config descriptor
//       │  1 byte tag (0x04)
//       │  1~4 bytes len
//       │  1 byte object type indication
//       │  1 byte stream type
//       │  3 bytes buffer size
//       │  4 bytes maximum bitrate
//       │  4 bytes average bitrate
//       │  │
//       │  └──decoder specific descriptor
//       │     1 byte tag (0x05)
//       │     1~4 bytes len
//       │     5 bits profile
//       │     4 bits frequency index
//       │     4 bits channel config
//       │     3 bits ?
//       │
//       └──sl config descriptor
//          1 byte tag (0x06)
//          1~4 bytes len
//          1 byte ?

impl Atom for Mp4a {
    const FOURCC: Fourcc = MP4_AUDIO;
}

impl ParseAtom for Mp4a {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut mp4a = Self::default();

        let start = reader.seek(SeekFrom::Current(0))?;

        reader.seek(SeekFrom::Current(28))?;

        let head = parse_head(reader)?;
        if head.fourcc() != ELEMENTARY_STREAM_DESCRIPTION {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(ELEMENTARY_STREAM_DESCRIPTION),
                "Missing esds atom".to_owned(),
            ));
        }

        parse_esds(reader, &mut mp4a, head.content_len())?;

        data::seek_to_end(reader, start, size.content_len())?;

        Ok(mp4a)
    }
}

fn parse_esds(reader: &mut (impl Read + Seek), info: &mut Mp4a, len: u64) -> crate::Result<()> {
    let (version, _) = parse_full_head(reader)?;

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

fn parse_es_desc(reader: &mut (impl Read + Seek), info: &mut Mp4a, len: u64) -> crate::Result<()> {
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

fn parse_dc_desc(reader: &mut (impl Read + Seek), audio_info: &mut Mp4a) -> crate::Result<()> {
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

fn parse_ds_desc(reader: &mut (impl Read + Seek), audio_info: &mut Mp4a) -> crate::Result<()> {
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
