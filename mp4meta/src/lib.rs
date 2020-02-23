pub use crate::atom::Atom;
pub use crate::tag::Tag;
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::content::{Content, Data};

mod tag;
mod atom;
mod content;
mod error;