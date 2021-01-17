use crate::{atom, Data, Tag};

/// A list of standard genre codes and values found in the `gnre` atom. This list is equal to the
/// ID3v1 genre list but all codes are incremented by 1.
pub const STANDARD_GENRES: [(u16, &str); 80] = [
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

/// ### Standard genre
impl Tag {
    /// Returns all standard genres (`gnre`).
    pub fn standard_genres(&self) -> impl Iterator<Item = u16> + '_ {
        self.bytes(&atom::STANDARD_GENRE).filter_map(|v| {
            if v.len() < 2 {
                None
            } else {
                Some(u16::from_be_bytes([v[0], v[1]]))
            }
        })
    }

    /// Returns the first standard genre (`gnre`).
    pub fn standard_genre(&self) -> Option<u16> {
        self.standard_genres().next()
    }

    /// Sets the standard genre (`gnre`). This will remove all other standard genres.
    pub fn set_standard_genre(&mut self, genre_code: u16) {
        if genre_code > 0 && genre_code <= 80 {
            let vec: Vec<u8> = genre_code.to_be_bytes().to_vec();
            self.set_data(atom::STANDARD_GENRE, Data::Reserved(vec));
        }
    }

    /// Adds a standard genre (`gnre`).
    pub fn add_standard_genre(&mut self, genre_code: u16) {
        if genre_code > 0 && genre_code <= 80 {
            let vec: Vec<u8> = genre_code.to_be_bytes().to_vec();
            self.add_data(atom::STANDARD_GENRE, Data::Reserved(vec))
        }
    }

    /// Removes all standard genres (`gnre`).
    pub fn remove_standard_genres(&mut self) {
        self.remove_data(&atom::STANDARD_GENRE);
    }
}

/// ### Genre
///
/// These are convenience functions that combine the values from the standard genre (`gnre`) and
/// custom genre (`©gen`).
impl Tag {
    /// Returns all genres (`gnre` or `©gen`).
    pub fn genres(&self) -> impl Iterator<Item = &str> {
        self.standard_genres()
            .filter_map(|genre_code| {
                for g in STANDARD_GENRES.iter() {
                    if g.0 == genre_code {
                        return Some(g.1);
                    }
                }
                None
            })
            .chain(self.custom_genres())
    }

    /// Returns the first genre (`gnre` or `©gen`).
    pub fn genre(&self) -> Option<&str> {
        if let Some(genre_code) = self.standard_genre() {
            for g in STANDARD_GENRES.iter() {
                if g.0 == genre_code {
                    return Some(g.1);
                }
            }
        }

        self.custom_genre()
    }

    /// Consumes all custom genres (`©gen`) and returns all genres (`gnre` or `©gen`).
    pub fn take_genres(&mut self) -> impl Iterator<Item = String> + '_ {
        self.standard_genres()
            .filter_map(|genre_code| {
                for g in STANDARD_GENRES.iter() {
                    if g.0 == genre_code {
                        return Some(g.1.to_owned());
                    }
                }
                None
            })
            .collect::<Vec<String>>()
            .into_iter()
            .chain(self.take_custom_genres())
    }

    /// Consumes all custom genres (`©gen`) and returns the first genre (`gnre` or `©gen`).
    pub fn take_genre(&mut self) -> Option<String> {
        if let Some(genre_code) = self.standard_genre() {
            for g in STANDARD_GENRES.iter() {
                if g.0 == genre_code {
                    return Some(g.1.to_owned());
                }
            }
        }

        self.take_custom_genre()
    }

    /// Sets the standard genre (`gnre`) if it matches a predefined value otherwise a custom genre
    /// (`©gen`). This will remove all other standard or custom genres.
    pub fn set_genre(&mut self, genre: impl Into<String>) {
        let gen = genre.into();

        for g in STANDARD_GENRES.iter() {
            if g.1 == gen {
                self.remove_custom_genres();
                self.set_standard_genre(g.0);
                return;
            }
        }

        self.remove_standard_genres();
        self.set_custom_genre(gen)
    }

    /// Adds the standard genre (`gnre`) if it matches one otherwise a custom genre (`©gen`).
    pub fn add_genre(&mut self, genre: impl Into<String>) {
        let gen = genre.into();

        for g in STANDARD_GENRES.iter() {
            if g.1 == gen {
                self.add_standard_genre(g.0);
                return;
            }
        }

        self.add_custom_genre(gen)
    }

    /// Removes the genre (`gnre` or `©gen`).
    pub fn remove_genres(&mut self) {
        self.remove_standard_genres();
        self.remove_custom_genres();
    }

    /// Returns all genres formatted in an easily readable way.
    pub(crate) fn format_genres(&self) -> Option<String> {
        if self.genres().count() > 1 {
            let mut string = String::from("genres:\n");
            for v in self.genres() {
                string.push_str("    ");
                string.push_str(v);
                string.push('n');
            }
            return Some(string);
        }

        match self.genre() {
            Some(s) => Some(format!("genre: {}\n", s)),
            None => None,
        }
    }
}
