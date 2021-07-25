use std::fmt;
use std::time::Duration;

use crate::{util, AudioInfo, ChannelConfig, SampleRate, Tag};

/// ### Audio information
impl Tag {
    /// Returns a reference of the audio information.
    pub fn audio_info(&self) -> &AudioInfo {
        &self.info
    }

    /// Returns the duration in seconds.
    pub fn duration(&self) -> Option<Duration> {
        self.info.duration
    }

    /// Returns the duration formatted in an easily readable way.
    pub(crate) fn format_duration(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.duration() {
            Some(d) => {
                write!(f, "duration: ")?;
                util::format_duration(f, d)?;
                writeln!(f)
            }
            None => Ok(()),
        }
    }

    /// Returns the channel configuration.
    pub fn channel_config(&self) -> Option<ChannelConfig> {
        self.info.channel_config
    }

    pub(crate) fn format_channel_config(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.channel_config() {
            Some(c) => writeln!(f, "channel config: {}", c),
            None => Ok(()),
        }
    }

    /// Returns the channel configuration.
    pub fn sample_rate(&self) -> Option<SampleRate> {
        self.info.sample_rate
    }

    pub(crate) fn format_sample_rate(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.sample_rate() {
            Some(c) => writeln!(f, "sample rate: {}", c),
            None => Ok(()),
        }
    }

    /// Returns the average bitrate.
    pub fn avg_bitrate(&self) -> Option<u32> {
        self.info.avg_bitrate
    }

    pub(crate) fn format_avg_bitrate(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.avg_bitrate() {
            Some(c) => writeln!(f, "average bitrate: {}kbps", c / 1024),
            None => Ok(()),
        }
    }

    /// Returns the maximum bitrate.
    pub fn max_bitrate(&self) -> Option<u32> {
        self.info.max_bitrate
    }

    pub(crate) fn format_max_bitrate(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.max_bitrate() {
            Some(c) => writeln!(f, "maximum bitrate: {}kbps", c / 1024),
            None => Ok(()),
        }
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}
