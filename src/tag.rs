use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;

use byteorder::{BigEndian, WriteBytesExt};

use crate::{AdvisoryRating, atom, Atom, Content, Data, Ident, MediaType};

/// A list of standard genre codes and values found in the `gnre` atom.
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
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tag {
    /// A vector containing metadata atoms
    pub atoms: Vec<Atom>,
    /// A vector containing readonly metadata atoms
    pub readonly_atoms: Vec<Atom>,
}

impl Tag {
    /// Creates a new MPEG-4 audio tag containing the atom.
    pub fn with(atoms: Vec<Atom>, readonly_atoms: Vec<Atom>) -> Tag {
        Tag { atoms, readonly_atoms }
    }

    /// Attempts to read a MPEG-4 audio tag from the reader.
    pub fn read_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
        atom::read_tag_from(reader)
    }

    /// Attempts to read a MPEG-4 audio tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
        let mut file = BufReader::new(File::open(path)?);
        Tag::read_from(&mut file)
    }

    /// Attempts to write the MPEG-4 audio tag to the writer. This will overwrite any metadata
    /// previously present on the file.
    pub fn write_to(&self, file: &File) -> crate::Result<()> {
        atom::write_tag_to(file, &self.atoms)
    }

    /// Attempts to write the MPEG-4 audio tag to the path. This will overwrite any metadata
    /// previously present on the file.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        self.write_to(&file)
    }

    /// Attempts to dump the MPEG-4 audio tag to the writer.
    pub fn dump_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        let ftyp = Atom::with(atom::FILE_TYPE, 0, Content::RawData(
            Data::Utf8("M4A \u{0}\u{0}\u{2}\u{0}isomiso2".into())
        ));
        let moov = Atom::with(atom::MOVIE, 0, Content::atoms()
            .add_atom_with(atom::USER_DATA, 0, Content::atoms()
                .add_atom_with(atom::METADATA, 4, Content::atoms()
                    .add_atom_with(atom::ITEM_LIST, 0, Content::Atoms(self.atoms.clone())),
                ),
            ),
        );

        ftyp.write_to(writer)?;
        moov.write_to(writer)?;

        Ok(())
    }

    /// Attempts to dump the MPEG-4 audio tag to the writer.
    pub fn dump_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let mut file = File::create(path)?;
        self.dump_to(&mut file)
    }
}

/// ## Individual string values
impl Tag {
    /// Returns the album (©alb).
    pub fn album(&self) -> Option<&str> {
        self.string(atom::ALBUM).next()
    }

    /// Sets the album (©alb).
    pub fn set_album(&mut self, album: impl Into<String>) {
        self.set_data(atom::ALBUM, Data::Utf8(album.into()));
    }

    /// Removes the album (©alb).
    pub fn remove_album(&mut self) {
        self.remove_data(atom::ALBUM);
    }


    /// Returns the copyright (cprt).
    pub fn copyright(&self) -> Option<&str> {
        self.string(atom::COPYRIGHT).next()
    }

    /// Sets the copyright (cprt).
    pub fn set_copyright(&mut self, copyright: impl Into<String>) {
        self.set_data(atom::COPYRIGHT, Data::Utf8(copyright.into()));
    }

    /// Removes the copyright (cprt).
    pub fn remove_copyright(&mut self) {
        self.remove_data(atom::COPYRIGHT);
    }


    /// Returns the encoder (©too).
    pub fn encoder(&self) -> Option<&str> {
        self.string(atom::ENCODER).next()
    }

    /// Sets the encoder (©too).
    pub fn set_encoder(&mut self, encoder: impl Into<String>) {
        self.set_data(atom::ENCODER, Data::Utf8(encoder.into()));
    }

    /// Removes the encoder (©too).
    pub fn remove_encoder(&mut self) {
        self.remove_data(atom::ENCODER);
    }


    /// Returns the title (©nam).
    pub fn title(&self) -> Option<&str> {
        self.string(atom::TITLE).next()
    }

