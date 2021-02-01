use std::{convert::TryFrom, fmt};

use crate::ErrorKind;

// iTunes media types
/// A media type code stored in the `stik` atom.
pub const MOVIE: u8 = 0;
/// A media type code stored in the `stik` atom.
pub const NORMAL: u8 = 1;
/// A media type code stored in the `stik` atom.
pub const AUDIOBOOK: u8 = 2;
/// A media type code stored in the `stik` atom.
pub const WHACKED_BOOKMARK: u8 = 5;
/// A media type code stored in the `stik` atom.
pub const MUSIC_VIDEO: u8 = 6;
/// A media type code stored in the `stik` atom.
pub const SHORT_FILM: u8 = 9;
/// A media type code stored in the `stik` atom.
pub const TV_SHOW: u8 = 10;
/// A media type code stored in the `stik` atom.
pub const BOOKLET: u8 = 11;

// iTunes advisory ratings
/// An advisory rating code stored in the `rtng` atom.
pub const CLEAN: u8 = 2;
/// An advisory rating code stored in the `rtng` atom.
pub const INOFFENSIVE: u8 = 0;

// channnel configurations
/// Mono
pub const MONO: u8 = 1;
/// Stereo
pub const STEREO: u8 = 2;
/// 3.0
pub const THREE: u8 = 3;
/// 4.0
pub const FOUR: u8 = 4;
/// 5.0
pub const FIVE: u8 = 5;
/// 5.1
pub const FIVE_ONE: u8 = 6;
/// 7.1
pub const SEVEN_ONE: u8 = 7;

// sample rates
/// 96000Hz
pub const F96000: u8 = 0x0;
/// 882000Hz
pub const F88200: u8 = 0x1;
/// 640000Hz
pub const F64000: u8 = 0x2;
/// 480000Hz
pub const F48000: u8 = 0x3;
/// 44100Hz
pub const F44100: u8 = 0x4;
/// 32000Hz
pub const F32000: u8 = 0x5;
/// 242000Hz
pub const F24000: u8 = 0x6;
/// 22050Hz
pub const F22050: u8 = 0x7;
/// 16000Hz
pub const F16000: u8 = 0x8;
/// 12000Hz
pub const F12000: u8 = 0x9;
/// 11025Hz
pub const F11025: u8 = 0xa;
/// 8000Hz
pub const F8000: u8 = 0xb;

/// An enum describing the media type of a file stored in the `stik` atom.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MediaType {
    /// A media type stored as 0 in the `stik` atom.
    Movie,
    /// A media type stored as 1 in the `stik` atom.
    Normal,
    /// A media type stored as 2 in the `stik` atom.
    AudioBook,
    /// A media type stored as 5 in the `stik` atom.
    WhackedBookmark,
    /// A media type stored as 6 in the `stik` atom.
    MusicVideo,
    /// A media type stored as 9 in the `stik` atom.
    ShortFilm,
    /// A media type stored as 10 in the `stik` atom.
    TvShow,
    /// A media type stored as 11 in the `stik` atom.
    Booklet,
}

impl MediaType {
    /// Returns the integer value corresponding to the media type.
    pub fn value(&self) -> u8 {
        match self {
            Self::Movie => MOVIE,
            Self::Normal => NORMAL,
            Self::AudioBook => AUDIOBOOK,
            Self::WhackedBookmark => WHACKED_BOOKMARK,
            Self::MusicVideo => MUSIC_VIDEO,
            Self::ShortFilm => SHORT_FILM,
            Self::TvShow => TV_SHOW,
            Self::Booklet => BOOKLET,
        }
    }
}

impl TryFrom<u8> for MediaType {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            MOVIE => Ok(Self::Movie),
            NORMAL => Ok(Self::Normal),
            AUDIOBOOK => Ok(Self::AudioBook),
            WHACKED_BOOKMARK => Ok(Self::WhackedBookmark),
            MUSIC_VIDEO => Ok(Self::MusicVideo),
            SHORT_FILM => Ok(Self::ShortFilm),
            TV_SHOW => Ok(Self::TvShow),
            BOOKLET => Ok(Self::Booklet),
            _ => Err(Self::Error::new(
                ErrorKind::UnknownMediaType(value),
                "Unknown media type".to_owned(),
            )),
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

/// An enum describing the rating of a file stored in the `rtng` atom.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdvisoryRating {
    /// An advisory rating stored as 2 in the `rtng` atom.
    Clean,
    /// An advisory rating stored as 0 in the `rtng` atom.
    Inoffensive,
    /// An advisory rating indicated by any other value than 0 or 2 in the `rtng` atom, containing
    /// the value.
    Explicit(u8),
}

