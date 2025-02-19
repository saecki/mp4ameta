use std::convert::TryFrom;
use std::fmt;
use std::time::Duration;

use crate::ErrorKind;

/// The iTunes media type of a file. This is stored in the `stik` atom.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaType {
    /// A media type stored as 0 in the `stik` atom.
    Movie = 0,
    /// A media type stored as 1 in the `stik` atom.
    Normal = 1,
    /// A media type stored as 2 in the `stik` atom.
    AudioBook = 2,
    /// A media type stored as 5 in the `stik` atom.
    WhackedBookmark = 5,
    /// A media type stored as 6 in the `stik` atom.
    MusicVideo = 6,
    /// A media type stored as 9 in the `stik` atom.
    ShortFilm = 9,
    /// A media type stored as 10 in the `stik` atom.
    TvShow = 10,
    /// A media type stored as 11 in the `stik` atom.
    Booklet = 11,
}

impl MediaType {
    const MOVIE: u8 = Self::Movie as u8;
    const NORMAL: u8 = Self::Normal as u8;
    const AUDIO_BOOK: u8 = Self::AudioBook as u8;
    const WHACKED_BOOKMARK: u8 = Self::WhackedBookmark as u8;
    const MUSIC_VIDEO: u8 = Self::MusicVideo as u8;
    const SHORT_FILM: u8 = Self::ShortFilm as u8;
    const TV_SHOW: u8 = Self::TvShow as u8;
    const BOOKLET: u8 = Self::Booklet as u8;

    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for MediaType {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            Self::MOVIE => Ok(Self::Movie),
            Self::NORMAL => Ok(Self::Normal),
            Self::AUDIO_BOOK => Ok(Self::AudioBook),
            Self::WHACKED_BOOKMARK => Ok(Self::WhackedBookmark),
            Self::MUSIC_VIDEO => Ok(Self::MusicVideo),
            Self::SHORT_FILM => Ok(Self::ShortFilm),
            Self::TV_SHOW => Ok(Self::TvShow),
            Self::BOOKLET => Ok(Self::Booklet),
            _ => Err(Self::Error::new(ErrorKind::UnknownMediaType(value), "Unknown media type")),
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Movie => write!(f, "Movie"),
            Self::Normal => write!(f, "Normal"),
            Self::AudioBook => write!(f, "Audiobook"),
            Self::WhackedBookmark => write!(f, "Whacked Bookmark"),
            Self::MusicVideo => write!(f, "Music Video"),
            Self::ShortFilm => write!(f, "Short Film"),
            Self::TvShow => write!(f, "TV-Show"),
            Self::Booklet => write!(f, "Booklet"),
        }
    }
}

/// The iTunes advisory rating of a file. This is stored in the `rtng` atom.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryRating {
    /// An advisory rating stored as 2 in the `rtng` atom.
    Clean = 2,
    /// An advisory rating stored as 0 in the `rtng` atom.
    Inoffensive = 0,
    /// An advisory rating indicated by any other value than 0 or 2 in the `rtng` atom.
    Explicit = 4,
}

impl AdvisoryRating {
    const CLEAN: u8 = Self::Clean as u8;
    const INOFFENSIVE: u8 = Self::Inoffensive as u8;

    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for AdvisoryRating {
    fn from(rating: u8) -> Self {
        match rating {
            Self::CLEAN => Self::Clean,
            Self::INOFFENSIVE => Self::Inoffensive,
            _ => Self::Explicit,
        }
    }
}

impl fmt::Display for AdvisoryRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clean => write!(f, "Clean"),
            Self::Inoffensive => write!(f, "Inoffensive"),
            Self::Explicit => write!(f, "Explicit"),
        }
    }
}

/// The channel configuration of an MPEG-4 audio track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelConfig {
    /// 1.0, channel: front-center.
    Mono = 1,
    /// 2.0, channels: front-left, front-right.
    Stereo = 2,
    /// 3.0, channels: front-center, front-left, front-right.
    Three = 3,
    /// 4.0, channels: front-center, front-left, front-right, back-center.
    Four = 4,
    /// 5.0, channels: front-center, front-left, front-right, back-left, back-right.
    Five = 5,
    /// 5.1, channels: front-center, front-left, front-right, back-left, back-right, LFE-channel.
    FiveOne = 6,
    /// 7.1, channels: front-center, front-left, front-right, side-left, side-right, back-left, back-right, LFE-channel.
    SevenOne = 7,
}

