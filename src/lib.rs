//! A library for reading and writing iTunes style MPEG-4 audio metadata.
//!
//! # Examples
//!
//! ## The easy way
//! ```no_run
//! let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();
//!
//! println!("{}", tag.artist().unwrap());
//!
//! tag.set_artist("artist");
//! tag.write_to_path("music.m4a").unwrap();
//! ```
//!
//! ## The hard way
//! ```no_run
//! use mp4ameta::{Data, Fourcc, Tag};
//!
//! let mut tag = Tag::read_from_path("music.m4a").unwrap();
//! let artist_ident = Fourcc(*b"\xa9ART");
//!
//! let artist = tag.strings_of(&artist_ident).next().unwrap();
//! println!("{}", artist);
//!
//! tag.set_data(artist_ident, Data::Utf8("artist".to_owned()));
//! tag.write_to_path("music.m4a").unwrap();
//! ```
//!
//! ## Using freeform identifiers
//! ```no_run
//! use mp4ameta::{Data, FreeformIdent, Tag};
//!
//! let mut tag = Tag::read_from_path("music.m4a").unwrap();
//! let isrc_ident = FreeformIdent::new("com.apple.iTunes", "ISRC");
//!
//! let isrc = tag.strings_of(&isrc_ident).next().unwrap();
//! println!("{}", isrc);
//!
//! tag.set_data(isrc_ident, Data::Utf8("isrc".to_owned()));
//! tag.write_to_path("music.m4a").unwrap();
//! ```
#![deny(rust_2018_idioms)]

pub use crate::atom::ident::{self, DataIdent, Fourcc, FreeformIdent, Ident};
pub use crate::atom::{ChplTimescale, Data, ReadConfig, WriteConfig};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::{Tag, Userdata, STANDARD_GENRES};
pub use crate::types::*;

pub(crate) use crate::atom::MetaItem;

#[macro_use]
mod atom;
mod error;
mod tag;
mod types;
mod util;