    /// Sets the title (©nam).
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.set_data(atom::TITLE, Data::Utf8(title.into()));
    }

    /// Removes the title (©nam).
    pub fn remove_title(&mut self) {
        self.remove_data(atom::TITLE);
    }


    /// Returns the lyrics (©lyr).
    pub fn lyrics(&self) -> Option<&str> {
        self.string(atom::LYRICS).next()
    }

    /// Sets the lyrics (©lyr).
    pub fn set_lyrics(&mut self, lyrics: impl Into<String>) {
        self.set_data(atom::LYRICS, Data::Utf8(lyrics.into()));
    }

    /// Removes the lyrics (©lyr).
    pub fn remove_lyrics(&mut self) {
        self.remove_data(atom::LYRICS);
    }


    /// Returns the movement (©mvn).
    pub fn movement(&self) -> Option<&str> {
        self.string(atom::MOVEMENT_NAME).next()
    }

    /// Sets the movement (©mvn).
    pub fn set_movement(&mut self, movement: impl Into<String>) {
        self.set_data(atom::MOVEMENT_NAME, Data::Utf8(movement.into()));
    }

    /// Removes the movement (©mvn).
    pub fn remove_movement(&mut self) {
        self.remove_data(atom::MOVEMENT_NAME)
    }


    /// Returns the tv episode number (tven).
    pub fn tv_episode_number(&self) -> Option<&str> {
        self.string(atom::TV_EPISODE_NUMBER).next()
    }

    /// Sets the tv episode number (tven).
    pub fn set_tv_episode_number(&mut self, tv_episode_number: impl Into<String>) {
        self.set_data(atom::TV_EPISODE_NUMBER, Data::Utf8(tv_episode_number.into()));
    }

    /// Removes the tv episode number (tven).
    pub fn remove_tv_episode_number(&mut self) {
        self.remove_data(atom::TV_EPISODE_NUMBER);
    }


    /// Returns the tv network name (tvnn).
    pub fn tv_network_name(&self) -> Option<&str> {
        self.string(atom::TV_NETWORK_NAME).next()
    }

    /// Sets the tv network name (tvnn).
    pub fn set_tv_network_name(&mut self, tv_network_name: impl Into<String>) {
        self.set_data(atom::TV_NETWORK_NAME, Data::Utf8(tv_network_name.into()));
    }

    /// Removes the tv network name (tvnn).
    pub fn remove_tv_network_name(&mut self) {
        self.remove_data(atom::TV_NETWORK_NAME);
    }


    /// Returns the tv show name (tvsh).
    pub fn tv_show_name(&self) -> Option<&str> {
        self.string(atom::TV_SHOW_NAME).next()
    }

    /// Sets the tv show name (tvsh).
    pub fn set_tv_show_name(&mut self, tv_show_name: impl Into<String>) {
        self.set_data(atom::TV_SHOW_NAME, Data::Utf8(tv_show_name.into()));
    }

    /// Removes the tv show name (tvsh).
    pub fn remove_tv_show_name(&mut self) {
        self.remove_data(atom::TV_SHOW_NAME);
    }


    /// Returns the year (©day).
    pub fn year(&self) -> Option<&str> {
        self.string(atom::YEAR).next()
    }

    /// Sets the year (©day).
    pub fn set_year(&mut self, year: impl Into<String>) {
        self.set_data(atom::YEAR, Data::Utf8(year.into()));
    }

    /// Removes the year (©day).
    pub fn remove_year(&mut self) {
        self.remove_data(atom::YEAR);
    }


    /// Returns the work (©wrk).
    pub fn work(&self) -> Option<&str> {
        self.string(atom::WORK).next()
    }

    /// Removes the work (©wrk).
    pub fn remove_work(&mut self) {
        self.remove_data(atom::WORK)
    }

    /// Sets the work (©wrk).
    pub fn set_work(&mut self, work: impl Into<String>) {
        self.set_data(atom::WORK, Data::Utf8(work.into()));
    }
}

/// ## Multiple string values
impl Tag {
    /// Returns all album artists (aART).
    pub fn album_artists(&self) -> impl Iterator<Item=&str> {
        self.string(atom::ALBUM_ARTIST)
    }

    /// Returns the first album artist (aART).
    pub fn album_artist(&self) -> Option<&str> {
        self.album_artists().next()
    }

    /// Sets the album artist (aART). This will remove all other album artists.
    pub fn set_album_artist(&mut self, album_artist: impl Into<String>) {
        self.set_data(atom::ALBUM_ARTIST, Data::Utf8(album_artist.into()));
    }

