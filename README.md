# rust-mp4ameta
[![Crate](https://img.shields.io/crates/v/id3.svg)](https://crates.io/crates/mp4ameta)
[![Documentation](https://docs.rs/id3/badge.svg)](https://docs.rs/mp4ameta)

A library for reading and writing MPEG-4 audio metadata.

## Usage
```rust
fn main() {
  	let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();

  	println!("{}", tag.artist().unwrap());

  	tag.set_artist("<artist>");

  	tag.write_to_path("music.m4a").unwrap();
}
```

## Supported Filetypes
- M4A
- M4B