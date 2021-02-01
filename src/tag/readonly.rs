use std::convert::TryFrom;

use crate::{atom, ChannelConfig, SampleRate, Tag};

/// ### Duration
impl Tag {
    /// Returns the duration in seconds.
    pub fn duration(&self) -> crate::Result<f64> {
        let vec = self.mvhd.as_ref().ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::AtomNotFound(atom::MOVIE_HEADER),
                "Missing mvhd atom".to_owned(),
            )
        })?;
        let parsing_err = || {
            crate::Error::new(
                crate::ErrorKind::Parsing,
                "Error parsing contents of mvhd".to_owned(),
            )
        };
        let version = vec.get(0).ok_or_else(parsing_err)?;

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
                let timescale_unit = be_int!(vec, 12, u32).ok_or_else(parsing_err)?;
                let duration_units = be_int!(vec, 16, u32).ok_or_else(parsing_err)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Ok(duration)
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
                let timescale_unit = be_int!(vec, 20, u32).ok_or_else(parsing_err)?;
                let duration_units = be_int!(vec, 24, u64).ok_or_else(parsing_err)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Ok(duration)
            }
            v => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(*v),
                "Duration could not be parsed, unknown mdhd version".to_owned(),
            )),
        }
    }

    /// Returns the duration formatted in an easily readable way.
    pub(crate) fn format_duration(&self) -> Option<String> {
        let total_seconds = self.duration().ok()?.round() as usize;
        let seconds = total_seconds % 60;
        let minutes = total_seconds / 60;

        Some(format!("duration: {}:{:02}\n", minutes, seconds))
    }
}

impl Tag {
    fn decoder_config_descriptor(&self) -> crate::Result<&[u8]> {
        let vec = self.mp4a.as_ref().ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::AtomNotFound(atom::MPEG4_AUDIO),
                "Missing mp4a atom".to_owned(),
            )
        })?;

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
        //     es descriptor
        //     1 byte tag (0x03)
        //     4 bytes len
        //     2 bytes id
        //     1 byte flag
        //
        //       decoder config descriptor
        //       1 byte tag (0x04)
        //       4 bytes len
        //       1 byte object type indication
        //       1 byte stream type
        //       3 bytes buffer size
        //       4 bytes maximum bitrate
        //       4 bytes average bitrate
        //
        //         decoder specific descriptor
        //         1 byte tag (0x05)
        //         4 bytes len
        //         5 bits profile
        //         4 bits frequency index
        //         4 bits channel config
        //         3 bits ?
        //
        //       sl config descriptor
        //       1 byte tag (0x06)
        //       4 bytes len

        if Some(atom::ESDS.as_ref()) != vec.get(32..36) {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(atom::ESDS),
                "Missing esds atom".to_owned(),
            ));
        }

        if Some(&atom::ES_DESCRIPTOR) != vec.get(40) {
            return Err(crate::Error::new(
                crate::ErrorKind::DescriptorNotFound(atom::ES_DESCRIPTOR),
                "Missing es descriptor".to_owned(),
            ));
        }

        if Some(&atom::DECODER_CONFIG_DESCRIPTOR) != vec.get(48) {
            return Err(crate::Error::new(
                crate::ErrorKind::DescriptorNotFound(atom::DECODER_CONFIG_DESCRIPTOR),
                "Missing decoder config descriptor".to_owned(),
            ));
        }

        vec.get(48..73).ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::Parsing,
                "Error parsing decoder config descriptor".to_owned(),
            )
        })
    }

    /// Returns the decoder specific descriptor inside of the mp4a atom.
    fn decoder_specific_descriptor(&self) -> crate::Result<&[u8]> {
        let bytes = self.decoder_config_descriptor()?;
        // decoder config descriptor
        // 1 byte tag (0x04)
        // 4 bytes len
        // 1 byte object type indication
        // 1 byte stream type
        // 3 bytes buffer size
        // 4 bytes maximum bitrate
        // 4 bytes average bitrate
        //
        //   decoder specific descriptor
        //   1 byte tag (0x05)
        //   4 bytes len
        //   5 bits profile
        //   4 bits frequency index
        //   4 bits channel config
        //   3 bits ?

        if Some(&atom::DECODER_SPECIFIC_DESCRIPTOR) != bytes.get(18) {
            return Err(crate::Error::new(
                crate::ErrorKind::DescriptorNotFound(atom::DECODER_SPECIFIC_DESCRIPTOR),
                "Missing decoder specific descriptor".to_owned(),
            ));
        }

        bytes.get(18..25).ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::Parsing,
                "Error parsing decoder specific descriptor".to_owned(),
            )
        })
    }
}

/// ### Channel config
impl Tag {
    /// Returns the channel config.
    pub fn channel_config(&self) -> crate::Result<ChannelConfig> {
        let bytes = self.decoder_specific_descriptor()?;

        // decoder specific descriptor
        // 1 byte tag (0x05)
        // 4 bytes len
        // 5 bits profile
        // 4 bits frequency index
        // 4 bits channel config
        // 3 bits ?
        match bytes.get(5) {
            Some(b) => ChannelConfig::try_from(b >> 3),
            None => Err(crate::Error::new(
                crate::ErrorKind::Parsing,
                "Error parsing decoder specific descriptor".to_owned(),
            )),
        }
    }
}

/// ### Sample rate
impl Tag {
    /// Returns the channel config.
    pub fn sample_rate(&self) -> crate::Result<SampleRate> {
        let bytes = self.decoder_specific_descriptor()?;

        // decoder specific descriptor
        // 1 byte tag (0x05)
        // 4 bytes len
        // 5 bits profile
        // 4 bits frequency index
        // 4 bits channel config
        // 3 bits ?
        if let Some(num) = be_int!(bytes, 5, u16) {
            let freq_index = ((num >> 7) & 0x0F) as u8;
            return SampleRate::try_from(freq_index);
        }

        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            "Error parsing decoder specific descriptor".to_owned(),
        ))
    }
}

/// ### Bit rate
impl Tag {
    /// Returns the average bitrate.
    pub fn average_bitrate(&self) -> crate::Result<u32> {
        let bytes = self.decoder_config_descriptor()?;

        // decoder config descriptor
        // 1 byte tag (0x04)
        // 4 bytes len
        // 1 byte object type indication
        // 1 byte stream type
        // 3 bytes buffer size
        // 4 bytes maximum bitrate
        // 4 bytes average bitrate
        // ...
        if let Some(avg_bitrate) = be_int!(bytes, 14, u32) {
            return Ok(avg_bitrate);
        }

        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            "Error parsing decoder config descriptor".to_owned(),
        ))
    }

    /// Returns the maximum bitrate.
    pub fn maximum_bitrate(&self) -> crate::Result<u32> {
        let bytes = self.decoder_config_descriptor()?;

        // decoder config descriptor
        // 1 byte tag (0x04)
        // 4 bytes len
        // 1 byte object type indication
        // 1 byte stream type
        // 3 bytes buffer size
        // 4 bytes maximum bitrate
        // 4 bytes average bitrate
        // ...
        if let Some(max_bitrate) = be_int!(bytes, 10, u32) {
            return Ok(max_bitrate);
        }

        Err(crate::Error::new(
            crate::ErrorKind::Parsing,
            "Error parsing decoder config descriptor".to_owned(),
        ))
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}
