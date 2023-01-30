use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// (`ftyp`) Identifier of an atom information about the filetype.
pub(crate) const FILETYPE: Fourcc = Fourcc(*b"ftyp");
/// (`mdat`)
pub(crate) const MEDIA_DATA: Fourcc = Fourcc(*b"mdat");
/// (`moov`) Identifier of an atom containing a structure of children storing metadata.
pub(crate) const MOVIE: Fourcc = Fourcc(*b"moov");
/// (`mvhd`) Identifier of an atom containing information about the whole movie (or audio file).
pub(crate) const MOVIE_HEADER: Fourcc = Fourcc(*b"mvhd");
/// (`trak`) Identifier of an atom containing information about a single track.
pub(crate) const TRACK: Fourcc = Fourcc(*b"trak");
/// (`mdia`) Identifier of an atom containing information about a tracks media type and data.
pub(crate) const MEDIA: Fourcc = Fourcc(*b"mdia");
/// (`minf`)
pub(crate) const MEDIA_INFORMATION: Fourcc = Fourcc(*b"minf");
/// (`stbl`)
pub(crate) const SAMPLE_TABLE: Fourcc = Fourcc(*b"stbl");
/// (`stco`)
pub(crate) const SAMPLE_TABLE_CHUNK_OFFSET: Fourcc = Fourcc(*b"stco");
/// (`co64`)
pub(crate) const SAMPLE_TABLE_CHUNK_OFFSET_64: Fourcc = Fourcc(*b"co64");
/// (`stsd`)
pub(crate) const SAMPLE_TABLE_SAMPLE_DESCRIPTION: Fourcc = Fourcc(*b"stsd");
/// (`mp4a`)
pub(crate) const MP4_AUDIO: Fourcc = Fourcc(*b"mp4a");
/// (`esds`)
pub(crate) const ELEMENTARY_STREAM_DESCRIPTION: Fourcc = Fourcc(*b"esds");
/// (`udta`) Identifier of an atom containing user metadata.
pub(crate) const USER_DATA: Fourcc = Fourcc(*b"udta");
/// (`meta`) Identifier of an atom containing a metadata item list.
pub(crate) const METADATA: Fourcc = Fourcc(*b"meta");
/// (`hdlr`) Identifier of an atom specifying the handler component that should interpret the medias data.
pub(crate) const HANDLER_REFERENCE: Fourcc = Fourcc(*b"hdlr");
/// (`ilst`) Identifier of an atom containing a list of metadata atoms.
pub(crate) const ITEM_LIST: Fourcc = Fourcc(*b"ilst");
/// (`data`) Identifier of an atom containing typed data.
pub(crate) const DATA: Fourcc = Fourcc(*b"data");
/// (`mean`)
pub(crate) const MEAN: Fourcc = Fourcc(*b"mean");
/// (`name`)
pub(crate) const NAME: Fourcc = Fourcc(*b"name");
/// (`free`)
pub(crate) const FREE: Fourcc = Fourcc(*b"free");

/// (`----`)
pub const FREEFORM: Fourcc = Fourcc(*b"----");

// iTunes 4.0 atoms
/// (`rtng`)
pub const ADVISORY_RATING: Fourcc = Fourcc(*b"rtng");
/// (`©alb`)
pub const ALBUM: Fourcc = Fourcc(*b"\xa9alb");
/// (`aART`)
pub const ALBUM_ARTIST: Fourcc = Fourcc(*b"aART");
/// (`©ART`)
pub const ARTIST: Fourcc = Fourcc(*b"\xa9ART");
/// (`covr`)
pub const ARTWORK: Fourcc = Fourcc(*b"covr");
/// (`tmpo`)
pub const BPM: Fourcc = Fourcc(*b"tmpo");
/// (`©cmt`)
pub const COMMENT: Fourcc = Fourcc(*b"\xa9cmt");
/// (`cpil`)
pub const COMPILATION: Fourcc = Fourcc(*b"cpil");
/// (`©wrt`)
pub const COMPOSER: Fourcc = Fourcc(*b"\xa9wrt");
/// (`cprt`)
pub const COPYRIGHT: Fourcc = Fourcc(*b"cprt");
/// (`©gen`)
pub const CUSTOM_GENRE: Fourcc = Fourcc(*b"\xa9gen");
/// (`disk`)
pub const DISC_NUMBER: Fourcc = Fourcc(*b"disk");
/// (`©too`)
pub const ENCODER: Fourcc = Fourcc(*b"\xa9too");
/// (`gnre`)
pub const STANDARD_GENRE: Fourcc = Fourcc(*b"gnre");
/// (`©nam`)
pub const TITLE: Fourcc = Fourcc(*b"\xa9nam");
/// (`trkn`)
pub const TRACK_NUMBER: Fourcc = Fourcc(*b"trkn");
/// (`©day`)
pub const YEAR: Fourcc = Fourcc(*b"\xa9day");

