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

pub use crate::core::atom::{self, Atom, AtomData, AtomT, Ident};
pub use crate::core::content::{Content, ContentT};
pub use crate::core::data::{self, Data};
pub use crate::core::types::{self, AdvisoryRating, MediaType};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::{genre::STANDARD_GENRES, Tag};

#[macro_use]
mod core;
mod error;
mod tag;