    /// Adds an album artist (aART).
    pub fn add_album_artist(&mut self, album_artist: impl Into<String>) {
        self.add_data(atom::ALBUM_ARTIST, Data::Utf8(album_artist.into()));
    }

    /// Removes all album artists (aART).
    pub fn remove_album_artists(&mut self) {
        self.remove_data(atom::ALBUM_ARTIST);
    }


    /// Returns all artists (©ART).
    pub fn artists(&self) -> impl Iterator<Item=&str> {
        self.string(atom::ARTIST)
    }

    /// Returns the first artist (©ART).
    pub fn artist(&self) -> Option<&str> {
        self.artists().next()
    }

    /// Sets the artist (©ART). This will remove all other artists.
    pub fn set_artist(&mut self, artist: impl Into<String>) {
        self.set_data(atom::ARTIST, Data::Utf8(artist.into()));
    }

    /// Adds an artist (©ART).
    pub fn add_artist(&mut self, artist: impl Into<String>) {
        self.add_data(atom::ARTIST, Data::Utf8(artist.into()));
    }

    /// Removes all artists (©ART).
    pub fn remove_artists(&mut self) {
        self.remove_data(atom::ARTIST);
    }


    /// Returns all categories (catg).
    pub fn categories(&self) -> impl Iterator<Item=&str> {
        self.string(atom::CATEGORY)
    }

    /// Returns the first category (catg).
    pub fn category(&self) -> Option<&str> {
        self.categories().next()
    }

    /// Sets the category (catg). This will remove all other categories.
    pub fn set_category(&mut self, category: impl Into<String>) {
        self.set_data(atom::CATEGORY, Data::Utf8(category.into()));
    }

    /// Adds a category (catg).
    pub fn add_category(&mut self, category: impl Into<String>) {
        self.add_data(atom::CATEGORY, Data::Utf8(category.into()));
    }

    /// Removes all categories (catg).
    pub fn remove_categories(&mut self) {
        self.remove_data(atom::CATEGORY);
    }


    /// Returns all comments (©cmt).
    pub fn comments(&self) -> impl Iterator<Item=&str> {
        self.string(atom::COMMENT)
    }

    /// Returns the first comment (©cmt).
    pub fn comment(&self) -> Option<&str> {
        self.comments().next()
    }

    /// Sets the comment (©cmt). This will remove all other comments.
    pub fn set_comment(&mut self, comment: impl Into<String>) {
        self.set_data(atom::COMMENT, Data::Utf8(comment.into()));
    }

    /// Adds a comment (©cmt).
    pub fn add_comment(&mut self, comment: impl Into<String>) {
        self.add_data(atom::COMMENT, Data::Utf8(comment.into()));
    }

    /// Removes all comments (©cmt).
    pub fn remove_comments(&mut self) {
        self.remove_data(atom::COMMENT);
    }


    /// Returns all composers (©wrt).
    pub fn composers(&self) -> impl Iterator<Item=&str> {
        self.string(atom::COMPOSER)
    }

    /// Returns the first composer (©wrt).
    pub fn composer(&self) -> Option<&str> {
        self.composers().next()
    }

    /// Sets the composer (©wrt). This will remove all other composers.
    pub fn set_composer(&mut self, composer: impl Into<String>) {
        self.set_data(atom::COMPOSER, Data::Utf8(composer.into()));
    }

    /// Adds a composer (©wrt).
    pub fn add_composer(&mut self, composer: impl Into<String>) {
        self.add_data(atom::COMPOSER, Data::Utf8(composer.into()));
    }

    /// Removes the composer (©wrt).
    pub fn remove_composers(&mut self) {
        self.remove_data(atom::COMMENT);
    }


    /// Returns all descriptions (desc).
    pub fn descriptions(&self) -> impl Iterator<Item=&str> {
        self.string(atom::DESCRIPTION)
    }

    /// Returns the first description (desc).
    pub fn description(&self) -> Option<&str> {
        self.descriptions().next()
    }

