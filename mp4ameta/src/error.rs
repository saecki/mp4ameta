use std::{error, fmt, io, str, string};

/// Type alias for the result of tag operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error kind indicating that an atom could not be found. Contains the atom's head.
    AtomNotFound([u8; 4]),
    /// An error kind indicating that an IO error has occurred. Contains the original io::Error.
    Io(io::Error),
    /// An error kind indicating that the reader does not contain mp4 metadata.
    NoTag,
    /// An error kind indicating that an error accured during parsing.
    Parsing,
    /// An error kind indicating that the `Content::TypedData` contains an unknown datatype.
    /// Contains the unknown datatype code.
    UnknownDataType(i32),
    /// An error kind indicating that the data can't be written to a file.
    UnWritableDataType,
    /// An error kind indicating that a string decoding error has occurred. Contains the invalid
    /// data.
    Utf8StringDecoding(string::FromUtf8Error),
    /// An error kind indicating that a string decoding error has occurred.
    Utf16StringDecoding(string::FromUtf16Error),
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
            kind: ErrorKind::Utf8StringDecoding(err),
            description: "Data is not valid utf-8.",
        }
    }
}

impl From<string::FromUtf16Error> for Error {
    fn from(err: string::FromUtf16Error) -> Error {
        Error {
            kind: ErrorKind::Utf16StringDecoding(err),
            description: "Data is not valid utf-16.",
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(f, "{:?}: {}", self.kind, self.description)
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(f, "{:?}: {}", self.kind, self.description)
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}