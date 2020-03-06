use std::{error, fmt, io, str, string};

pub type Result<T> = std::result::Result<T, Error>;

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error kind indicating that an IO error has occurred. Contains the original io::Error.
    Io(io::Error),
    /// An error kind indicating that a string decoding error has occurred. Contains the invalid
    /// data.
    Utf8StringDecoding(Vec<u8>),
    /// An error kind indicating that a string decoding error has occurred.
    Utf16StringDecoding,
    /// An error kind indicating that the reader does not contain mp4 metadata.
    NoTag,
    /// An error kind indicating that some input was invalid.
    InvalidInput,
    /// An error kind indicating that the typed data contains an unknown datatype. Contains the
    /// unknown datatype code.
    UnknownDataType(u32),
    /// An error kind indicating that the raw data is empty.
    EmptyData,
    /// An error kind indicating that an atom could not be found. Contains the atom's f.
    AtomNotFound([u8; 4]),
    /// An error kind indicating that an error accured during parsing.
    Parsing,
}

/// A structure able to represent any error that may occur while performing metadata operations.
pub struct Error {
    /// The kind of error.
    pub kind: ErrorKind,
    /// A human readable string describing the error.
    pub description: &'static str,
}

impl Error {
    /// Creates a new `Error` using the error kind and description.
    pub fn new(kind: ErrorKind, description: &'static str) -> Error {
        Error {
            kind,
            description,
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        if let Some(cause) = self.source() {
            cause.description()
        } else {
            match self.kind {
                ErrorKind::Io(ref err) => error::Error::description(err),
                _ => self.description,
            }
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            kind: ErrorKind::Io(err),
            description: "",
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error {
            kind: ErrorKind::Utf8StringDecoding(err.into_bytes()),
            description: "data is not valid utf-8",
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Error {
        Error {
            kind: ErrorKind::Utf8StringDecoding(vec![]),
            description: "data is not valid utf-8",
        }
    }
}

impl From<string::FromUtf16Error> for Error {
    fn from(_err: string::FromUtf16Error) -> Error {
        Error {
            kind: ErrorKind::Utf16StringDecoding,
            description: "data is not valid utf-16",
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(f, "{:?}: {}", self.kind, self.description)
        } else {
            write!(f, "{}", self.description)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(f, "{:?}: {}", self.kind, error::Error::description(self))
        } else {
            write!(f, "{}", error::Error::description(self))
        }
    }
}