    /// Sets the description (desc). This will remove all other descriptions.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.set_data(atom::DESCRIPTION, Data::Utf8(description.into()));
    }

    /// Adds a description (desc).
    pub fn add_description(&mut self, description: impl Into<String>) {
        self.add_data(atom::DESCRIPTION, Data::Utf8(description.into()));
    }

    /// Removes the description (desc).
    pub fn remove_descriptions(&mut self) {
        self.remove_data(atom::DESCRIPTION);
    }


    /// Returns all groupings (©grp).
    pub fn groupings(&self) -> impl Iterator<Item=&str> {
        self.string(atom::GROUPING)
    }

    /// Returns the first grouping (©grp).
    pub fn grouping(&self) -> Option<&str> {
        self.groupings().next()
    }

    /// Sets the grouping (©grp). This will remove all other groupings.
    pub fn set_grouping(&mut self, grouping: impl Into<String>) {
        self.set_data(atom::GROUPING, Data::Utf8(grouping.into()));
    }

    /// Adds a grouping (©grp).
    pub fn add_grouping(&mut self, grouping: impl Into<String>) {
        self.add_data(atom::GROUPING, Data::Utf8(grouping.into()));
    }

    /// Removes the grouping (©grp).
    pub fn remove_groupings(&mut self) {
        self.remove_data(atom::GROUPING);
    }


    /// Returns all keywords (keyw).
    pub fn keywords(&self) -> impl Iterator<Item=&str> {
        self.string(atom::KEYWORD)
    }

    /// Returns the first keyword (keyw).
    pub fn keyword(&self) -> Option<&str> {
        self.keywords().next()
    }

    /// Sets the keyword (keyw). This will remove all other keywords.
    pub fn set_keyword(&mut self, keyword: impl Into<String>) {
        self.set_data(atom::KEYWORD, Data::Utf8(keyword.into()));
    }

    /// Adds a keyword (keyw).
    pub fn add_keyword(&mut self, keyword: impl Into<String>) {
        self.set_data(atom::KEYWORD, Data::Utf8(keyword.into()));
    }

    /// Removes the keyword (keyw).
    pub fn remove_keywords(&mut self) {
        self.remove_data(atom::KEYWORD);
    }
}

/// ## Flags
impl Tag {
    /// Returns the compilation flag (cpil).
    pub fn compilation(&self) -> bool {
        let vec = match self.data(atom::COMPILATION).next() {
            Some(Data::Reserved(v)) => v,
            Some(Data::BeSigned(v)) => v,
            _ => return false,
        };

        if vec.is_empty() {
            return false;
        }

        vec[0] != 0
    }

    /// Sets the compilation flag to true (cpil).
    pub fn set_compilation(&mut self) {
        self.set_data(atom::COMPILATION, Data::BeSigned(vec![1u8]));
    }

    /// Removes the compilation flag (cpil).
    pub fn remove_compilation(&mut self) {
        self.remove_data(atom::COMPILATION)
    }


    /// Returns the gapless playback flag (pgap).
    pub fn gapless_playback(&self) -> bool {
        let vec = match self.be_signed(atom::GAPLESS_PLAYBACK).next() {
            Some(v) => v,
            None => return false,
        };

        if vec.is_empty() {
            return false;
        }

        vec[0] != 0
    }

    /// Sets the gapless playback flag to true (pgap).
    pub fn set_gapless_playback(&mut self) {
        self.set_data(atom::GAPLESS_PLAYBACK, Data::BeSigned(vec![1u8]));
    }

    /// Removes the gapless playback flag (pgap).
    pub fn remove_gapless_playback(&mut self) {
        self.remove_data(atom::GAPLESS_PLAYBACK)
    }


    /// Returns the show movement flag (shwm).
    pub fn show_movement(&self) -> bool {
        let vec = match self.be_signed(atom::SHOW_MOVEMENT).next() {
            Some(v) => v,
            None => return false,
        };

        if vec.is_empty() {
            return false;
        }

        vec[0] != 0
    }

    /// Sets the show movement flag to true (shwm).
    pub fn set_show_movement(&mut self) {
        self.set_data(atom::SHOW_MOVEMENT, Data::BeSigned(vec![1u8]));
    }

    /// Removes the show movement flag (shwm).
    pub fn remove_show_movement(&mut self) {
        self.remove_data(atom::SHOW_MOVEMENT)
    }
}

