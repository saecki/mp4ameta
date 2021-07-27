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
#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces
)]
pub use crate::atom::ident::*;
pub use crate::atom::{ident, Data, ReadConfig, WriteConfig};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::tag::{Tag, STANDARD_GENRES};
pub use crate::types::*;

pub(crate) use crate::atom::MetaItem;

#[macro_use]
mod atom;
mod error;
mod tag;
mod types;
mod util;
