use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::Atom;
use std::fmt::Debug;

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Debug)]
pub struct Tag {
    /// A vector containing metadata atoms
    atoms: Vec<Atom>,
}

impl<'a> Tag {
    /// Creates a new empty Tag
    pub fn new() -> Tag {
        Tag { atoms: vec![] }
    }

    pub fn with(atoms: Vec<Atom>) -> Tag { Tag { atoms } }

    /// Attempts to read a MP4 tag from the reader.
    pub fn read_from(reader: &mut BufReader<File>) -> crate::Result<Tag> {
        Atom::read_from(reader)
    }

    /// Attempts to read a MP4 tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }
}

#[test]
fn test() {
    let tag = Tag::read_from_path("/mnt/data/Music/Three Days Grace - Human/10 - Three Days Grace - One Too Many.m4a");

    match tag {
        Ok(t) => println!("tag: {:?}", t),
        Err(e) => println!("error: {:?}", e),
    }
}