/// ## Integer values
impl Tag {
    /// Returns the bpm (tmpo)
    pub fn bpm(&self) -> Option<u16> {
        let vec = match self.data(atom::BPM).next()? {
            Data::Reserved(v) => v,
            Data::BeSigned(v) => v,
            _ => return None,
        };

        if vec.len() < 2 {
            return None;
        }

        Some(u16::from_be_bytes([vec[0], vec[1]]))
    }

    /// Sets the bpm (tmpo)
    pub fn set_bpm(&mut self, bpm: u16) {
        let mut vec = Vec::with_capacity(2);
        vec.write_u16::<BigEndian>(bpm).unwrap();

        self.set_data(atom::BPM, Data::BeSigned(vec));
    }

    /// Removes the bpm (tmpo).
    pub fn remove_bpm(&mut self) {
        self.remove_data(atom::BPM);
    }


    /// Returns the movement count (©mvc).
    pub fn movement_count(&self) -> Option<u16> {
        let vec = self.be_signed(atom::MOVEMENT_COUNT).next()?;

        if vec.len() < 2 {
            return None;
        }

        Some(u16::from_be_bytes([vec[0], vec[1]]))
    }

    /// Sets the movement count (©mvc).
    pub fn set_movement_count(&mut self, count: u16) {
        let mut vec: Vec<u8> = Vec::with_capacity(2);
        vec.write_u16::<BigEndian>(count).unwrap();

        self.set_data(atom::MOVEMENT_COUNT, Data::BeSigned(vec));
    }

    /// Removes the movement count (©mvc).
    pub fn remove_movement_count(&mut self) {
        self.remove_data(atom::MOVEMENT_COUNT)
    }


    /// Returns the movement index (©mvi).
    pub fn movement_index(&self) -> Option<u16> {
        let vec = self.be_signed(atom::MOVEMENT_INDEX).next()?;

        if vec.len() < 2 {
            return None;
        }

        Some(u16::from_be_bytes([vec[0], vec[1]]))
    }

    /// Sets the movement index (©mvi).
    pub fn set_movement_index(&mut self, index: u16) {
        let mut vec: Vec<u8> = Vec::with_capacity(2);
        vec.write_u16::<BigEndian>(index).unwrap();

        self.set_data(atom::MOVEMENT_INDEX, Data::BeSigned(vec));
    }

    /// Removes the movement index (©mvi).
    pub fn remove_movement_index(&mut self) {
        self.remove_data(atom::MOVEMENT_INDEX)
    }
}

/// ## Tuple values
impl Tag {
    /// Returns the track number and the total number of tracks (trkn).
    pub fn track_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.reserved(atom::TRACK_NUMBER).next() {
            Some(v) => v,
            None => return (None, None),
        };

        let track_number = if vec.len() < 4 {
            None
        } else {
            Some(u16::from_be_bytes([vec[2], vec[3]]))
        };

        let total_tracks = if vec.len() < 6 {
            None
        } else {
            Some(u16::from_be_bytes([vec[4], vec[5]]))
        };

        (track_number, total_tracks)
    }

    /// Sets the track number and the total number of tracks (trkn).
    pub fn set_track_number(&mut self, track_number: u16, total_tracks: u16) {
        let vec16 = vec![0u16, track_number, total_tracks, 0u16];
        let mut vec = Vec::with_capacity(8);

        for i in vec16 {
            vec.write_u16::<BigEndian>(i).unwrap();
        }

        self.set_data(atom::TRACK_NUMBER, Data::Reserved(vec));
    }

    /// Removes the track number and the total number of tracks (trkn).
    pub fn remove_track_number(&mut self) {
        self.remove_data(atom::TRACK_NUMBER);
    }

    /// Returns the disc number and total number of discs (disk).
    pub fn disc_number(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.reserved(atom::DISC_NUMBER).next() {
            Some(v) => v,
            None => return (None, None),
        };

        let disc_number = if vec.len() < 4 {
            None
        } else {
            Some(u16::from_be_bytes([vec[2], vec[3]]))
        };

        let total_discs = if vec.len() < 6 {
            None
        } else {
            Some(u16::from_be_bytes([vec[4], vec[5]]))
        };

        (disc_number, total_discs)
    }

    /// Sets the disc number and the total number of discs (disk).
    pub fn set_disc_number(&mut self, disc_number: u16, total_discs: u16) {
        let vec16 = vec![0u16, disc_number, total_discs];
        let mut vec = Vec::with_capacity(6);

        for i in vec16 {
            vec.write_u16::<BigEndian>(i).unwrap();
        }

        self.set_data(atom::DISC_NUMBER, Data::Reserved(vec));
    }

    /// Removes the disc number and the total number of discs (disk).
    pub fn remove_disc_number(&mut self) {
        self.remove_data(atom::DISC_NUMBER);
    }
}

