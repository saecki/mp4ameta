use core::{fmt, ops::Deref};

/// (`ftyp`) Identifier of an atom information about the filetype.
pub const FILETYPE: AtomIdent = AtomIdent(*b"ftyp");
/// (`mdat`)
pub const MEDIA_DATA: AtomIdent = AtomIdent(*b"mdat");
/// (`moov`) Identifier of an atom containing a structure of children storing metadata.
pub const MOVIE: AtomIdent = AtomIdent(*b"moov");
/// (`mvhd`) Identifier of an atom containing information about the whole movie (or audio file).
pub const MOVIE_HEADER: AtomIdent = AtomIdent(*b"mvhd");
/// (`trak`) Identifier of an atom containing information about a single track.
pub const TRACK: AtomIdent = AtomIdent(*b"trak");
/// (`mdia`) Identifier of an atom containing information about a tracks media type and data.
pub const MEDIA: AtomIdent = AtomIdent(*b"mdia");
/// (`mdhd`) Identifier of an atom containing information about a track
pub const MEDIA_HEADER: AtomIdent = AtomIdent(*b"mdhd");
/// (`minf`)
pub const METADATA_INFORMATION: AtomIdent = AtomIdent(*b"minf");
/// (`stbl`)
pub const SAMPLE_TABLE: AtomIdent = AtomIdent(*b"stbl");
/// (`stco`)
pub const SAMPLE_TABLE_CHUNK_OFFSET: AtomIdent = AtomIdent(*b"stco");
/// (`udta`) Identifier of an atom containing user metadata.
pub const USER_DATA: AtomIdent = AtomIdent(*b"udta");
/// (`meta`) Identifier of an atom containing a metadata item list.
pub const METADATA: AtomIdent = AtomIdent(*b"meta");
/// (`ilst`) Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: AtomIdent = AtomIdent(*b"ilst");
/// (`data`) Identifier of an atom containing typed data.
pub const DATA: AtomIdent = AtomIdent(*b"data");
/// (`mean`)
pub const MEAN: AtomIdent = AtomIdent(*b"mean");
/// (`name`)
pub const NAME: AtomIdent = AtomIdent(*b"name");
/// (`free`)
pub const FREE: AtomIdent = AtomIdent(*b"free");

/// (`----`)
pub const FREEFORM: AtomIdent = AtomIdent(*b"----");

// iTunes 4.0 atoms
/// (`rtng`)
pub const ADVISORY_RATING: AtomIdent = AtomIdent(*b"rtng");
/// (`©alb`)
pub const ALBUM: AtomIdent = AtomIdent(*b"\xa9alb");
/// (`aART`)
pub const ALBUM_ARTIST: AtomIdent = AtomIdent(*b"aART");
/// (`©ART`)
pub const ARTIST: AtomIdent = AtomIdent(*b"\xa9ART");
/// (`covr`)
pub const ARTWORK: AtomIdent = AtomIdent(*b"covr");
/// (`tmpo`)
pub const BPM: AtomIdent = AtomIdent(*b"tmpo");
/// (`©cmt`)
pub const COMMENT: AtomIdent = AtomIdent(*b"\xa9cmt");
/// (`cpil`)
pub const COMPILATION: AtomIdent = AtomIdent(*b"cpil");
/// (`©wrt`)
pub const COMPOSER: AtomIdent = AtomIdent(*b"\xa9wrt");
/// (`cprt`)
pub const COPYRIGHT: AtomIdent = AtomIdent(*b"cprt");
/// (`©gen`)
pub const CUSTOM_GENRE: AtomIdent = AtomIdent(*b"\xa9gen");
/// (`disk`)
pub const DISC_NUMBER: AtomIdent = AtomIdent(*b"disk");
/// (`©too`)
pub const ENCODER: AtomIdent = AtomIdent(*b"\xa9too");
/// (`gnre`)
pub const STANDARD_GENRE: AtomIdent = AtomIdent(*b"gnre");
/// (`©nam`)
pub const TITLE: AtomIdent = AtomIdent(*b"\xa9nam");
/// (`trkn`)
pub const TRACK_NUMBER: AtomIdent = AtomIdent(*b"trkn");
/// (`©day`)
pub const YEAR: AtomIdent = AtomIdent(*b"\xa9day");