impl ChannelConfig {
    const MONO: u8 = Self::Mono as u8;
    const STEREO: u8 = Self::Stereo as u8;
    const THREE: u8 = Self::Three as u8;
    const FOUR: u8 = Self::Four as u8;
    const FIVE: u8 = Self::Five as u8;
    const FIVE_ONE: u8 = Self::FiveOne as u8;
    const SEVEN_ONE: u8 = Self::SevenOne as u8;

    /// Returns the number of channels.
    pub const fn channel_count(&self) -> u8 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::FiveOne => 6,
            Self::SevenOne => 8,
        }
    }
}

impl TryFrom<u8> for ChannelConfig {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            Self::MONO => Ok(Self::Mono),
            Self::STEREO => Ok(Self::Stereo),
            Self::THREE => Ok(Self::Three),
            Self::FOUR => Ok(Self::Four),
            Self::FIVE => Ok(Self::Five),
            Self::FIVE_ONE => Ok(Self::FiveOne),
            Self::SEVEN_ONE => Ok(Self::SevenOne),
            _ => Err(Self::Error::new(
                crate::ErrorKind::UnknownChannelConfig(value),
                "Unknown channel config index",
            )),
        }
    }
}

impl fmt::Display for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mono => write!(f, "Mono"),
            Self::Stereo => write!(f, "Stereo"),
            Self::Three => write!(f, "3.0"),
            Self::Four => write!(f, "4.0"),
            Self::Five => write!(f, "5.0"),
            Self::FiveOne => write!(f, "5.1"),
            Self::SevenOne => write!(f, "7.1"),
        }
    }
}

/// An enum representing the sample rate of an MPEG-4 audio track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleRate {
    /// A Sample rate of 96000Hz
    Hz96000 = 0,
    /// A Sample rate of 88200Hz
    Hz88200 = 1,
    /// A Sample rate of 64000Hz
    Hz64000 = 2,
    /// A Sample rate of 48000Hz
    Hz48000 = 3,
    /// A Sample rate of 44100Hz
    Hz44100 = 4,
    /// A Sample rate of 32000Hz
    Hz32000 = 5,
    /// A Sample rate of 24000Hz
    Hz24000 = 6,
    /// A Sample rate of 24050Hz
    Hz22050 = 7,
    /// A Sample rate of 16000Hz
    Hz16000 = 8,
    /// A Sample rate of 12000Hz
    Hz12000 = 9,
    /// A Sample rate of 11050Hz
    Hz11025 = 10,
    /// A Sample rate of 8000Hz
    Hz8000 = 11,
    /// A Sample rate of 7350Hz
    Hz7350 = 12,
}

impl SampleRate {
    const HZ_96000: u8 = Self::Hz96000 as u8;
    const HZ_88200: u8 = Self::Hz88200 as u8;
    const HZ_64000: u8 = Self::Hz64000 as u8;
    const HZ_48000: u8 = Self::Hz48000 as u8;
    const HZ_44100: u8 = Self::Hz44100 as u8;
    const HZ_32000: u8 = Self::Hz32000 as u8;
    const HZ_24000: u8 = Self::Hz24000 as u8;
    const HZ_22050: u8 = Self::Hz22050 as u8;
    const HZ_16000: u8 = Self::Hz16000 as u8;
    const HZ_12000: u8 = Self::Hz12000 as u8;
    const HZ_11025: u8 = Self::Hz11025 as u8;
    const HZ_8000: u8 = Self::Hz8000 as u8;
    const HZ_7350: u8 = Self::Hz7350 as u8;

    /// Returns the sample rate in Hz.
    pub const fn hz(&self) -> u32 {
        match self {
            Self::Hz96000 => 96000,
            Self::Hz88200 => 88200,
            Self::Hz64000 => 64000,
            Self::Hz48000 => 48000,
            Self::Hz44100 => 44100,
            Self::Hz32000 => 32000,
            Self::Hz24000 => 24000,
            Self::Hz22050 => 22050,
            Self::Hz16000 => 16000,
            Self::Hz12000 => 12000,
            Self::Hz11025 => 11025,
            Self::Hz8000 => 8000,
            Self::Hz7350 => 7350,
        }
    }
}

