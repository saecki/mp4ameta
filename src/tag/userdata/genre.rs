use std::fmt;

use crate::{ident, Data, Userdata};

/// A list of standard genre codes and values found in the `gnre` atom. The codes are equivalent to
/// the ID3v1 genre codes plus 1.
pub const STANDARD_GENRES: [&str; 80] = [
    "Blues",
    "Classic rock",
    "Country",
    "Dance",
    "Disco",
    "Funk",
    "Grunge",
    "Hip,-Hop",
    "Jazz",
    "Metal",
    "New Age",
    "Oldies",
    "Other",
    "Pop",
    "Rhythm and Blues",
    "Rap",
    "Reggae",
    "Rock",
    "Techno",
    "Industrial",
    "Alternative",
    "Ska",
    "Death metal",
    "Pranks",
    "Soundtrack",
    "Euro-Techno",
    "Ambient",
    "Trip-Hop",
    "Vocal",
    "Jazz & Funk",
    "Fusion",
    "Trance",
    "Classical",
    "Instrumental",
    "Acid",
    "House",
    "Game",
    "Sound clip",
    "Gospel",
    "Noise",
    "Alternative Rock",
    "Bass",
    "Soul",
    "Punk",
    "Space",
    "Meditative",
    "Instrumental Pop",
    "Instrumental Rock",
    "Ethnic",
    "Gothic",
    "Darkwave",
    "Techno-Industrial",
    "Electronic",
    "Pop-Folk",
    "Eurodance",
    "Dream",
    "Southern Rock",
    "Comedy",
    "Cult",
    "Gangsta",
    "Top 41",
    "Christian Rap",
    "Pop/Funk",
    "Jungle",
    "Native US",
    "Cabaret",
    "New Wave",
    "Psychedelic",
    "Rave",
    "Show tunes",
    "Trailer",
    "Lo,-Fi",
    "Tribal",
    "Acid Punk",
    "Acid Jazz",
    "Polka",
    "Retro",
    "Musical",
    "Rock ’n’ Roll",
    "Hard Rock",
];

/// ### Standard genre
impl Userdata {
    /// Returns all standard genres (`gnre`).
    pub fn standard_genres(&self) -> impl Iterator<Item = u16> + '_ {
        self.bytes_of(&ident::STANDARD_GENRE).filter_map(|v| {
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
        let vec: Vec<u8> = genre_code.to_be_bytes().to_vec();
        self.set_data(ident::STANDARD_GENRE, Data::Reserved(vec));
    }

    /// Sets all standard genres (`gnre`). This will remove all other standard genres.
    pub fn set_standard_genres(&mut self, genre_codes: impl IntoIterator<Item = u16>) {
        let data = genre_codes.into_iter().map(|c| Data::Reserved(c.to_be_bytes().to_vec()));
        self.set_all_data(ident::STANDARD_GENRE, data);
    }

    /// Adds a standard genre (`gnre`).
    pub fn add_standard_genre(&mut self, genre_code: u16) {
        let vec: Vec<u8> = genre_code.to_be_bytes().to_vec();
        self.add_data(ident::STANDARD_GENRE, Data::Reserved(vec))
    }

    /// Adds all standard genres (`gnre`).
    pub fn add_standard_genres(&mut self, genre_codes: impl IntoIterator<Item = u16>) {
        let data = genre_codes.into_iter().map(|c| Data::Reserved(c.to_be_bytes().to_vec()));
        self.add_all_data(ident::STANDARD_GENRE, data)
    }

    /// Removes all standard genres (`gnre`).
    pub fn remove_standard_genres(&mut self) {
        self.remove_data_of(&ident::STANDARD_GENRE);
    }
}

/// ### Genre
///
/// These are convenience methods that operate on values of both standard genres (`gnre`) and
/// custom genres (`©gen`).
impl Userdata {
    /// Returns all genres, first the standard genres (`gnre`) then custom ones (`©gen`).
    pub fn genres(&self) -> impl Iterator<Item = &str> + '_ {
        #[allow(clippy::redundant_closure)]
        self.standard_genres().filter_map(|c| stanard_genre(c)).chain(self.custom_genres())
    }

    /// Returns the first genre (`gnre` or `©gen`).
    pub fn genre(&self) -> Option<&str> {
        if let Some(g) = self.standard_genre().and_then(stanard_genre) {
            return Some(g);
        }

        self.custom_genre()
    }

    /// Removes all custom genres (`©gen`) and returns all genres, first standard genres (`gnre`)
    /// then custom ones (`©gen`).
    pub fn take_genres(&mut self) -> impl Iterator<Item = String> + '_ {
        self.standard_genres()
            .filter_map(stanard_genre)
            .map(str::to_owned)
            .collect::<Vec<String>>()
            .into_iter()
            .chain(self.take_custom_genres())
    }

    /// Removes all custom genres (`©gen`) and returns the first genre (`gnre` or `©gen`).
    pub fn take_genre(&mut self) -> Option<String> {
        if let Some(g) = self.standard_genre().and_then(stanard_genre) {
            return Some(g.to_owned());
        }

        self.take_custom_genre()
    }

    /// Sets the custom genre (`©gen`). This will remove all other standard or custom genres.
    pub fn set_genre(&mut self, genre: impl Into<String>) {
        self.set_custom_genre(genre.into());
        self.remove_standard_genres();
    }

    /// Sets the custom genre (`©gen`). This will remove all other standard or custom genres.
    pub fn set_genres(&mut self, genres: impl IntoIterator<Item = String>) {
        self.set_custom_genres(genres);
        self.remove_standard_genres();
    }

    /// Adds a custom genre (`©gen`).
    pub fn add_genre(&mut self, genre: impl Into<String>) {
        self.add_custom_genre(genre.into());
    }

    /// Removes the genre (`gnre` or `©gen`).
    pub fn remove_genres(&mut self) {
        self.remove_standard_genres();
        self.remove_custom_genres();
    }

    /// Returns all genres formatted in an easily readable way.
    pub(crate) fn format_genres(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.genres().count() > 1 {
            writeln!(f, "genres:")?;
            for v in self.genres() {
                writeln!(f, "    {}", v)?;
            }
        } else if let Some(s) = self.genre() {
            writeln!(f, "genre: {}", s)?;
        }
        Ok(())
    }
}

fn stanard_genre(code: u16) -> Option<&'static str> {
    let c = code as usize;
    if c > 0 && c <= STANDARD_GENRES.len() {
        return Some(STANDARD_GENRES[c - 1]);
    }

    None
}