// iTunes 4.2 atoms
/// (`©grp`)
pub const GROUPING: AtomIdent = AtomIdent(*b"\xa9grp");
/// (`stik`)
pub const MEDIA_TYPE: AtomIdent = AtomIdent(*b"stik");

// iTunes 4.9 atoms
/// (`catg`)
pub const CATEGORY: AtomIdent = AtomIdent(*b"catg");
/// (`keyw`)
pub const KEYWORD: AtomIdent = AtomIdent(*b"keyw");
/// (`pcst`)
pub const PODCAST: AtomIdent = AtomIdent(*b"pcst");
/// (`egid`)
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: AtomIdent = AtomIdent(*b"egid");
/// (`purl`)
pub const PODCAST_URL: AtomIdent = AtomIdent(*b"purl");

// iTunes 5.0
/// (`desc`)
pub const DESCRIPTION: AtomIdent = AtomIdent(*b"desc");
/// (`©lyr`)
pub const LYRICS: AtomIdent = AtomIdent(*b"\xa9lyr");

// iTunes 6.0
/// (`tves`)
pub const TV_EPISODE: AtomIdent = AtomIdent(*b"tves");
/// (`tven`)
pub const TV_EPISODE_NUMBER: AtomIdent = AtomIdent(*b"tven");
/// (`tvnn`)
pub const TV_NETWORK_NAME: AtomIdent = AtomIdent(*b"tvnn");
/// (`tvsn`)
pub const TV_SEASON: AtomIdent = AtomIdent(*b"tvsn");
/// (`tvsh`)
pub const TV_SHOW_NAME: AtomIdent = AtomIdent(*b"tvsh");

// iTunes 6.0.2
/// (`purd`)
pub const PURCHASE_DATE: AtomIdent = AtomIdent(*b"purd");

// iTunes 7.0
/// (`pgap`)
pub const GAPLESS_PLAYBACK: AtomIdent = AtomIdent(*b"pgap");

// Work, Movement
/// (`©mvn`)
pub const MOVEMENT: AtomIdent = AtomIdent(*b"\xa9mvn");
/// (`©mvc`)
pub const MOVEMENT_COUNT: AtomIdent = AtomIdent(*b"\xa9mvc");
/// (`©mvi`)
pub const MOVEMENT_INDEX: AtomIdent = AtomIdent(*b"\xa9mvi");
/// (`©wrk`)
pub const WORK: AtomIdent = AtomIdent(*b"\xa9wrk");
/// (`shwm`)
pub const SHOW_MOVEMENT: AtomIdent = AtomIdent(*b"shwm");

/// A 4 byte atom identifier.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct AtomIdent(pub [u8; 4]);

impl Deref for AtomIdent {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for AtomIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident({})", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

impl fmt::Display for AtomIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(|b| char::from(*b)).collect::<String>())
    }
}

/// A identifier for atoms
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ident {
    /// A standard identifier containing just an atom identifier.
    Std(AtomIdent),
    /// A identifier of a freeform (`----`) atom containing it's mean and name strings.
    Freeform {
        /// The mean string, typically in reverse domain notation.
        mean: String,
        /// The name string actually used to identify the freeform atom.
        name: String,
    },
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Std(ident) => write!(f, "{}", ident),
            Self::Freeform { mean, name } => write!(f, "{}:{}", mean, name),
        }
    }
}

impl Ident {
    /// Creates a new identifier of type [`Ident::Freeform`](Self::Freeform) containing the atom
    /// identifier, mean, and name.
    pub fn freeform(mean: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Freeform { mean: mean.into(), name: name.into() }
    }

    /// Creates a new identifier of type [`Ident::Std`](Self::Std) containing an atom identifier
    /// with the 4-byte identifier.
    pub const fn bytes(bytes: [u8; 4]) -> Self {
        Self::Std(AtomIdent(bytes))
    }
}