impl TryFrom<u8> for SampleRate {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            Self::HZ_96000 => Ok(Self::Hz96000),
            Self::HZ_88200 => Ok(Self::Hz88200),
            Self::HZ_64000 => Ok(Self::Hz64000),
            Self::HZ_48000 => Ok(Self::Hz48000),
            Self::HZ_44100 => Ok(Self::Hz44100),
            Self::HZ_32000 => Ok(Self::Hz32000),
            Self::HZ_24000 => Ok(Self::Hz24000),
            Self::HZ_22050 => Ok(Self::Hz22050),
            Self::HZ_16000 => Ok(Self::Hz16000),
            Self::HZ_12000 => Ok(Self::Hz12000),
            Self::HZ_11025 => Ok(Self::Hz11025),
            Self::HZ_8000 => Ok(Self::Hz8000),
            Self::HZ_7350 => Ok(Self::Hz7350),
            _ => Err(Self::Error::new(
                crate::ErrorKind::UnknownSampleRate(value),
                "Unknown sample rate index",
            )),
        }
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}Hz", self.hz())
    }
}

/// Audio information of an mp4 track.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioInfo {
    /// The duration of the track.
    pub duration: Duration,
    /// The channel configuration of the track.
    pub channel_config: Option<ChannelConfig>,
    /// The sample rate of the track.
    pub sample_rate: Option<SampleRate>,
    /// The maximum bitrate of the track.
    pub max_bitrate: Option<u32>,
    /// The average bitrate of the track.
    pub avg_bitrate: Option<u32>,
}

/// Type alias for an image reference.
pub type ImgRef<'a> = Img<&'a [u8]>;
/// Type alias for a mutable image reference.
pub type ImgMut<'a> = Img<&'a mut Vec<u8>>;
/// Type alias for an owned image buffer.
pub type ImgBuf = Img<Vec<u8>>;

/// Image data with an associated format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Img<T> {
    /// The image format.
    pub fmt: ImgFmt,
    /// The image data.
    pub data: T,
}

impl<T> Img<T> {
    pub const fn new(fmt: ImgFmt, data: T) -> Self {
        Self { fmt, data }
    }

    pub const fn bmp(data: T) -> Self {
        Self::new(ImgFmt::Bmp, data)
    }

    pub const fn jpeg(data: T) -> Self {
        Self::new(ImgFmt::Jpeg, data)
    }

    pub const fn png(data: T) -> Self {
        Self::new(ImgFmt::Png, data)
    }
}

/// The image format used to store images inside the userdata of an MPEG-4 file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImgFmt {
    /// Bmp.
    Bmp,
    /// Jpeg.
    Jpeg,
    /// Png.
    Png,
}

impl ImgFmt {
    /// Returns `true` if the img fmt is [`Bmp`].
    ///
    /// [`Bmp`]: ImgFmt::Bmp
    #[must_use]
    pub fn is_bmp(&self) -> bool {
        matches!(self, Self::Bmp)
    }

    /// Returns `true` if the img fmt is [`Jpeg`].
    ///
    /// [`Jpeg`]: ImgFmt::Jpeg
    #[must_use]
    pub fn is_jpeg(&self) -> bool {
        matches!(self, Self::Jpeg)
    }

    /// Returns `true` if the img fmt is [`Png`].
    ///
    /// [`Png`]: ImgFmt::Png
    #[must_use]
    pub fn is_png(&self) -> bool {
        matches!(self, Self::Png)
    }
}

/// A chapter.
///
/// Note that chapter titles have a relatively small maximum size.
/// For chapter lists this limit is 255 ([`u8::MAX`]);
/// For chapter tracks this limit is 65535 ([`u16::MAX`]);
/// If this limit is exceeded the title is truncated.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chapter {
    /// The start of the chapter.
    pub start: Duration,
    /// The title of the chapter.
    pub title: String,
}

impl Chapter {
    pub fn new(start: Duration, title: impl Into<String>) -> Self {
        Self { start, title: title.into() }
    }
}
