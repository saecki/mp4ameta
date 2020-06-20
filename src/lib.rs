//! A library to read ITunes style MPEG-4 audio metadata.

extern crate byteorder;
extern crate core;

pub use crate::atom::Atom;
pub use crate::content::Content;
pub use crate::data::Data;
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::Tag;

pub mod atom;
pub mod content;
pub mod data;
pub mod error;
pub mod tag;
#[cfg(test)]
mod test;
