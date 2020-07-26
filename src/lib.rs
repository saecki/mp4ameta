//! A library for reading and writing iTunes style MPEG-4 audio metadata.
#![warn(missing_docs)]

extern crate byteorder;
extern crate core;

pub use crate::atom::Atom;
pub use crate::content::Content;
pub use crate::data::Data;
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::Tag;

mod atom;
mod content;
mod data;
mod error;
mod tag;
