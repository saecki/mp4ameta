use std::borrow::Cow;
use std::{error, fmt, io, string};

use crate::Fourcc;

/// Type alias for the result of tag operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug)]
pub enum ErrorKind {
    /// An atom could not be found. Contains the atom's identifier.
    AtomNotFound(Fourcc),
    /// A descriptor could not be found. Contains the descriptor's tag.
    DescriptorNotFound(u8),
    /// No filetype (`ftyp`) atom, which indicates na MPEG-4 file, could be found.
    NoFtyp,
    /// The size of an atom is smaller than its header.
    InvalidAtomSize,
    /// The content of an atom suggests another length than its header.
    SizeMismatch,
    /// The [`ChannelConfig`] code is unknown. Contains the unknown code.
    ///
    /// [`ChannelConfig`]: crate::ChannelConfig
    UnknownChannelConfig(u8),
    /// The [`MediaType`] code is unknown. Contains the unknown code.
    ///
    /// [`MediaType`]: crate::MediaType
    UnknownMediaType(u8),
    /// The [`SampleRate`] index is unknown. Contains the unknown index.
    ///
    /// [`SampleRate`]: crate::SampleRate
    UnknownSampleRate(u8),
    ChapterTitleTooLong(usize),
    /// Either the version byte of an atom or a descriptor is unknown. Contains the unknown version.
    UnknownVersion(u8),
    /// An invalid utf-8 string was found. Contains the invalid data.
    Utf8StringDecoding(string::FromUtf8Error),
    /// An invalid utf-16 string was found.
    Utf16StringDecoding(string::FromUtf16Error),
    /// An IO error has occurred.
    Io(io::Error),
}

/// Any error that may occur while performing metadata operations.
pub struct Error {
    /// The kind of error that occurred.
    pub kind: ErrorKind,
    /// A human readable string describing the error.
    pub description: Cow<'static, str>,
}

impl Error {
    pub fn new(kind: ErrorKind, description: impl Into<Cow<'static, str>>) -> Error {
        Error { kind, description: description.into() }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        let description = format!("IO error: {err}");
        Error::new(ErrorKind::Io(err), description)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::new(ErrorKind::Utf8StringDecoding(err), "Data is not valid utf-8.")
    }
}

impl From<string::FromUtf16Error> for Error {
    fn from(err: string::FromUtf16Error) -> Error {
        Error::new(ErrorKind::Utf16StringDecoding(err), "Data is not valid utf-16.")
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.description.is_empty() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{}:\n{:?}", self.description, self.kind)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.description.is_empty() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{}:\n{:?}", self.description, self.kind)
        }
    }
}