/// ## Genre
impl Tag {
    /// Returns all genres (gnre) or (©gen).
    pub fn genres(&self) -> impl Iterator<Item=&str> {
        self.standard_genres().filter_map(|genre_code| {
            for g in GENRES.iter() {
                if g.0 == genre_code {
                    return Some(g.1);
                }
            }
            None
        }).chain(
            self.custom_genres()
        )
    }

    /// Returns the first genre (gnre) or (©gen).
    pub fn genre(&self) -> Option<&str> {
        if let Some(genre_code) = self.standard_genre() {
            for g in GENRES.iter() {
                if g.0 == genre_code {
                    return Some(g.1);
                }
            }
        }

        self.custom_genre()
    }

    /// Sets the standard genre (gnre) if it matches one otherwise a custom genre (©gen).
    pub fn set_genre(&mut self, genre: impl Into<String>) {
        let gen = genre.into();


        for g in GENRES.iter() {
            if g.1 == gen {
                self.remove_custom_genres();
                self.set_standard_genre(g.0);
                return;
            }
        }

        self.remove_standard_genres();
        self.set_custom_genre(gen)
    }

    /// Adds the standard genre (gnre) if it matches one otherwise a custom genre (©gen).
    pub fn add_genre(&mut self, genre: impl Into<String>) {
        let gen = genre.into();

        for g in GENRES.iter() {
            if g.1 == gen {
                self.add_standard_genre(g.0);
                return;
            }
        }

        self.add_custom_genre(gen)
    }

    /// Removes the genre (gnre) or (©gen).
    pub fn remove_genres(&mut self) {
        self.remove_standard_genres();
        self.remove_custom_genres();
    }


    /// Returns all standard genres (gnre).
    pub fn standard_genres(&self) -> impl Iterator<Item=u16> + '_ {
        self.reserved(atom::STANDARD_GENRE)
            .filter_map(|v| {
                if v.len() < 2 {
                    None
                } else {
                    Some(u16::from_be_bytes([v[0], v[1]]))
                }
            })
    }

    /// Returns the first standard genre (gnre).
    pub fn standard_genre(&self) -> Option<u16> {
        self.standard_genres().next()
    }

    /// Sets the standard genre (gnre). This will remove all other standard genres.
    pub fn set_standard_genre(&mut self, genre_code: u16) {
        if genre_code > 0 && genre_code <= 80 {
            let mut vec: Vec<u8> = Vec::with_capacity(2);
            vec.write_u16::<BigEndian>(genre_code).unwrap();
            self.set_data(atom::STANDARD_GENRE, Data::Reserved(vec));
        }
    }

    /// Adds a standard genre (gnre).
    pub fn add_standard_genre(&mut self, genre_code: u16) {
        if genre_code > 0 && genre_code <= 80 {
            let mut vec: Vec<u8> = Vec::with_capacity(2);
            vec.write_u16::<BigEndian>(genre_code).unwrap();
            self.add_data(atom::STANDARD_GENRE, Data::Reserved(vec))
        }
    }

    /// Removes all standard genres (gnre).
    pub fn remove_standard_genres(&mut self) {
        self.remove_data(atom::STANDARD_GENRE);
    }


    /// Returns all custom genres (©gen).
    pub fn custom_genres(&self) -> impl Iterator<Item=&str> {
        self.string(atom::CUSTOM_GENRE)
    }

    /// Returns the first custom genre (©gen).
    pub fn custom_genre(&self) -> Option<&str> {
        self.string(atom::CUSTOM_GENRE).next()
    }

    /// Sets the custom genre (©gen). This will remove all other custom genres.
    pub fn set_custom_genre(&mut self, custom_genre: impl Into<String>) {
        self.set_data(atom::CUSTOM_GENRE, Data::Utf8(custom_genre.into()));
    }

    /// Adds a custom genre (©gen).
    pub fn add_custom_genre(&mut self, custom_genre: impl Into<String>) {
        self.add_data(atom::CUSTOM_GENRE, Data::Utf8(custom_genre.into()));
    }

    /// Removes the custom genre (©gen).
    pub fn remove_custom_genres(&mut self) {
        self.remove_data(atom::CUSTOM_GENRE);
    }
}

