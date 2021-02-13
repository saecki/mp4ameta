use std::time::Duration;

use crate::{ChannelConfig, SampleRate, Tag};

/// ### Audio information
impl Tag {
    /// Returns the duration in seconds.
    pub fn duration(&self) -> Option<Duration> {
        self.info.duration
    }

    /// Returns the duration formatted in an easily readable way.
    pub(crate) fn format_duration(&self) -> Option<String> {
        let total_seconds = self.duration()?.as_secs();
        let seconds = total_seconds % 60;
        let minutes = total_seconds / 60 % 60;
        let hours = total_seconds / 60 / 60;

        match (hours, minutes) {
            (0, 0) => Some(format!("duration: {:02}\n", seconds)),
            (0, _) => Some(format!("duration: {}:{:02}\n", minutes, seconds)),
            (_, _) => Some(format!("duration: {}:{:02}:{:02}\n", hours, minutes, seconds)),
        }
    }

    /// Returns the channel configuration.
    pub fn channel_config(&self) -> Option<ChannelConfig> {
        self.info.channel_config
    }

    /// Returns the channel configuration.
    pub fn sample_rate(&self) -> Option<SampleRate> {
        self.info.sample_rate
    }

    /// Returns the average bitrate.
    pub fn avg_bitrate(&self) -> Option<u32> {
        self.info.avg_bitrate
    }

    /// Returns the maximum bitrate.
    pub fn max_bitrate(&self) -> Option<u32> {
        self.info.max_bitrate
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}
