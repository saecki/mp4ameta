use std::fmt;
use std::ops::{Deref, DerefMut};

/// (`ftyp`) Identifier of an atom information about the filetype.
pub const FILETYPE: FourCC = FourCC(*b"ftyp");
/// (`mdat`)
pub const MEDIA_DATA: FourCC = FourCC(*b"mdat");
/// (`moov`) Identifier of an atom containing a structure of children storing metadata.
pub const MOVIE: FourCC = FourCC(*b"moov");
/// (`mvhd`) Identifier of an atom containing information about the whole movie (or audio file).
pub const MOVIE_HEADER: FourCC = FourCC(*b"mvhd");
/// (`trak`) Identifier of an atom containing information about a single track.
pub const TRACK: FourCC = FourCC(*b"trak");
/// (`mdia`) Identifier of an atom containing information about a tracks media type and data.
pub const MEDIA: FourCC = FourCC(*b"mdia");
/// (`mdhd`) Identifier of an atom containing information about a track
pub const MEDIA_HEADER: FourCC = FourCC(*b"mdhd");
/// (`minf`)
pub const MEDIA_INFORMATION: FourCC = FourCC(*b"minf");
/// (`stbl`)
pub const SAMPLE_TABLE: FourCC = FourCC(*b"stbl");
/// (`stco`)
pub const SAMPLE_TABLE_CHUNK_OFFSET: FourCC = FourCC(*b"stco");
/// (`stsd`)
pub const SAMPLE_TABLE_SAMPLE_DESCRIPTION: FourCC = FourCC(*b"stsd");
/// (`mp4a`)
pub const MPEG4_AUDIO: FourCC = FourCC(*b"mp4a");
/// (`esds`)
pub const ESDS: FourCC = FourCC(*b"esds");
/// (`udta`) Identifier of an atom containing user metadata.
pub const USER_DATA: FourCC = FourCC(*b"udta");
/// (`meta`) Identifier of an atom containing a metadata item list.
pub const METADATA: FourCC = FourCC(*b"meta");
/// (`ilst`) Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: FourCC = FourCC(*b"ilst");
/// (`data`) Identifier of an atom containing typed data.
pub const DATA: FourCC = FourCC(*b"data");
/// (`mean`)
pub const MEAN: FourCC = FourCC(*b"mean");
/// (`name`)
pub const NAME: FourCC = FourCC(*b"name");
/// (`free`)
pub const FREE: FourCC = FourCC(*b"free");

/// (`----`)
pub const FREEFORM: FourCC = FourCC(*b"----");
/// A identifier used internally as a wildcard.
pub const WILDCARD: FourCC = FourCC([255, 255, 255, 255]);

// iTunes 4.0 atoms
/// (`rtng`)
pub const ADVISORY_RATING: FourCC = FourCC(*b"rtng");
/// (`©alb`)
pub const ALBUM: FourCC = FourCC(*b"\xa9alb");
/// (`aART`)
pub const ALBUM_ARTIST: FourCC = FourCC(*b"aART");
/// (`©ART`)
pub const ARTIST: FourCC = FourCC(*b"\xa9ART");
/// (`covr`)
pub const ARTWORK: FourCC = FourCC(*b"covr");
/// (`tmpo`)
pub const BPM: FourCC = FourCC(*b"tmpo");
/// (`©cmt`)
pub const COMMENT: FourCC = FourCC(*b"\xa9cmt");
/// (`cpil`)
pub const COMPILATION: FourCC = FourCC(*b"cpil");
/// (`©wrt`)
pub const COMPOSER: FourCC = FourCC(*b"\xa9wrt");
/// (`cprt`)
pub const COPYRIGHT: FourCC = FourCC(*b"cprt");
/// (`©gen`)
pub const CUSTOM_GENRE: FourCC = FourCC(*b"\xa9gen");
/// (`disk`)
pub const DISC_NUMBER: FourCC = FourCC(*b"disk");
/// (`©too`)
pub const ENCODER: FourCC = FourCC(*b"\xa9too");
/// (`gnre`)
pub const STANDARD_GENRE: FourCC = FourCC(*b"gnre");
/// (`©nam`)
pub const TITLE: FourCC = FourCC(*b"\xa9nam");
/// (`trkn`)
pub const TRACK_NUMBER: FourCC = FourCC(*b"trkn");
/// (`©day`)
pub const YEAR: FourCC = FourCC(*b"\xa9day");

// iTunes 4.2 atoms
/// (`©grp`)
pub const GROUPING: FourCC = FourCC(*b"\xa9grp");
/// (`stik`)
pub const MEDIA_TYPE: FourCC = FourCC(*b"stik");

