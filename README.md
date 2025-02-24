# mp4ameta
[![Build](https://github.com/Saecki/mp4ameta/actions/workflows/build.yml/badge.svg)](https://github.com/Saecki/mp4ameta/actions/workflows/build.yml)
[![Crate](https://img.shields.io/crates/v/mp4ameta.svg)](https://crates.io/crates/mp4ameta)
[![Documentation](https://img.shields.io/docsrs/mp4ameta?label=docs.rs)](https://docs.rs/mp4ameta)
![License](https://img.shields.io/crates/l/mp4ameta)
![LOC](https://tokei.rs/b1/github/saecki/mp4ameta?category=code)

A library for reading and writing iTunes style MPEG-4 audio metadata.
Most commonly this kind of metadata is found inside `m4a` or `m4b` files but basically any `mp4` container supports it.

## Examples

### The Easy Way
```rs
let mut tag = mp4ameta::Tag::read_from_path("music.m4a").unwrap();

println!("{}", tag.artist().unwrap());

tag.set_artist("artist");
tag.write_to_path("music.m4a").unwrap();
```

### The Hard Way
```rs
use mp4ameta::{Data, Fourcc, Tag};

let mut tag = Tag::read_from_path("music.m4a").unwrap();
let artist_ident = Fourcc(*b"\xa9ART");

let artist = tag.strings_of(&artist_ident).next().unwrap();
println!("{}", artist);

tag.set_data(artist_ident, Data::Utf8("artist".to_owned()));
tag.write_to_path("music.m4a").unwrap();
```

### Using Freeform Identifiers
```rs
use mp4ameta::{Data, FreeformIdent, Tag};

let mut tag = Tag::read_from_path("music.m4a").unwrap();
let isrc_ident = FreeformIdent::new_static("com.apple.iTunes", "ISRC");

let isrc = tag.strings_of(&isrc_ident).next().unwrap();
println!("{}", isrc);

tag.set_data(isrc_ident, Data::Utf8("isrc".to_owned()));
tag.write_to_path("music.m4a").unwrap();
```

### Chapters
There are two ways of storing chapters in mp4 files.
They can either be stored inside a chapter list, or a chapter track.
```rs
use mp4ameta::{Chapter, Tag};
use std::time::Duration;

let mut tag = Tag::read_from_path("audiobook.m4b").unwrap();

for chapter in tag.chapter_track() {
    let mins = chapter.start.as_secs() / 60;
    let secs = chapter.start.as_secs() % 60;
    println!("{mins:02}:{secs:02} {}", chapter.title);
}
tag.chapter_track_mut().clear();

tag.chapter_list_mut().extend([
    Chapter::new(Duration::ZERO, "first chapter"),
    Chapter::new(Duration::from_secs(3 * 60 + 42), "second chapter"),
    Chapter::new(Duration::from_secs(7 * 60 + 13), "third chapter"),
]);

tag.write_to_path("audiobook.m4b").unwrap();
```

### Read and Write Configurations
Read only the data that is relevant for your usecase.
And (over)write only the data that you want to edit.

By default all data is read and written.
```rs
use mp4ameta::{ChplTimescale, ReadConfig, Tag, WriteConfig};

// Only read the metadata item list, not chapters or audio information
let read_cfg = ReadConfig {
    read_meta_items: true,
    read_image_data: false,
    read_chapter_list: false,
    read_chapter_track: false,
    read_audio_info: false,
    chpl_timescale: ChplTimescale::DEFAULT,
};
let mut tag = Tag::read_with_path("music.m4a", &read_cfg).unwrap();

println!("{tag}");

tag.clear_meta_items();

// Only overwrite the metadata item list, leave chapters intact
let write_cfg = WriteConfig {
    write_meta_items: true,
    write_chapter_list: false,
    write_chapter_track: false,
    chpl_timescale: ChplTimescale::DEFAULT,
};
tag.write_with_path("music.m4a", &write_cfg).unwrap();
```

## Useful Links
- QuickTime spec
    - [Overview of QTFF](https://developer.apple.com/documentation/quicktime-file-format)
    - [Movie atoms](https://developer.apple.com/documentation/quicktime-file-format/movie_atoms)
    - [User data atoms](https://developer.apple.com/documentation/quicktime-file-format/user_data_atoms)
- [MultimediaWiki QuickTime container](https://wiki.multimedia.cx/index.php/QuickTime_container)
- [AtomicParsley docs](http://atomicparsley.sourceforge.net/mpeg-4files.html)
- [Mutagen docs](https://mutagen.readthedocs.io/en/latest/api/mp4.html)
- [Hydrogen audio tag mapping](https://wiki.hydrogenaud.io/index.php?title=Tag_Mapping)
- [MusicBrainz Picard tag mapping](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html)
- [Filetype list](https://ftyps.com/)

## Testing
__Run all tests:__<br/>
`cargo test`

__Test this library on your collection:__<br/>
`cargo test -- --nocapture collection <path>`

