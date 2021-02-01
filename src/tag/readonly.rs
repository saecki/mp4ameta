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
                let timescale = be_int!(vec, 12, u32).ok_or_else(parsing_err)?;
                let duration = be_int!(vec, 16, u32).ok_or_else(parsing_err)?;

                Ok(duration as f64 / timescale as f64)
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
                let timescale = be_int!(vec, 20, u32).ok_or_else(parsing_err)?;
                let duration = be_int!(vec, 24, u64).ok_or_else(parsing_err)?;

                Ok(duration as f64 / timescale as f64)
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

/// ### Channel config
impl Tag {
    /// Returns the channel config.
    pub fn channel_config(&self) -> Option<ChannelConfig> {
        self.audio_info.channel_config
    }
}

/// ### Sample rate
impl Tag {
    /// Returns the channel config.
    pub fn sample_rate(&self) -> Option<SampleRate> {
        self.audio_info.sample_rate
    }
}

/// ### Bit rate
impl Tag {
    /// Returns the average bitrate.
    pub fn avg_bitrate(&self) -> Option<u32> {
        self.audio_info.avg_bitrate
    }

    /// Returns the maximum bitrate.
    pub fn max_bitrate(&self) -> Option<u32> {
        self.audio_info.max_bitrate
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}