// iTunes 4.2 atoms
/// (`©grp`)
pub const GROUPING: Fourcc = Fourcc(*b"\xa9grp");
/// (`stik`)
pub const MEDIA_TYPE: Fourcc = Fourcc(*b"stik");

// iTunes 4.9 atoms
/// (`catg`)
pub const CATEGORY: Fourcc = Fourcc(*b"catg");
/// (`keyw`)
pub const KEYWORD: Fourcc = Fourcc(*b"keyw");
/// (`pcst`)
pub const PODCAST: Fourcc = Fourcc(*b"pcst");
/// (`egid`)
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: Fourcc = Fourcc(*b"egid");
/// (`purl`)
pub const PODCAST_URL: Fourcc = Fourcc(*b"purl");

// iTunes 5.0
/// (`desc`)
pub const DESCRIPTION: Fourcc = Fourcc(*b"desc");
/// (`©lyr`)
pub const LYRICS: Fourcc = Fourcc(*b"\xa9lyr");

// iTunes 6.0
/// (`tves`)
pub const TV_EPISODE: Fourcc = Fourcc(*b"tves");
/// (`tven`)
pub const TV_EPISODE_NAME: Fourcc = Fourcc(*b"tven");
/// (`tvnn`)
pub const TV_NETWORK_NAME: Fourcc = Fourcc(*b"tvnn");
/// (`tvsn`)
pub const TV_SEASON: Fourcc = Fourcc(*b"tvsn");
/// (`tvsh`)
pub const TV_SHOW_NAME: Fourcc = Fourcc(*b"tvsh");

// iTunes 6.0.2
/// (`purd`)
pub const PURCHASE_DATE: Fourcc = Fourcc(*b"purd");

// iTunes 7.0
/// (`pgap`)
pub const GAPLESS_PLAYBACK: Fourcc = Fourcc(*b"pgap");

// Work, Movement
/// (`©mvn`)
pub const MOVEMENT: Fourcc = Fourcc(*b"\xa9mvn");
/// (`©mvc`)
pub const MOVEMENT_COUNT: Fourcc = Fourcc(*b"\xa9mvc");
/// (`©mvi`)
pub const MOVEMENT_INDEX: Fourcc = Fourcc(*b"\xa9mvi");
/// (`©wrk`)
pub const WORK: Fourcc = Fourcc(*b"\xa9wrk");
/// (`shwm`)
pub const SHOW_MOVEMENT: Fourcc = Fourcc(*b"shwm");

// Freeform
/// Mean string of most freeform identifiers (`com.apple.iTunes`)
pub const APPLE_ITUNES_MEAN: &str = "com.apple.iTunes";

/// (`----:com.apple.iTunes:ISRC`)
pub const ISRC: FreeformIdent = FreeformIdent::new(APPLE_ITUNES_MEAN, "ISRC");
/// (`----:com.apple.iTunes:LYRICIST`)
pub const LYRICIST: FreeformIdent = FreeformIdent::new(APPLE_ITUNES_MEAN, "LYRICIST");

/// A trait providing information about an identifier.
pub trait Ident: PartialEq<DataIdent> {
    /// Returns a 4 byte atom identifier.
    fn fourcc(&self) -> Option<Fourcc>;
    /// Returns a freeform identifier.
    fn freeform(&self) -> Option<FreeformIdent>;
}

