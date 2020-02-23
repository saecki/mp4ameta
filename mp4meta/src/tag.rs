use std::path::Path;
use std::io::{BufReader};
use std::fs::File;
use crate::Atom;

pub struct Tag {
    atoms: Vec<Atom>
}

impl Tag {
    pub fn new() -> Tag {
        Tag { atoms: Vec::new() }
    }

    /// Attempts to read a MP4 tag from the reader.
    pub fn read_from(reader: &mut BufReader<File>) -> crate::error::Result<Tag> {
        Atom::read_from(reader);

        Ok(Tag::new())
    }

    /// Attempts to read a MP4 tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::error::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }
}

#[test]
fn test() {
        let tag = Tag::read_from_path("/mnt/data/Music/Slipknot - Unsainted.m4a");

    if let Ok(t) = tag {
        println!("wow");
    }
}