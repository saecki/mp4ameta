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
//!
//! tag.write_to_path("music.m4a").unwrap();
//! ```
//!
//! ## The hard way
//! ```no_run
//! use mp4ameta::{atom, Data, FourCC, Tag};
//!
//! let mut tag = Tag::read_from_path("music.m4a").unwrap();
//! let artist_ident = FourCC(*b"\xa9ART");
//!
//! let artist = tag.string(&artist_ident).next().unwrap();
//! println!("{}", artist);
//!
//! tag.set_data(artist_ident, Data::Utf8("artist".to_owned()));
//!
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
//! let isrc = tag.string(&isrc_ident).next().unwrap();
//! println!("{}", isrc);
//!
//! tag.set_data(isrc_ident, Data::Utf8("isrc".to_owned()));
//!
//! tag.write_to_path("music.m4a").unwrap();
//! ```
#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub use crate::core::atom::{self, Atom, AtomData, AtomT, DataIdent, FourCC, FreeformIdent, Ident};
pub use crate::core::content::{Content, ContentT};
pub use crate::core::data::{self, Data};
pub use crate::core::types::{self, AdvisoryRating, ChannelConfig, MediaType, SampleRate};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::{Tag, STANDARD_GENRES};

#[macro_use]
mod core;
mod error;
mod tag;
