use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{Atom, atom, Content, Data};

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Debug)]
pub struct Tag {
    /// A vector containing metadata atoms
    pub atom: Atom,
}

impl Tag {
    /// Creates a new empty `Tag`.
    pub fn new() -> Tag {
        Tag { atom: Atom::new() }
    }

    /// Creates a new `Tag` containing the `Atom`.
    pub fn with(atom: Atom) -> Tag {
        Tag { atom }
    }

    /// Attempts to read a MP4 `Tag` from the reader.
    pub fn read_from(reader: &mut impl Read) -> crate::Result<Tag> {
        Atom::read_from(reader)
    }

    /// Attempts to read a MP4 `Tag` from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }

    /// Attempts to return the album `String` (©alb).
    pub fn album(&self) -> Option<String> { self.get_string(atom::ALBUM) }

    /// Attempts to return the album artist `String` (aART).
    pub fn album_artist(&self) -> Option<String> {
        self.get_string(atom::ALBUM_ARTIST)
    }

    /// Attempts to return the artist `String` (©ART).
    pub fn artist(&self) -> Option<String> {
        self.get_string(atom::ARTIST)
    }

    /// Return the lyrics `String` (©lyr).
    pub fn lyrics(&self) -> Option<String> {
        self.get_string(atom::LYRICS)
    }

    /// Attempts to return the title `String` (©nam).
    pub fn title(&self) -> Option<String> {
        self.get_string(atom::TITLE)
    }

    /// Attempts to return the year `String` (©day).
    pub fn year(&self) -> Option<String> {
        self.get_string(atom::YEAR)
    }

    /// Attempts to return the genre `String` (©gen) or (gnre).
    pub fn genre(&self) -> Option<String> {
        if let Some(s) = self.get_string(atom::GENRE) {
            return Some(s);
        }

        if let Some(v) = self.get_unknown(atom::GENERIC_GENRE) {
            let mut chunks = v.chunks(2);

            if let Ok(genre_code) = chunks.next()?.read_u16::<BigEndian>() {
                for g in atom::GENRES.iter() {
                    if g.0 == genre_code {
                        return Some(String::from(g.1));
                    }
                }
            };
        }

        None
    }

    /// Attempts to return the track number and the total number of tracks (trkn).
    pub fn track_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.get_unknown(atom::TRACK_NUMBER) {
            Some(v) => v,
            None => return (None, None),
        };

        let mut buffs = Vec::new();

        for chunk in vec.chunks(2) {
            buffs.push(chunk);
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

    /// Attempts to return the disk number and total number of disks (disk).
    pub fn disk_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.get_unknown(atom::DISK_NUMBER) {
            Some(v) => v,
            None => return (None, None),
        };

        let mut buffs = Vec::new();

        for chunk in vec.chunks(2) {
            buffs.push(chunk);
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

    /// Attempts to return the artwork image `Data` of type `Data::JPEG` or `Data::PNG` (covr).
    pub fn artwork(&self) -> Option<Data> {
        self.get_image(atom::ARTWORK)
    }

    /// Attempts to return a `Vec<u8>` containing byte data corresponding to the provided head.
    pub fn get_unknown(&self, head: [u8; 4]) -> Option<Vec<u8>> {
        match self.get_data(head) {
            Some(Data::Unknown(Ok(v))) => Some(v.to_vec()),
            _ => None,
        }
    }

    /// Attempts to return a string corresponding to the provided head.
    pub fn get_string(&self, head: [u8; 4]) -> Option<String> {
        let d = self.get_data(head)?;

        match d {
            Data::UTF8(Ok(s)) => Some(String::from(s)),
            Data::UTF16(Ok(s)) => Some(String::from(s)),
            _ => None,
        }
    }

    /// Attempts to return image `Data` of type `Data::JPEG` or `Data::PNG` corresponding to the provided head.
    pub fn get_image(&self, head: [u8; 4]) -> Option<Data> {
        let d = self.get_data(head)?;

        match d {
            Data::JPEG(Ok(d)) => Some(Data::JPEG(Ok(d.to_vec()))),
            Data::PNG(Ok(d)) => Some(Data::PNG(Ok(d.to_vec()))),
            _ => None,
        }
    }

    /// Attempts to return the `Data` corresponding to the provided head.
    pub fn get_data(&self, head: [u8; 4]) -> Option<&Data> {
        if let Content::Atoms(v) = &self.atom.first_child()?.first_child()?.first_child()?.content {
            for a in v {
                if a.head == head {
                    if let Content::TypedData(data) = &a.first_child()?.content {
                        return Some(data);
                    }
                }
            }
        }

        None
    }
}

#[test]
fn test() {
    let tag = Tag::read_from_path("/mnt/data/Music/Three Days Grace - Human/1 - Three Days Grace - Human Race.m4a");

    match tag {
        Ok(t) => {
            println!("tag: {:#?}", t);
            println!("album: {:?}", t.album());
            println!("album artist: {:?}", t.album_artist());
            println!("artist: {:?}", t.artist());
            println!("disk number: {:?}", t.disk_number());
            println!("genre: {:?}", t.genre());
            println!("lyrics: {:?}", t.lyrics());
            println!("title: {:?}", t.title());
            println!("track number: {:?}", t.track_number());
            println!("year: {:?}", t.year());

            match t.artwork() {
                Some(Data::JPEG(Ok(v))) => {
                    let mut writer = BufWriter::new(File::create("./cover.jpg").unwrap());
                    writer.write(&v).expect("error writing artwork");
                }
                Some(Data::PNG(Ok(v))) => {
                    let mut writer = BufWriter::new(File::create("./cover.png").unwrap());
                    writer.write(&v).expect("error writing artwork");
                }
                _ => (),
            }
        }
        Err(e) => panic!("error: {:#?}", e),
    }
}