/// ## Custom values
impl Tag {
    /// Returns the artwork image data of type `Data::JPEG` or `Data::PNG` (covr).
    pub fn artwork(&self) -> impl Iterator<Item=&Data> {
        self.image(atom::ARTWORK)
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

    /// Sets the artwork image data of type `Data::JPEG` or `Data::PNG` (covr).
    pub fn add_artwork(&mut self, image: Data) {
        match &image {
            Data::Jpeg(_) => (),
            Data::Png(_) => (),
            _ => return,
        }

        self.add_data(atom::ARTWORK, image);
    }

    /// Removes the artwork image data (covr).
    pub fn remove_artwork(&mut self) {
        self.remove_data(atom::ARTWORK);
    }


    /// Returns the media type (stik).
    pub fn media_type(&self) -> Option<MediaType> {
        let vec = match self.data(atom::MEDIA_TYPE).next()? {
            Data::Reserved(v) => v,
            Data::BeSigned(v) => v,
            _ => return None,
        };

        if vec.is_empty() {
            return None;
        }

        MediaType::try_from(vec[0]).ok()
    }

    /// Sets the media type (stik).
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.set_data(atom::MEDIA_TYPE, Data::Reserved(vec![media_type.value()]));
    }

    /// Removes the media type (stik).
    pub fn remove_media_type(&mut self) {
        self.remove_data(atom::MEDIA_TYPE);
    }


    /// Returns the rating (rtng).
    pub fn advisory_rating(&self) -> Option<AdvisoryRating> {
        let vec = match self.data(atom::ADVISORY_RATING).next()? {
            Data::Reserved(v) => v,
            Data::BeSigned(v) => v,
            _ => return None,
        };

        if vec.is_empty() {
            return None;
        }

        Some(AdvisoryRating::from(vec[0]))
    }

    /// Sets the rating (rtng).
    pub fn set_advisory_rating(&mut self, rating: AdvisoryRating) {
        self.set_data(atom::ADVISORY_RATING, Data::Reserved(vec![rating.value()]));
    }

    /// Removes the rating (rtng).
    pub fn remove_advisory_rating(&mut self) {
        self.remove_data(atom::ADVISORY_RATING);
    }
}

/// ## Readonly values
impl Tag {
    /// Returns the duration in seconds.
    /// [Spec](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-SW34)
    pub fn duration(&self) -> Option<f64> {
        let mut vec = None;

        for a in &self.readonly_atoms {
            if a.ident == atom::MEDIA_HEADER {
                if let Content::RawData(Data::Reserved(v)) = &a.content {
                    vec = Some(v);
                    break;
                }
            }
        }

        let vec = vec?;

        if vec.len() < 24 {
            return None;
        }

        let buf: Vec<u32> = vec
            .chunks_exact(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        let timescale_unit = buf[3];
        let duration_units = buf[4];

        let duration = duration_units as f64 / timescale_unit as f64;

        Some(duration)
    }

    /// returns the filetype (ftyp).
    pub fn filetype(&self) -> Option<String> {
        for a in &self.readonly_atoms {
            if a.ident == atom::FILE_TYPE {
                if let Content::RawData(Data::Utf8(s)) = &a.content {
                    return Some(s.to_string());
                }
            }
        }

        None
    }
}

/// ## Accessors
impl Tag {
    /// Returns all byte data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Reserved(vec![1,2,3,4,5,6]));
    /// assert_eq!(tag.reserved(Ident(*b"test")).next().unwrap().to_vec(), vec![1,2,3,4,5,6]);
    /// ```
    pub fn reserved(&self, ident: Ident) -> impl Iterator<Item=&Vec<u8>> {
        self.data(ident).filter_map(|d| {
            match d {
                Data::Reserved(v) => Some(v),
                _ => None,
            }
        })
    }

