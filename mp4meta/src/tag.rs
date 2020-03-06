use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::{Atom, atom, Content, Data};

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Debug)]
pub struct Tag {
    /// A vector containing metadata atoms
    pub atom: Atom,
}

impl Tag {
    /// Creates a new empty Tag
    pub fn new() -> Tag {
        Tag { atom: Atom::new() }
    }

    pub fn with(atom: Atom) -> Tag {
        Tag { atom }
    }

    /// Attempts to read a MP4 tag from the reader.
    pub fn read_from(reader: &mut BufReader<File>) -> crate::Result<Tag> {
        Atom::read_from(reader)
    }

    /// Attempts to read a MP4 tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }

    pub fn get_utf8(&self, head: [u8; 4]) -> Option<String> {
        if let Content::Atoms(v) = &self.atom.first_child()?.first_child()?.first_child()?.content {
            for a in v {
                if a.head == head {
                    if let Content::TypedData(Data::UTF8(Ok(s))) = &a.first_child()?.content {
                        return Some(String::from(s));
                    }
                }
            }
        }

        None
    }

    pub fn get_unknown(&self, head: [u8; 4]) -> Option<Vec<u8>> {
        if let Content::Atoms(v) = &self.atom.first_child()?.first_child()?.first_child()?.content {
            for a in v {
                if a.head == head {
                    if let Content::TypedData(Data::Unknown(Ok(v))) = &a.first_child()?.content {
                        return Some(v.to_vec());
                    }
                }
            }
        }

        None
    }


    pub fn title(&self) -> Option<String> {
        self.get_utf8(atom::TITLE)
    }

    pub fn artist(&self) -> Option<String> {
        self.get_utf8(atom::ARTIST)
    }

    pub fn album_artist(&self) -> Option<String> {
        self.get_utf8(atom::ALBUM_ARTIST)
    }

    pub fn album(&self) -> Option<String> {
        self.get_utf8(atom::ALBUM)
    }

    pub fn genre(&self) -> Option<String> {
        self.get_utf8(atom::GENRE)
    }

    pub fn year(&self) -> Option<String> {
        self.get_utf8(atom::YEAR)
    }

    pub fn track_number(&self) -> Option<(u32, u32)> {
        let vec = self.get_unknown(atom::TRACK_NUMBER);
    }
}

#[test]
fn test() {
    let tag = Tag::read_from_path("/mnt/data/Music/Three Days Grace - Human/1 - Three Days Grace - Human Race.m4a");

    match tag {
        Ok(t) => {
            println!("tag: {:#?}", t);
            println!("title: {:?}", t.title());
            println!("artist: {:?}", t.artist());
            println!("album artist: {:?}", t.album_artist());
            println!("album: {:?}", t.album());
            println!("genre: {:?}", t.genre());
            println!("year: {:?}", t.year());
        }
        Err(e) => panic!("error: {:#?}", e),
    }
}