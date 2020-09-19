//! A library for reading and writing iTunes style MPEG-4 audio metadata.
#![warn(missing_docs)]

extern crate byteorder;
extern crate core;

pub use crate::atom::{Atom, AtomT, Ident};
pub use crate::content::{Content, ContentT};
pub use crate::data::{Data, DataT};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::Tag;
pub use crate::types::{AdvisoryRating, MediaType};

mod atom;
mod content;
mod data;
mod error;
mod tag;
mod types;