// iTunes 4.9 atoms
/// (`catg`)
pub const CATEGORY: FourCC = FourCC(*b"catg");
/// (`keyw`)
pub const KEYWORD: FourCC = FourCC(*b"keyw");
/// (`pcst`)
pub const PODCAST: FourCC = FourCC(*b"pcst");
/// (`egid`)
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: FourCC = FourCC(*b"egid");
/// (`purl`)
pub const PODCAST_URL: FourCC = FourCC(*b"purl");

// iTunes 5.0
/// (`desc`)
pub const DESCRIPTION: FourCC = FourCC(*b"desc");
/// (`©lyr`)
pub const LYRICS: FourCC = FourCC(*b"\xa9lyr");

// iTunes 6.0
/// (`tves`)
pub const TV_EPISODE: FourCC = FourCC(*b"tves");
/// (`tven`)
pub const TV_EPISODE_NUMBER: FourCC = FourCC(*b"tven");
/// (`tvnn`)
pub const TV_NETWORK_NAME: FourCC = FourCC(*b"tvnn");
/// (`tvsn`)
pub const TV_SEASON: FourCC = FourCC(*b"tvsn");
/// (`tvsh`)
pub const TV_SHOW_NAME: FourCC = FourCC(*b"tvsh");

// iTunes 6.0.2
/// (`purd`)
pub const PURCHASE_DATE: FourCC = FourCC(*b"purd");

// iTunes 7.0
/// (`pgap`)
pub const GAPLESS_PLAYBACK: FourCC = FourCC(*b"pgap");

// Work, Movement
/// (`©mvn`)
pub const MOVEMENT: FourCC = FourCC(*b"\xa9mvn");
/// (`©mvc`)
pub const MOVEMENT_COUNT: FourCC = FourCC(*b"\xa9mvc");
/// (`©mvi`)
pub const MOVEMENT_INDEX: FourCC = FourCC(*b"\xa9mvi");
/// (`©wrk`)
pub const WORK: FourCC = FourCC(*b"\xa9wrk");
/// (`shwm`)
pub const SHOW_MOVEMENT: FourCC = FourCC(*b"shwm");

/// A trait providing information about an identifier.
pub trait Ident {
    /// Returns a 4 byte atom identifier.
    fn fourcc(&self) -> Option<FourCC>;
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
pub struct FourCC(pub [u8; 4]);

impl Deref for FourCC {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FourCC {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Ident for FourCC {
    fn fourcc(&self) -> Option<FourCC> {
        Some(*self)
    }

    fn freeform(&self) -> Option<FreeformIdent> {
        None
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident({})", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

/// An identifier of a freeform (`----`) atom containing borrowd mean and name strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreeformIdent<'a> {
    /// The mean string, typically in reverse domain notation.
    mean: &'a str,
    /// The name string used to identify the freeform atom.
    name: &'a str,
}

impl Ident for FreeformIdent<'_> {
    fn fourcc(&self) -> Option<FourCC> {
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
    pub fn new(mean: &'a str, name: &'a str) -> Self {
        Self { mean, name }
    }
}

/// An identifier for data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataIdent {
    /// A standard identifier containing a 4 byte atom identifier.
    FourCC(FourCC),
    /// An identifier of a freeform (`----`) atom containing owned mean and name strings.
    Freeform {
        /// The mean string, typically in reverse domain notation.
        mean: String,
        /// The name string used to identify the freeform atom.
        name: String,
    },
}

impl Ident for DataIdent {
    fn fourcc(&self) -> Option<FourCC> {
        match self {
            Self::FourCC(i) => Some(*i),
            Self::Freeform { .. } => None,
        }
    }

    fn freeform(&self) -> Option<FreeformIdent> {
        match self {
            Self::FourCC(_) => None,
            Self::Freeform { mean, name } => Some(FreeformIdent::new(mean.as_str(), name.as_str())),
        }
    }
}

impl fmt::Display for DataIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FourCC(ident) => write!(f, "{}", ident),
            Self::Freeform { mean, name } => write!(f, "----:{}:{}", mean, name),
        }
    }
}

impl From<FourCC> for DataIdent {
    fn from(value: FourCC) -> Self {
        Self::FourCC(value)
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
    /// Creates a new identifier of type [`DataIdent::Freeform`](Self::Freeform) containing the owned
    /// mean, and name string.
    pub fn freeform(mean: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Freeform { mean: mean.into(), name: name.into() }
    }

    /// Creates a new identifier of type [`DataIdent::FourCC`](Self::FourCC) containing an atom identifier
    /// with the 4-byte identifier.
    pub const fn fourcc(bytes: [u8; 4]) -> Self {
        Self::FourCC(FourCC(bytes))
    }
}
