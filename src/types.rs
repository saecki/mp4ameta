// ITunes Media types
const MOVIE: u8 = 0;
const NORMAL: u8 = 1;
const AUDIOBOOK: u8 = 2;
const WHACKED_BOOKMARK: u8 = 5;
const MUSIC_VIDEO: u8 = 6;
const SHORT_FILM: u8 = 9;
const TV_SHOW: u8 = 10;
const BOOKLET: u8 = 11;

// ITunes Ratings
const CLEAN: u8 = 2;
const EXPLICIT: u8 = 4;

/// An enum describing the media type of a file.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Movie,
    Normal,
    AudioBook,
    WhackedBookmark,
    MusicVideo,
    ShortFilm,
    TvShow,
    Booklet,
}

impl MediaType {
    pub fn from(media_type: u8) -> Option<Self> {
        match media_type {
            MOVIE => Some(Self::Movie),
            NORMAL => Some(Self::Normal),
            AUDIOBOOK => Some(Self::AudioBook),
            WHACKED_BOOKMARK => Some(Self::WhackedBookmark),
            MUSIC_VIDEO => Some(Self::MusicVideo),
            SHORT_FILM => Some(Self::ShortFilm),
            TV_SHOW => Some(Self::TvShow),
            BOOKLET => Some(Self::Booklet),
            _ => None,
        }
    }

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

/// An enum describing the rating of a file.
#[derive(Debug, Clone, PartialEq)]
pub enum Rating {
    Clean,
    Explicit,
}

impl Rating {
    pub fn from(rating: u8) -> Option<Self> {
        match rating {
            CLEAN => Some(Self::Clean),
            EXPLICIT => Some(Self::Explicit),
            _ => None,
        }
    }

    /// Returns the integer value
    pub fn value(&self) -> u8 {
        match self {
            Rating::Clean => CLEAN,
            Rating::Explicit => EXPLICIT,
        }
    }
}