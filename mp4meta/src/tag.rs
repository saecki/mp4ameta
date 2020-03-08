use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

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
    /// Creates a new empty MPEG-4 audio tag.
    pub fn new() -> Tag {
        Tag { atom: Atom::metadata_atom() }
    }

    /// Creates a new MPEG-4 audio tag containing the atom.
    pub fn with(atom: Atom) -> Tag {
        Tag { atom }
    }

    /// Attempts to read a MPEG-4 audio tag from the reader.
    pub fn read_from(reader: &mut impl Read) -> crate::Result<Tag> {
        Ok(Tag::with(Atom::read_from(reader)?))
    }

    /// Attempts to read a MPEG-4 audio tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }

    /// Attempts to write the MPEG-4 audio tag to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.atom.write_to(writer)
    }

    /// Attempts to write the MPEG-4 audio tag to the path.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let mut file = BufWriter::new(File::open(path)?);
        self.write_to(&mut file)
    }

    /// Returns the album (©alb).
    pub fn album(&self) -> Option<&str> { self.get_string(atom::ALBUM) }

    /// Sets the album (©alb).
    pub fn set_album(&mut self, album: impl Into<String>) {
        self.set_data(atom::ALBUM, Data::Utf8(Ok(album.into())));
    }

    /// Removes the album data (©alb).
    pub fn remove_album(&mut self) {
        self.remove_data(atom::ALBUM);
    }

    /// Returns the album artist (aART).
    pub fn album_artist(&self) -> Option<&str> {
        self.get_string(atom::ALBUM_ARTIST)
    }

    /// Sets the album artist (aART).
    pub fn set_album_artist(&mut self, album_artist: impl Into<String>) {
        self.set_data(atom::ALBUM_ARTIST, Data::Utf8(Ok(album_artist.into())));
    }

    /// Removes the album artist data (aART).
    pub fn remove_album_artist(&mut self) {
        self.remove_data(atom::ALBUM_ARTIST);
    }

    /// Returns the artist (©ART).
    pub fn artist(&self) -> Option<&str> {
        self.get_string(atom::ARTIST)
    }

    /// Sets the artist (©ART).
    pub fn set_artist(&mut self, artist: impl Into<String>) {
        self.set_data(atom::ARTIST, Data::Utf8(Ok(artist.into())));
    }

    /// Removes the artist data (©ART).
    pub fn remove_artist(&mut self) {
        self.remove_data(atom::ARTIST);
    }

    /// Returns the category (catg).
    pub fn category(&self) -> Option<&str> {
        self.get_string(atom::CATEGORY)
    }


    /// Sets the category (catg).
    pub fn set_category(&mut self, category: impl Into<String>) {
        self.set_data(atom::CATEGORY, Data::Utf8(Ok(category.into())));
    }

    /// Removes the category data (catg).
    pub fn remove_category(&mut self) {
        self.remove_data(atom::CATEGORY);
    }

    /// Returns the comment (©cmt).
    pub fn comment(&self) -> Option<&str> {
        self.get_string(atom::COMMENT)
    }

    /// Sets the comment (©cmt).
    pub fn set_comment(&mut self, comment: impl Into<String>) {
        self.set_data(atom::COMMENT, Data::Utf8(Ok(comment.into())));
    }

    /// Removes the comment data (©cmt).
    pub fn remove_comment(&mut self) {
        self.remove_data(atom::COMMENT);
    }

    /// Returns the composer (©wrt).
    pub fn composer(&self) -> Option<&str> {
        self.get_string(atom::COMPOSER)
    }

    /// Sets the composer (©wrt).
    pub fn set_composer(&mut self, composer: impl Into<String>) {
        self.set_data(atom::COMPOSER, Data::Utf8(Ok(composer.into())));
    }

    /// Removes the composer data (©wrt).
    pub fn remove_composer(&mut self) {
        self.remove_data(atom::COMMENT);
    }

    /// Returns the copyright (cprt).
    pub fn copyright(&self) -> Option<&str> {
        self.get_string(atom::COPYRIGHT)
    }

    /// Sets the copyright (cprt).
    pub fn set_copyright(&mut self, copyright: impl Into<String>) {
        self.set_data(atom::COPYRIGHT, Data::Utf8(Ok(copyright.into())));
    }

    /// Removes the copyright data (cprt).
    pub fn remove_copyright(&mut self) {
        self.remove_data(atom::COPYRIGHT);
    }

    /// Returns the description (desc).
    pub fn description(&self) -> Option<&str> {
        self.get_string(atom::DESCRIPTION)
    }

    /// Sets the description (desc).
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.set_data(atom::DESCRIPTION, Data::Utf8(Ok(description.into())));
    }

    /// Removes the description data (desc).
    pub fn remove_description(&mut self) {
        self.remove_data(atom::DESCRIPTION);
    }

    /// Returns the encoder (©too).
    pub fn encoder(&self) -> Option<&str> {
        self.get_string(atom::ENCODER)
    }

    /// Sets the encoder (©too).
    pub fn set_encoder(&mut self, encoder: impl Into<String>) {
        self.set_data(atom::ENCODER, Data::Utf8(Ok(encoder.into())));
    }

    /// Removes the encoder data (©too).
    pub fn remove_encoder(&mut self) {
        self.remove_data(atom::ENCODER);
    }

    /// Returns the grouping (©grp).
    pub fn grouping(&self) -> Option<&str> {
        self.get_string(atom::GROUPING)
    }

    /// Sets the grouping (©grp).
    pub fn set_grouping(&mut self, grouping: impl Into<String>) {
        self.set_data(atom::GROUPING, Data::Utf8(Ok(grouping.into())));
    }

    /// Removes the grouping data (©grp).
    pub fn remove_grouping(&mut self) {
        self.remove_data(atom::GROUPING);
    }

    /// Returns the keyword (keyw).
    pub fn keyword(&self) -> Option<&str> {
        self.get_string(atom::KEYWORD)
    }

    /// Sets the keyword (keyw).
    pub fn set_keyword(&mut self, keyword: impl Into<String>) {
        self.set_data(atom::KEYWORD, Data::Utf8(Ok(keyword.into())));
    }

    /// Removes the keyword data (keyw).
    pub fn remove_keyword(&mut self) {
        self.remove_data(atom::KEYWORD);
    }

    /// Returns the lyrics (©lyr).
    pub fn lyrics(&self) -> Option<&str> {
        self.get_string(atom::LYRICS)
    }

    /// Sets the lyrics (©lyr).
    pub fn set_lyrics(&mut self, lyrics: impl Into<String>) {
        self.set_data(atom::LYRICS, Data::Utf8(Ok(lyrics.into())));
    }

    /// Removes the lyrics data (©lyr).
    pub fn remove_lyrics(&mut self) {
        self.remove_data(atom::LYRICS);
    }

    /// Returns the title (©nam).
    pub fn title(&self) -> Option<&str> {
        self.get_string(atom::TITLE)
    }

    /// Sets the title (©nam).
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.set_data(atom::TITLE, Data::Utf8(Ok(title.into())));
    }

    /// Removes the title data (©nam).
    pub fn remove_title(&mut self) {
        self.remove_data(atom::TITLE);
    }

    /// Returns the tv episode number (tven).
    pub fn tv_episode_number(&self) -> Option<&str> {
        self.get_string(atom::TV_EPISODE_NUMBER)
    }

    /// Sets the tv episode number (tven).
    pub fn set_tv_episode_number(&mut self, tv_episode_number: impl Into<String>) {
        self.set_data(atom::TV_EPISODE_NUMBER, Data::Utf8(Ok(tv_episode_number.into())));
    }

    /// Removes the tv episode number data (tven).
    pub fn remove_tv_episode_number(&mut self) {
        self.remove_data(atom::TV_EPISODE_NUMBER);
    }

    /// Returns the tv network name (tvnn).
    pub fn tv_network_name(&self) -> Option<&str> {
        self.get_string(atom::TV_NETWORK_NAME)
    }

    /// Sets the tv network name (tvnn).
    pub fn set_tv_network_name(&mut self, tv_network_name: impl Into<String>) {
        self.set_data(atom::TV_NETWORK_NAME, Data::Utf8(Ok(tv_network_name.into())));
    }

    /// Removes the tv network name data (tvnn).
    pub fn remove_tv_network_name(&mut self) {
        self.remove_data(atom::TV_NETWORK_NAME);
    }

    /// Returns the tv show name (tvsh).
    pub fn tv_show_name(&self) -> Option<&str> {
        self.get_string(atom::TV_SHOW_NAME)
    }

    /// Sets the tv show name (tvsh).
    pub fn set_tv_show_name(&mut self, tv_show_name: impl Into<String>) {
        self.set_data(atom::TV_SHOW_NAME, Data::Utf8(Ok(tv_show_name.into())));
    }

    /// Removes the tv show name data (tvsh).
    pub fn remove_tv_show_name(&mut self) {
        self.remove_data(atom::TV_SHOW_NAME);
    }

    /// Returns the year (©day).
    pub fn year(&self) -> Option<&str> {
        self.get_string(atom::YEAR)
    }

    /// Sets the year (©day).
    pub fn set_year(&mut self, year: impl Into<String>) {
        self.set_data(atom::YEAR, Data::Utf8(Ok(year.into())));
    }

    /// Removes the year data (©day).
    pub fn remove_year(&mut self) {
        self.remove_data(atom::YEAR);
    }

    /// Returns the genre (©gen) or (gnre).
    pub fn genre(&self) -> Option<&str> {
        if let Some(s) = self.get_string(atom::CUSTOM_GENRE) {
            return Some(s);
        }

        if let Some(v) = self.get_reserved(atom::STANDARD_GENRE) {
            let mut chunks = v.chunks(2);

            if let Ok(genre_code) = chunks.next()?.read_u16::<BigEndian>() {
                for g in GENRES.iter() {
                    if g.0 == genre_code {
                        return Some(g.1);
                    }
                }
            }
        }

        None
    }

    /// Sets the standard genre (©gen) if it matches one otherwise a custom genre (gnre).
    pub fn set_genre(&mut self, genre: impl Into<String>) {
        let gen = genre.into();

        for g in GENRES.iter() {
            if g.1 == gen {
                self.remove_data(atom::CUSTOM_GENRE);
                self.set_data(atom::STANDARD_GENRE, Data::Reserved(Ok(vec![0u8, g.0 as u8])));
                return;
            }
        }

        self.remove_data(atom::STANDARD_GENRE);
        self.set_data(atom::CUSTOM_GENRE, Data::Utf8(Ok(gen)));
    }

    /// Removes the genre (©gen) or (gnre).
    pub fn remove_genre(&mut self) {
        self.remove_data(atom::STANDARD_GENRE);
        self.remove_data(atom::CUSTOM_GENRE);
    }

    /// Returns the track number and the total number of tracks (trkn).
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

    /// Sets the track number and the total number of tracks (trkn).
    pub fn set_track_number(&mut self, track_number: u16, total_tracks: u16) {
        let vec32 = vec![0u16, track_number, total_tracks, 0u16];
        let mut vec = Vec::new();

        for i in vec32 {
            vec.write_u16::<BigEndian>(i);
        }

        self.set_data(atom::TRACK_NUMBER, Data::Reserved(Ok(vec)));
    }

    /// Removes the track number and the total number of tracks (trkn).
    pub fn remove_track_number(&mut self) {
        self.remove_data(atom::TRACK_NUMBER);
    }

    /// Returns the disk number and total number of disks (disk).
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

    /// Sets the disk number and the total number of disks (disk).
    pub fn set_disk_number(&mut self, disk_number: u16, total_disks: u16) {
        let vec32 = vec![0u16, disk_number, total_disks];
        let mut vec = Vec::new();

        for i in vec32 {
            vec.write_u16::<BigEndian>(i);
        }

        self.set_data(atom::DISK_NUMBER, Data::Reserved(Ok(vec)));
    }

    /// Removes the disk number and the total number of disks (disk).
    pub fn remove_disk_number(&mut self) {
        self.remove_data(atom::DISK_NUMBER);
    }

    /// Returns the artwork image data of type `Data::JPEG` or `Data::PNG` (covr).
    pub fn artwork(&self) -> Option<Data> {
        self.get_image(atom::ARTWORK)
    }

    /// Sets the artwork image data of type `Data::JPEG` or `Data::PNG` (covr).
    pub fn set_artwork(&mut self, image: Data) {
        match &image {
            Data::Jpeg(_) => (),
            Data::Png(_) => (),
            _ => return,
        }

        self.set_data(atom::ARTWORK, image);
    }

    /// Removes the artwork image data (covr).
    pub fn remove_artwork(&mut self) {
        self.remove_data(atom::ARTWORK);
    }

    /// Attempts to return byte data corresponding to the provided head.
    pub fn get_reserved(&self, head: [u8; 4]) -> Option<&Vec<u8>> {
        match self.get_data(head) {
            Some(Data::Reserved(Ok(v))) => Some(v),
            _ => None,
        }
    }

    /// Attempts to return a string reference corresponding to the provided head.
    pub fn get_string(&self, head: [u8; 4]) -> Option<&str> {
        let d = self.get_data(head)?;

        match d {
            Data::Utf8(Ok(s)) => Some(s),
            Data::Utf16(Ok(s)) => Some(s),
            _ => None,
        }
    }

    /// Attempts to return a mutable string reference corresponding to the provided head.
    pub fn get_mut_string(&mut self, head: [u8; 4]) -> Option<&mut String> {
        let d = self.get_mut_data(head)?;

        match d {
            Data::Utf8(Ok(s)) => Some(s),
            Data::Utf16(Ok(s)) => Some(s),
            _ => None,
        }
    }

    /// Attempts to return image data of type `Data::JPEG` or `Data::PNG` corresponding to the provided head.
    pub fn get_image(&self, head: [u8; 4]) -> Option<Data> {
        let d = self.get_data(head)?;

        match d {
            Data::Jpeg(Ok(d)) => Some(Data::Jpeg(Ok(d.to_vec()))),
            Data::Png(Ok(d)) => Some(Data::Png(Ok(d.to_vec()))),
            _ => None,
        }
    }

    /// Attempts to return a data reference corresponding to the provided head.
    pub fn get_data(&self, head: [u8; 4]) -> Option<&Data> {
        if let Some(v) = self.get_atoms() {
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

    /// Attempts to return a mutable data reference corresponding to the provided head.
    pub fn get_mut_data(&mut self, head: [u8; 4]) -> Option<&mut Data> {
        if let Some(v) = self.get_mut_atoms() {
            for a in v {
                if a.head == head {
                    if let Content::TypedData(data) = &mut a.mut_first_child()?.content {
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    /// Updates or appends a new atom with the data corresponding to the head.
    ///
    /// # Example
    /// ```
    /// use mp4meta::{Tag, Data};
    ///
    /// let mut tag = Tag::new();
    /// tag.set_data(*b"test", Data::Utf8(Ok(String::from("data"))));
    /// assert_eq!(tag.get_string(*b"test").unwrap(), "data");
    /// ```
    pub fn set_data(&mut self, head: [u8; 4], data: Data) {
        if let Some(v) = self.get_mut_atoms() {
            for i in 0..v.len() {
                if v[i].head == head {
                    if let Some(p) = v[i].mut_first_child() {
                        if let Content::TypedData(d) = &mut p.content {
                            *d = data;
                            return;
                        }
                    }
                }
            }

            v.push(Atom::with(head, 0, Content::data_atom_with(data)));
        }
    }

    /// Removes the data corresponding to the head.
    ///
    /// # Example
    /// ```
    /// use mp4meta::{Tag, Data};
    ///
    /// let mut tag = Tag::new();
    /// tag.set_data(*b"test", Data::Utf8(Ok(String::from("data"))));
    /// tag.remove_data(*b"test");
    /// assert!(tag.get_data(*b"test").is_none())
    /// ```
    pub fn remove_data(&mut self, head: [u8; 4]) {
        if let Some(v) = self.get_mut_atoms() {
            for i in 0..v.len() {
                if v[i].head == head {
                    v.remove(i);
                    return;
                }
            }
        }
    }

    /// Returns a reference to the metadata atoms.
    pub fn get_atoms(&self) -> Option<&Vec<Atom>> {
        match &self.atom.first_child()?.first_child()?.first_child()?.content {
            Content::Atoms(v) => Some(v),
            _ => None
        }
    }

    /// Returns a mutable reference to the metadata atoms.
    pub fn get_mut_atoms(&mut self) -> Option<&mut Vec<Atom>> {
        match &mut self.atom.mut_first_child()?.mut_first_child()?.mut_first_child()?.content {
            Content::Atoms(v) => Some(v),
            _ => None
        }
    }
}

#[test]
fn test() {
    let mut tag = Tag::read_from_path("/mnt/data/Music/SOiL - Redefine/4 - SOiL - Cross My Heart.m4a");

    match &mut tag {
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