impl AdvisoryRating {
    /// Returns the integer value corresponding to the rating.
    pub fn value(&self) -> u8 {
        match self {
            Self::Clean => CLEAN,
            Self::Inoffensive => INOFFENSIVE,
            Self::Explicit(r) => *r,
        }
    }
}

impl From<u8> for AdvisoryRating {
    fn from(rating: u8) -> Self {
        match rating {
            CLEAN => Self::Clean,
            INOFFENSIVE => Self::Inoffensive,
            _ => Self::Explicit(rating),
        }
    }
}

impl fmt::Display for AdvisoryRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clean => write!(f, "Clean"),
            Self::Inoffensive => write!(f, "Inoffensive"),
            Self::Explicit(r) => write!(f, "Explicit {}", r),
        }
    }
}

/// An enum representing the channel configuration of an MPEG-4 audio track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelConfig {
    /// Mono
    Mono,
    /// Stereo
    Stereo,
    /// 3.0
    Three,
    /// 4.0
    Four,
    /// 5.0
    Five,
    /// 5.1
    FiveOne,
    /// 7.1
    SevenOne,
}

impl ChannelConfig {
    /// Returns the integer value corresponding to the channel config.
    pub fn value(&self) -> u8 {
        match self {
            Self::Mono => MONO,
            Self::Stereo => STEREO,
            Self::Three => THREE,
            Self::Four => FOUR,
            Self::Five => FIVE,
            Self::FiveOne => FIVE_ONE,
            Self::SevenOne => SEVEN_ONE,
        }
    }
}

impl TryFrom<u8> for ChannelConfig {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            MONO => Ok(Self::Mono),
            STEREO => Ok(Self::Stereo),
            THREE => Ok(Self::Three),
            FOUR => Ok(Self::Four),
            FIVE => Ok(Self::Five),
            FIVE_ONE => Ok(Self::FiveOne),
            SEVEN_ONE => Ok(Self::SevenOne),
            _ => Err(Self::Error::new(
                crate::ErrorKind::UnknownChannelConfig(value),
                "Unknown channel config".to_owned(),
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SampleRate {
    /// A Sample rate of 96000Hz
    F96000,
    /// A Sample rate of 88200Hz
    F88200,
    /// A Sample rate of 64000Hz
    F64000,
    /// A Sample rate of 48000Hz
    F48000,
    /// A Sample rate of 44100Hz
    F44100,
    /// A Sample rate of 32000Hz
    F32000,
    /// A Sample rate of 24000Hz
    F24000,
    /// A Sample rate of 24050Hz
    F22050,
    /// A Sample rate of 16000Hz
    F16000,
    /// A Sample rate of 12000Hz
    F12000,
    /// A Sample rate of 11050Hz
    F11025,
    /// A Sample rate of 8000Hz
    F8000,
}

impl TryFrom<u8> for SampleRate {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            F96000 => Ok(Self::F96000),
            F88200 => Ok(Self::F88200),
            F64000 => Ok(Self::F64000),
            F48000 => Ok(Self::F48000),
            F44100 => Ok(Self::F44100),
            F32000 => Ok(Self::F32000),
            F24000 => Ok(Self::F24000),
            F22050 => Ok(Self::F22050),
            F16000 => Ok(Self::F16000),
            F12000 => Ok(Self::F12000),
            F11025 => Ok(Self::F11025),
            F8000 => Ok(Self::F8000),
            _ => Err(Self::Error::new(
                crate::ErrorKind::UnknownChannelConfig(value),
                "Unknown channel config".to_owned(),
            )),
        }
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F96000 => write!(f, "96000Hz"),
            Self::F88200 => write!(f, "88200Hz"),
            Self::F64000 => write!(f, "64000Hz"),
            Self::F48000 => write!(f, "48000Hz"),
            Self::F44100 => write!(f, "44100Hz"),
            Self::F32000 => write!(f, "32000Hz"),
            Self::F24000 => write!(f, "24000Hz"),
            Self::F22050 => write!(f, "22050Hz"),
            Self::F16000 => write!(f, "16000Hz"),
            Self::F12000 => write!(f, "12000Hz"),
            Self::F11025 => write!(f, "11025Hz"),
            Self::F8000 => write!(f, "8000Hz"),
        }
    }
}