    /// Returns all byte data representing a big endian integer corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::BeSigned(vec![1,2,3,4,5,6]));
    /// assert_eq!(tag.be_signed(Ident(*b"test")).next().unwrap().to_vec(), vec![1,2,3,4,5,6]);
    /// ```
    pub fn be_signed(&self, ident: Ident) -> impl Iterator<Item=&Vec<u8>> {
        self.data(ident).filter_map(|d| {
            match d {
                Data::BeSigned(v) => Some(v),
                _ => None,
            }
        })
    }

    /// Returns all string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data");
    /// ```
    pub fn string(&self, ident: Ident) -> impl Iterator<Item=&str> {
        self.data(ident).filter_map(|d| {
            match d {
                Data::Utf8(s) => Some(&**s),
                Data::Utf16(s) => Some(&**s),
                _ => None,
            }
        })
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// tag.mut_string(Ident(*b"test")).next().unwrap().push('1');
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data1");
    /// ```
    pub fn mut_string(&mut self, ident: Ident) -> impl Iterator<Item=&mut String> {
        self.mut_data(ident).filter_map(|d| {
            match d {
                Data::Utf8(s) => Some(s),
                Data::Utf16(s) => Some(s),
                _ => None,
            }
        })
    }

    /// Returns all image data of type `Data::JPEG` or `Data::PNG` corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Jpeg("<the image data>".as_bytes().to_vec()));
    /// match tag.image(Ident(*b"test")).next().unwrap() {
    ///     Data::Jpeg(v) => assert_eq!(*v, "<the image data>".as_bytes()),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn image(&self, ident: Ident) -> impl Iterator<Item=&Data> {
        self.data(ident).filter(|d| {
            match d {
                Data::Jpeg(_) => true,
                Data::Png(_) => true,
                _ => false,
            }
        })
    }

    /// Returns all data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// match tag.data(Ident(*b"test")).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn data(&self, ident: Ident) -> impl Iterator<Item=&Data> {
        self.atoms.iter().filter_map(|a| {
            if a.ident == ident {
                if let Content::TypedData(d) = &a.first_child()?.content {
                    return Some(d);
                }
            }
            None
        }).collect::<Vec<&Data>>().into_iter()
    }

    /// Returns all mutable data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// if let Data::Utf8(s) = tag.mut_data(Ident(*b"test")).next().unwrap() {
    ///     s.push('1');
    /// }
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data1");
    /// ```
    pub fn mut_data(&mut self, ident: Ident) -> impl Iterator<Item=&mut Data> {
        self.atoms.iter_mut().filter_map(|a| {
            if a.ident == ident {
                if let Content::TypedData(d) = &mut a.mut_first_child()?.content {
                    return Some(d);
                }
            }
            None
        }).collect::<Vec<&mut Data>>().into_iter()
    }

    /// Removes all other atoms, corresponding to the identifier, and adds a new atom containing the
    /// provided data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data");
    /// ```
    pub fn set_data(&mut self, ident: Ident, data: Data) {
        self.remove_data(ident);
        self.atoms.push(Atom::with(ident, 0, Content::data_atom_with(data)));
    }

    /// Adds a new atom, corresponding to the identifier, containing the provided data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.add_data(Ident(*b"test"), Data::Utf8("data1".into()));
    /// tag.add_data(Ident(*b"test"), Data::Utf8("data2".into()));
    /// let mut strings = tag.string(Ident(*b"test"));
    /// assert_eq!(strings.next().unwrap(), "data1");
    /// assert_eq!(strings.next().unwrap(), "data2");
    /// ```
    pub fn add_data(&mut self, ident: Ident, data: Data) {
        self.atoms.push(Atom::with(ident, 0, Content::data_atom_with(data)));
    }

    /// Removes the data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// assert!(tag.data(Ident(*b"test")).next().is_some());
    /// tag.remove_data(Ident(*b"test"));
    /// assert!(tag.data(Ident(*b"test")).next().is_none());
    /// ```
    pub fn remove_data(&mut self, ident: Ident) {
        let mut i = 0;
        while i < self.atoms.len() {
            if self.atoms[i].ident == ident {
                self.atoms.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
