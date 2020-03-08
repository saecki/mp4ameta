use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{Atom, atom, Content, Data};

/// List of standard genres found in the `gnre` `Atom`.
pub const GENRES: [(u16, &str); 80] = [
    (1, "Blues"),
    (2, "Classic rock"),
    (3, "Country"),
    (4, "Dance"),
    (5, "Disco"),
    (6, "Funk"),
    (7, "Grunge"),
    (8, "Hip,-Hop"),
    (9, "Jazz"),
    (10, "Metal"),
    (11, "New Age"),
    (12, "Oldies"),
    (13, "Other"),
    (14, "Pop"),
    (15, "Rhythm and Blues"),
    (16, "Rap"),
    (17, "Reggae"),
    (18, "Rock"),
    (19, "Techno"),
    (20, "Industrial"),
    (21, "Alternative"),
    (22, "Ska"),
    (23, "Death metal"),
    (24, "Pranks"),
    (25, "Soundtrack"),
    (26, "Euro-Techno"),
    (27, "Ambient"),
    (28, "Trip-Hop"),
    (29, "Vocal"),
    (30, "Jazz & Funk"),
    (31, "Fusion"),
    (32, "Trance"),
    (33, "Classical"),
    (34, "Instrumental"),
    (35, "Acid"),
    (36, "House"),
    (37, "Game"),
    (38, "Sound clip"),
    (39, "Gospel"),
    (40, "Noise"),
    (41, "Alternative Rock"),
    (42, "Bass"),
    (43, "Soul"),
    (44, "Punk"),
    (45, "Space"),
    (46, "Meditative"),
    (47, "Instrumental Pop"),
    (48, "Instrumental Rock"),
    (49, "Ethnic"),
    (50, "Gothic"),
    (51, "Darkwave"),
    (52, "Techno-Industrial"),
    (53, "Electronic"),
    (54, "Pop-Folk"),
    (55, "Eurodance"),
    (56, "Dream"),
    (57, "Southern Rock"),
    (58, "Comedy"),
    (59, "Cult"),
    (60, "Gangsta"),
    (61, "Top 41"),
    (62, "Christian Rap"),
    (63, "Pop/Funk"),
    (64, "Jungle"),
    (65, "Native US"),
    (66, "Cabaret"),
    (67, "New Wave"),
    (68, "Psychedelic"),
    (69, "Rave"),
    (70, "Show tunes"),
    (71, "Trailer"),
    (72, "Lo,-Fi"),
    (73, "Tribal"),
    (74, "Acid Punk"),
    (75, "Acid Jazz"),
    (76, "Polka"),
    (77, "Retro"),
    (78, "Musical"),
    (79, "Rock ’n’ Roll"),
    (80, "Hard Rock"),
];

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Debug)]
pub struct Tag {
    /// A vector containing metadata atoms
    pub atom: Atom,
}

impl Tag {
    /// Creates a new empty MPEG-4 tag.
    pub fn new() -> Tag {
        Tag { atom: Atom::new() }
    }

    /// Creates a new MPEG-4 tag containing the atom.
    pub fn with(atom: Atom) -> Tag {
        Tag { atom }
    }

    /// Attempts to read a MPEG-4 tag from the reader.
    pub fn read_from(reader: &mut impl Read) -> crate::Result<Tag> {
        Atom::read_from(reader)
    }

    /// Attempts to read a MPEG-4 tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }

    /// Attempts to return the album (©alb).
    pub fn album(&self) -> Option<String> { self.get_string(atom::ALBUM) }

    /// Attempts to return the album artist (aART).
    pub fn album_artist(&self) -> Option<String> {
        self.get_string(atom::ALBUM_ARTIST)
    }

    /// Attempts to return the artist (©ART).
    pub fn artist(&self) -> Option<String> {
        self.get_string(atom::ARTIST)
    }

    /// Attempts to return the lyrics (©lyr).
    pub fn lyrics(&self) -> Option<String> {
        self.get_string(atom::LYRICS)
    }

    /// Attempts to return the title (©nam).
    pub fn title(&self) -> Option<String> {
        self.get_string(atom::TITLE)
    }

    /// Attempts to return the year (©day).
    pub fn year(&self) -> Option<String> {
        self.get_string(atom::YEAR)
    }

    /// Attempts to return the genre (©gen) or (gnre).
    pub fn genre(&self) -> Option<String> {
        if let Some(s) = self.get_string(atom::GENRE) {
            return Some(s);
        }

        if let Some(v) = self.get_reserved(atom::GENERIC_GENRE) {
            let mut chunks = v.chunks(2);

            if let Ok(genre_code) = chunks.next()?.read_u16::<BigEndian>() {
                for g in GENRES.iter() {
                    if g.0 == genre_code {
                        return Some(String::from(g.1));
                    }
                }
            }
        }

        None
    }

    /// Attempts to return the track number and the total number of tracks (trkn).
    pub fn track_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.get_reserved(atom::TRACK_NUMBER) {
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
        let vec = match self.get_reserved(atom::DISK_NUMBER) {
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

    /// Attempts to return byte data corresponding to the provided head.
    pub fn get_reserved(&self, head: [u8; 4]) -> Option<Vec<u8>> {
        match self.get_data(head) {
            Some(Data::Reserved(Ok(v))) => Some(v.to_vec()),
            _ => None,
        }
    }

    /// Attempts to return a `String` corresponding to the provided head.
    pub fn get_string(&self, head: [u8; 4]) -> Option<String> {
        let d = self.get_data(head)?;

        match d {
            Data::Utf8(Ok(s)) => Some(String::from(s)),
            Data::Utf16(Ok(s)) => Some(String::from(s)),
            _ => None,
        }
    }

    /// Attempts to return image `Data` of type `Data::JPEG` or `Data::PNG` corresponding to the provided head.
    pub fn get_image(&self, head: [u8; 4]) -> Option<Data> {
        let d = self.get_data(head)?;

        match d {
            Data::Jpeg(Ok(d)) => Some(Data::Jpeg(Ok(d.to_vec()))),
            Data::Png(Ok(d)) => Some(Data::Png(Ok(d.to_vec()))),
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
                Some(Data::Jpeg(Ok(v))) => {
                    let mut writer = BufWriter::new(File::create("./cover.jpg").unwrap());
                    writer.write(&v).expect("error writing artwork");
                }
                Some(Data::Png(Ok(v))) => {
                    let mut writer = BufWriter::new(File::create("./cover.png").unwrap());
                    writer.write(&v).expect("error writing artwork");
                }
                _ => (),
            }
        }
        Err(e) => panic!("error: {:#?}", e),
    }
}