use std::convert::TryFrom;
use crate::ErrorKind;

// ITunes Media types
pub const MOVIE: u8 = 0;
pub const NORMAL: u8 = 1;
pub const AUDIOBOOK: u8 = 2;
pub const WHACKED_BOOKMARK: u8 = 5;
pub const MUSIC_VIDEO: u8 = 6;
pub const SHORT_FILM: u8 = 9;
pub const TV_SHOW: u8 = 10;
pub const BOOKLET: u8 = 11;

// ITunes Ratings
pub const CLEAN: u8 = 2;
pub const INOFFENSIVE: u8 = 0;

/// An enum describing the media type of a file stored in the `stik` atom.
#[derive(Debug, Clone, PartialEq)]
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
    /// A media type stored as 10 in the `stik` atom.
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
            _ => Err(crate::Error::new(
                ErrorKind::UnknownMediaType(value),
                "Unknown media type".into(),
            )),
        }
    }
}

/// An enum describing the rating of a file.
#[derive(Debug, Clone, PartialEq)]
pub enum AdvisoryRating {
    /// A rating stored as 2 in the `stik` atom.
    Clean,
    /// A rating stored as 0 in the `stik` atom.
    Inoffensive,
    /// A rating indicated by any other value than 0 or 2 in the `stik` atom, containing the value.
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