//! A library for reading and writing iTunes style MPEG-4 audio metadata.
//!
//! # Example
//!
//! ```no_run
//! let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();
//!
//! println!("{}", tag.artist().unwrap());
//!
//! tag.set_artist("artist");
//!
//! tag.write_to_path("music.m4a").unwrap();
//! ```
#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub use crate::atom::{Atom, AtomT, Ident};
pub use crate::content::{Content, ContentT};
pub use crate::data::{Data, DataT};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::{GENRES, Tag};
pub use crate::types::{AdvisoryRating, MediaType};

mod atom;
mod content;
mod data;
mod error;
mod tag;
mod types;
