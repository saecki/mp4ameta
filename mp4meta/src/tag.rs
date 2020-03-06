use byteorder::{BigEndian, ReadBytesExt};
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

    /// Returns the title (©nam).
    pub fn title(&self) -> Option<String> {
        self.get_utf8(atom::TITLE)
    }

    /// Returns the artist (©ART).
    pub fn artist(&self) -> Option<String> {
        self.get_utf8(atom::ARTIST)
    }

    /// Returns the album artist (aART).
    pub fn album_artist(&self) -> Option<String> {
        self.get_utf8(atom::ALBUM_ARTIST)
    }

    /// Returns the album (©alb).
    pub fn album(&self) -> Option<String> {
        self.get_utf8(atom::ALBUM)
    }

    /// Returns the genre (©gen).
    pub fn genre(&self) -> Option<String> {
        self.get_utf8(atom::GENRE)
    }

    /// Returns the year (©day).
    pub fn year(&self) -> Option<String> {
        self.get_utf8(atom::YEAR)
    }

    /// Return the lyrics (©lyr).
    pub fn lyrics(&self) -> Option<String> {
        self.get_utf8(atom::LYRICS)
    }

    /// Returns the track number and the total number of tracks (trkn).
    pub fn track_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.get_unknown(atom::TRACK_NUMBER) {
            Some(v) => v,
            None => return (None, None),
        };

        let mut buffs = Vec::new();

        for chunk in vec.chunks(2) {
            buffs.push(chunk)
        }

        let track_number = match buffs[1].read_u16::<BigEndian>() {
            Ok(tnr) => Some(tnr),
            Err(_) => None,
        };

        let total_tracks = match buffs[2].read_u16::<BigEndian>() {
            Ok(atr) => Some(atr),
            Err(_) => None,
        };

        (track_number, total_tracks)
    }

    /// Returns disk number and total number of disks (disk).
    pub fn disk_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.get_unknown(atom::DISK_NUMBER) {
            Some(v) => v,
            None => return (None, None),
        };

        let mut buffs = Vec::new();

        for chunk in vec.chunks(2) {
            buffs.push(chunk)
        }

        let disk_number = match buffs[1].read_u16::<BigEndian>() {
            Ok(tnr) => Some(tnr),
            Err(_) => None,
        };

        let total_disks = match buffs[2].read_u16::<BigEndian>() {
            Ok(atr) => Some(atr),
            Err(_) => None,
        };

        (disk_number, total_disks)
    }
}

#[test]
fn test() {
    let tag = Tag::read_from_path("/mnt/data/Music/SOiL - Redefine/10 - SOiL - Love Hate Game.m4a");

    match tag {
        Ok(t) => {
            println!("tag: {:#?}", t);
            println!("title: {:?}", t.title());
            println!("artist: {:?}", t.artist());
            println!("album artist: {:?}", t.album_artist());
            println!("album: {:?}", t.album());
            println!("genre: {:?}", t.genre());
            println!("year: {:?}", t.year());
            println!("track number: {:?}", t.track_number());
            println!("disk number: {:?}", t.disk_number());
        }
        Err(e) => panic!("error: {:#?}", e),
    }
}