// TODO: figure out how to implement PartialEq for Ident or require an implementation as a trait bound.
/// Returns wheter the identifiers match.
pub fn idents_match(a: &impl Ident, b: &impl Ident) -> bool {
    a.fourcc() == b.fourcc() && a.freeform() == b.freeform()
}

/// A 4 byte atom identifier (four character code).
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Fourcc(pub [u8; 4]);

impl Deref for Fourcc {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fourcc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq<DataIdent> for Fourcc {
    fn eq(&self, other: &DataIdent) -> bool {
        match other {
            DataIdent::Fourcc(f) => self == f,
            DataIdent::Freeform { .. } => false,
        }
    }
}

impl Ident for Fourcc {
    fn fourcc(&self) -> Option<Fourcc> {
        Some(*self)
    }

    fn freeform(&self) -> Option<FreeformIdent> {
        None
    }
}

impl FromStr for Fourcc {
    type Err = TryFromSliceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Fourcc(s.as_bytes().try_into()?))
    }
}

impl fmt::Debug for Fourcc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fourcc({})", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

impl fmt::Display for Fourcc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

/// An identifier of a freeform (`----`) atom containing borrowd mean and name strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreeformIdent<'a> {
    /// The mean string, typically in reverse domain notation.
    pub mean: &'a str,
    /// The name string used to identify the freeform atom.
    pub name: &'a str,
}

impl PartialEq<DataIdent> for FreeformIdent<'_> {
    fn eq(&self, other: &DataIdent) -> bool {
        match other {
            DataIdent::Fourcc(_) => false,
            DataIdent::Freeform { mean, name } => self.mean == mean && self.name == name,
        }
    }
}

impl Ident for FreeformIdent<'_> {
    fn fourcc(&self) -> Option<Fourcc> {
        None
    }

    fn freeform(&self) -> Option<FreeformIdent> {
        Some(self.clone())
    }
}

impl fmt::Display for FreeformIdent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "----:{}:{}", self.mean, self.name)
    }
}

impl<'a> FreeformIdent<'a> {
    /// Creates a new freeform ident containing the mean and name as borrowed strings.
    pub const fn new(mean: &'a str, name: &'a str) -> Self {
        Self { mean, name }
    }
}

/// An identifier for data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataIdent {
    /// A standard identifier containing a 4 byte atom identifier.
    Fourcc(Fourcc),
    /// An identifier of a freeform (`----`) atom containing owned mean and name strings.
    Freeform {
        /// The mean string, typically in reverse domain notation.
        mean: String,
        /// The name string used to identify the freeform atom.
        name: String,
    },
}

impl Ident for DataIdent {
    fn fourcc(&self) -> Option<Fourcc> {
        match self {
            Self::Fourcc(i) => Some(*i),
            Self::Freeform { .. } => None,
        }
    }

    fn freeform(&self) -> Option<FreeformIdent> {
        match self {
            Self::Fourcc(_) => None,
            Self::Freeform { mean, name } => Some(FreeformIdent::new(mean.as_str(), name.as_str())),
        }
    }
}

impl fmt::Display for DataIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Fourcc(ident) => write!(f, "{ident}"),
            Self::Freeform { mean, name } => write!(f, "----:{mean}:{name}"),
        }
    }
}

impl From<Fourcc> for DataIdent {
    fn from(value: Fourcc) -> Self {
        Self::Fourcc(value)
    }
}

impl From<FreeformIdent<'_>> for DataIdent {
    fn from(value: FreeformIdent) -> Self {
        Self::freeform(value.mean, value.name)
    }
}

impl From<&FreeformIdent<'_>> for DataIdent {
    fn from(value: &FreeformIdent) -> Self {
        Self::freeform(value.mean, value.name)
    }
}

impl DataIdent {
    /// Creates a new identifier of type [`DataIdent::Freeform`] containing the owned mean, and
    /// name string.
    pub fn freeform(mean: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Freeform { mean: mean.into(), name: name.into() }
    }

    /// Creates a new identifier of type [`DataIdent::Fourcc`] containing an atom identifier with
    /// the 4-byte identifier.
    pub const fn fourcc(bytes: [u8; 4]) -> Self {
        Self::Fourcc(Fourcc(bytes))
    }
}
