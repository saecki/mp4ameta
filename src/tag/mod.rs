use std::convert::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;

use crate::{
    atom::{self, idents_match, DataIdent, Ident},
    ChannelConfig,
};
use crate::{AdvisoryRating, Atom, AtomData, Data, MediaType};

pub mod genre;
pub mod tuple;

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    /// The `ftyp` atom.
    pub ftyp: String,
    /// The `mvhd` atom.
    pub mvhd: Option<Vec<u8>>,
    /// The `stsd` atom.
    pub mp4a: Option<Vec<u8>>,
    /// A vector containing metadata atoms
    pub atoms: Vec<AtomData>,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::new();

        if let Some(s) = self.format_album_artists() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_artists() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_composers() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_album() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_title() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_genres() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_year() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_track() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_disc() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_duration() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_artworks() {
            string.push_str(&s);
        }
        if let Some(r) = self.advisory_rating() {
            string.push_str(&format!("advisory rating: {}\n", r));
        }
        if let Some(m) = self.media_type() {
            string.push_str(&format!("media type: {}\n", m));
        }
        if let Some(s) = self.format_groupings() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_descriptions() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_comments() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_categories() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_keywords() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_copyright() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_encoder() {
            string.push_str(&s);
        }
        if let Some(i) = self.bpm() {
            string.push_str(&format!("bpm: {}\n", i));
        }
        if let Some(s) = self.format_movement() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_work() {
            string.push_str(&s);
        }
        if let Some(i) = self.movement_count() {
            string.push_str(&format!("movement count: {}\n", i));
        }
        if let Some(i) = self.movement_index() {
            string.push_str(&format!("movement index: {}\n", i));
        }
        if self.show_movement() {
            string.push_str("show movement\n");
        }
        if self.gapless_playback() {
            string.push_str("gapless playback\n");
        }
        if self.compilation() {
            string.push_str("compilation\n");
        }
        if let Some(s) = self.format_lyrics() {
            string.push_str(&s);
        }
        for a in self.atoms.iter() {
            if let (DataIdent::Freeform { .. }, Some(s)) = (&a.ident, &a.data.string()) {
                string.push_str(&format!("{}: {}\n", a.ident, s));
            }
        }

        write!(f, "{}", string)
    }
}

impl Tag {
    /// Creates a new MPEG-4 audio tag containing the atom.
    pub const fn new(
        ftyp: String,
        mvhd: Option<Vec<u8>>,
        mp4a: Option<Vec<u8>>,
        atoms: Vec<AtomData>,
    ) -> Self {
        Self { ftyp, mvhd, mp4a, atoms }
    }

    /// Attempts to read a MPEG-4 audio tag from the reader.
    pub fn read_from(reader: &mut (impl Read + Seek)) -> crate::Result<Self> {
        atom::read_tag_from(reader)
    }

    /// Attempts to read a MPEG-4 audio tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        Self::read_from(&mut file)
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
        let atoms: Vec<Atom> = self.atoms.iter().map(Atom::from).collect();
        atom::dump_tag_to(writer, atoms)
    }

    /// Attempts to dump the MPEG-4 audio tag to the writer.
    pub fn dump_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let mut file = File::create(path)?;
        self.dump_to(&mut file)
    }
}

// ## Individual string values
mp4ameta_proc::individual_string_value_accessor!("album", "©alb");
mp4ameta_proc::individual_string_value_accessor!("copyright", "cprt");
mp4ameta_proc::individual_string_value_accessor!("encoder", "©too");
mp4ameta_proc::individual_string_value_accessor!("lyrics", "©lyr");
mp4ameta_proc::individual_string_value_accessor!("movement", "©mvn");
mp4ameta_proc::individual_string_value_accessor!("title", "©nam");
mp4ameta_proc::individual_string_value_accessor!("tv_episode_number", "tven");
mp4ameta_proc::individual_string_value_accessor!("tv_network_name", "tvnn");
mp4ameta_proc::individual_string_value_accessor!("tv_show_name", "tvsh");
mp4ameta_proc::individual_string_value_accessor!("work", "©wrk");
mp4ameta_proc::individual_string_value_accessor!("year", "©day");

// ## Multiple string values
mp4ameta_proc::multiple_string_values_accessor!("album_artist", "aART");
mp4ameta_proc::multiple_string_values_accessor!("artist", "©ART");
mp4ameta_proc::multiple_string_values_accessor!("category", "catg");
mp4ameta_proc::multiple_string_values_accessor!("comment", "©cmt");
mp4ameta_proc::multiple_string_values_accessor!("composer", "©wrt");
mp4ameta_proc::multiple_string_values_accessor!("custom_genre", "©gen");
mp4ameta_proc::multiple_string_values_accessor!("description", "desc");
mp4ameta_proc::multiple_string_values_accessor!("grouping", "©grp");
mp4ameta_proc::multiple_string_values_accessor!("keyword", "keyw");

// ## Flags
mp4ameta_proc::flag_value_accessor!("compilation", "cpil");
mp4ameta_proc::flag_value_accessor!("gapless_playback", "pgap");
mp4ameta_proc::flag_value_accessor!("show_movement", "shwm");

// ## Integer values
mp4ameta_proc::integer_value_accessor!("bpm", "tmpo");
mp4ameta_proc::integer_value_accessor!("movement_count", "©mvc");
mp4ameta_proc::integer_value_accessor!("movement_index", "©mvi");

// ## Custom values
/// ### Artwork
impl Tag {
    /// Returns all artwork images of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`).
    pub fn artworks(&self) -> impl Iterator<Item = &Data> {
        self.image(&atom::ARTWORK)
    }

    /// Returns the first artwork image of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`).
    pub fn artwork(&self) -> Option<&Data> {
        self.image(&atom::ARTWORK).next()
    }

    /// Consumes and returns all artwork images of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`).
    pub fn take_artworks(&mut self) -> impl Iterator<Item = Data> + '_ {
        self.take_image(&atom::ARTWORK)
    }

    /// Consumes all and returns the first artwork image of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`).
    pub fn take_artwork(&mut self) -> Option<Data> {
        self.take_image(&atom::ARTWORK).next()
    }

    /// Sets the artwork image data of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`). This will remove all other artworks.
    pub fn set_artwork(&mut self, image: Data) {
        if image.is_image() {
            self.set_data(atom::ARTWORK, image);
        }
    }

    /// Adds artwork image data of type [`Data::Jpeg`](crate::Data::Jpeg) or
    /// [`Data::Png`](crate::Data::Png) (`covr`).
    pub fn add_artwork(&mut self, image: Data) {
        if image.is_image() {
            self.add_data(atom::ARTWORK, image);
        }
    }

    /// Removes all artworks (`covr`).
    pub fn remove_artwork(&mut self) {
        self.remove_data(&atom::ARTWORK);
    }

    /// Returns information about all artworks formatted in an easily readable way.
    fn format_artworks(&self) -> Option<String> {
        let format_artwork = |a: &Data| {
            let mut string = String::new();
            match a {
                Data::Png(_) => string.push_str("png"),
                Data::Jpeg(_) => string.push_str("jpeg"),
                _ => unreachable!(),
            }

            let len = a.image_data().unwrap().len();

            if len < 1024 {
                string.push_str(&format!(" {}", len));
            } else if len < 1024usize.pow(2) {
                let size = len / 1024;
                string.push_str(&format!(" {}k", size));
            } else {
                let size = len / 1024usize.pow(2);
                string.push_str(&format!(" {}M", size));
            }

            string.push('\n');

            string
        };

        if self.artworks().count() > 1 {
            let mut string = String::from("artworks:\n");
            for a in self.artworks() {
                string.push_str("    ");
                string.push_str(&format_artwork(a));
            }

            return Some(string);
        }

        let a = self.artwork()?;
        Some(format!("artwork: {}", format_artwork(a)))
    }
}

/// ### Media type
impl Tag {
    /// Returns the media type (`stik`).
    pub fn media_type(&self) -> Option<MediaType> {
        let vec = self.bytes(&atom::MEDIA_TYPE).next()?;

        if vec.is_empty() {
            return None;
        }

        MediaType::try_from(vec[0]).ok()
    }

    /// Sets the media type (`stik`).
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.set_data(atom::MEDIA_TYPE, Data::Reserved(vec![media_type.value()]));
    }

    /// Removes the media type (`stik`).
    pub fn remove_media_type(&mut self) {
        self.remove_data(&atom::MEDIA_TYPE);
    }
}

/// ### Advisory rating
impl Tag {
    /// Returns the advisory rating (`rtng`).
    pub fn advisory_rating(&self) -> Option<AdvisoryRating> {
        let vec = self.bytes(&atom::ADVISORY_RATING).next()?;

        if vec.is_empty() {
            return None;
        }

        Some(AdvisoryRating::from(vec[0]))
    }

    /// Sets the advisory rating (`rtng`).
    pub fn set_advisory_rating(&mut self, rating: AdvisoryRating) {
        self.set_data(atom::ADVISORY_RATING, Data::Reserved(vec![rating.value()]));
    }

    /// Removes the advisory rating (`rtng`).
    pub fn remove_advisory_rating(&mut self) {
        self.remove_data(&atom::ADVISORY_RATING);
    }
}

// ## Readonly values
/// ### Duration
impl Tag {
    /// Returns the duration in seconds.
    pub fn duration(&self) -> crate::Result<f64> {
        let vec = self.mvhd.as_ref().ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::AtomNotFound(atom::MOVIE_HEADER),
                "Missing mvhd atom".to_owned(),
            )
        })?;
        let parsing_err = || {
            crate::Error::new(
                crate::ErrorKind::Parsing,
                "Error parsing contents of mvhd".to_owned(),
            )
        };
        let version = vec.get(0).ok_or_else(parsing_err)?;

        match version {
            0 => {
                // # Version 0
                // 1 byte version
                // 3 bytes flags
                // 4 bytes creation time
                // 4 bytes motification time
                // 4 bytes time scale
                // 4 bytes duration
                // ...
                let timescale_unit = be_int!(vec, 12, u32).ok_or_else(parsing_err)?;
                let duration_units = be_int!(vec, 16, u32).ok_or_else(parsing_err)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Ok(duration)
            }
            1 => {
                // # Version 1
                // 1 byte version
                // 3 bytes flags
                // 8 bytes creation time
                // 8 bytes motification time
                // 4 bytes time scale
                // 8 bytes duration
                // ...
                let timescale_unit = be_int!(vec, 20, u32).ok_or_else(parsing_err)?;
                let duration_units = be_int!(vec, 24, u64).ok_or_else(parsing_err)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Ok(duration)
            }
            v => Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(*v),
                "Duration could not be parsed, unknown mdhd version".to_owned(),
            )),
        }
    }

    /// Returns the duration formatted in an easily readable way.
    fn format_duration(&self) -> Option<String> {
        let total_seconds = self.duration().ok()?.round() as usize;
        let seconds = total_seconds % 60;
        let minutes = total_seconds / 60;

        Some(format!("duration: {}:{:02}\n", minutes, seconds))
    }
}

/// ### Channel config
impl Tag {
    /// Returns the channel config.
    pub fn channel_config(&self) -> crate::Result<ChannelConfig> {
        let vec = self.mp4a.as_ref().ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::AtomNotFound(atom::MPEG4_AUDIO),
                "Missing mp4a atom".to_owned(),
            )
        })?;

        // mp4a structure
        // 4 bytes ?
        // 2 bytes ?
        // 2 bytes data reference index
        // 8 bytes ?
        // 2 bytes channel count
        // 2 bytes sample size
        // 4 bytes ?
        // 4 bytes sample rate
        //
        //   esds box
        //   4 bytes len
        //   4 bytes ident
        //   1 byte version
        //   3 bytes flags
        //
        //     es descriptor
        //     1 byte tag (0x03)
        //     3 bytes len
        //     2 bytes id
        //     1 byte flag
        //
        //       decoder config descriptor
        //       1 byte tag (0x04)
        //       3 bytes len
        //       1 byte object type indication
        //       1 byte stream type
        //       3 bytes buffer size
        //       4 bytes maximum bitrate
        //       4 bytes average bitrate
        //
        //         decoder specific descriptor
        //         1 byte tag (0x05)
        //         3 bytes len
        //
        //       sl config descriptor
        //       1 byte tag (0x06)
        //       3 bytes len
        if let Some(f) = vec.get(28..32) {
            if f == atom::ESDS.as_ref() {
                return Ok(ChannelConfig::Mono);
            }
        }

        Err(crate::Error::new(
            crate::ErrorKind::AtomNotFound(atom::ESDS),
            "Missing esds atom".to_owned(),
        ))
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> &str {
        self.ftyp.as_str()
    }
}

/// ## Data accessors
impl Tag {
    /// Returns all byte data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::BeSigned(b"data".to_vec()));
    /// assert_eq!(tag.bytes(&test).next().unwrap(), b"data");
    /// ```
    pub fn bytes<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &Vec<u8>> {
        self.data(ident).filter_map(Data::bytes)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.bytes_mut(&test).next().unwrap().push(49);
    /// assert_eq!(tag.bytes(&test).next().unwrap(), b"data1");
    /// ```
    pub fn bytes_mut<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = &mut Vec<u8>> {
        self.data_mut(ident).filter_map(Data::bytes_mut)
    }

    /// Consumes all byte data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// assert_eq!(tag.take_bytes(&test).next(), Some(b"data".to_vec()));
    /// assert_eq!(tag.bytes(&test).next(), None);
    /// ```
    pub fn take_bytes<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.take_data(ident).filter_map(Data::take_bytes)
    }

    /// Returns all string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.string(&test).next().unwrap(), "data");
    /// ```
    pub fn string<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &str> {
        self.data(ident).filter_map(Data::string)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// tag.string_mut(&test).next().unwrap().push('1');
    /// assert_eq!(tag.string(&test).next().unwrap(), "data1");
    /// ```
    pub fn string_mut<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = &mut String> {
        self.data_mut(ident).filter_map(Data::string_mut)
    }

    /// Consumes all strings corresponding to the identifier and returns them.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.take_string(&test).next(), Some("data".into()));
    /// assert_eq!(tag.string(&test).next(), None);
    /// ```
    pub fn take_string<'a>(&'a mut self, ident: &'a impl Ident) -> impl Iterator<Item = String> {
        self.take_data(ident).filter_map(Data::take_string)
    }

    /// Returns all image data references of type [Data::Jpeg](crate::Data::Jpeg)
    /// or [Data::Jpeg](crate::Data::Png) corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"<the image data>".to_vec()));
    /// match tag.image(&test).next().unwrap() {
    ///     Data::Jpeg(v) => assert_eq!(*v, b"<the image data>"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn image<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &Data> {
        self.data(ident).filter_map(Data::image)
    }

    /// Returns all mutable image data references of type [Data::Jpeg](crate::Data::Jpeg)
    /// or [Data::Jpeg](crate::Data::Png) corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"<the image data>".to_vec()));
    /// match tag.image_mut(&test).next().unwrap() {
    ///     Data::Jpeg(v) => v.push(49),
    ///     _ => panic!("data type does match"),
    /// }
    /// match tag.image(&test).next().unwrap() {
    ///     Data::Jpeg(v) => assert_eq!(*v, b"<the image data>1"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn image_mut<'a>(&'a mut self, ident: &'a impl Ident) -> impl Iterator<Item = &mut Data> {
        self.data_mut(ident).filter_map(Data::image_mut)
    }

    /// Consumes all data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Png(b"<the image data>".to_vec()));
    /// match tag.take_data(&test).next().unwrap() {
    ///     Data::Png(s) =>  assert_eq!(s, b"<the image data>".to_vec()),
    ///     _ => panic!("data does not match"),
    /// };
    /// assert_eq!(tag.string(&test).next(), None);
    /// ```
    pub fn take_image(&mut self, ident: &impl Ident) -> impl Iterator<Item = Data> {
        self.take_data(ident).filter_map(Data::take_image)
    }

    /// Returns all data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// match tag.data(&test).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn data<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &Data> {
        self.atoms.iter().filter(move |a| idents_match(&a.ident, ident)).map(|a| &a.data)
    }

    /// Returns all mutable data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// if let Data::Utf8(s) = tag.data_mut(&test).next().unwrap() {
    ///     s.push('1');
    /// }
    /// assert_eq!(tag.string(&test).next().unwrap(), "data1");
    /// ```
    pub fn data_mut<'a>(&'a mut self, ident: &'a impl Ident) -> impl Iterator<Item = &mut Data> {
        self.atoms.iter_mut().filter(move |a| idents_match(&a.ident, ident)).map(|a| &mut a.data)
    }

    /// Consumes all data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// match tag.take_data(&test).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data".to_string()),
    ///     _ => panic!("data does not match"),
    /// };
    /// assert_eq!(tag.string(&test).next(), None);
    /// ```
    pub fn take_data(&mut self, ident: &impl Ident) -> impl Iterator<Item = Data> {
        let mut data = Vec::new();

        let mut i = 0;
        while i < self.atoms.len() {
            if idents_match(&self.atoms[i].ident, ident) {
                let removed = self.atoms.swap_remove(i);
                data.push(removed.data);
            } else {
                i += 1;
            }
        }

        data.into_iter()
    }

    /// Removes all other atoms, corresponding to the identifier, and adds a new atom containing the
    /// provided data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.string(&test).next().unwrap(), "data");
    /// ```
    pub fn set_data(&mut self, ident: impl Into<DataIdent>, data: Data) {
        let ident = ident.into();
        self.remove_data(&ident);
        self.atoms.push(AtomData::new(ident, data));
    }

    /// Adds a new atom, corresponding to the identifier, containing the provided data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("data1".into()));
    /// tag.add_data(test, Data::Utf8("data2".into()));
    /// let mut strings = tag.string(&test);
    /// assert_eq!(strings.next(), Some("data1"));
    /// assert_eq!(strings.next(), Some("data2"));
    /// assert_eq!(strings.next(), None)
    /// ```
    pub fn add_data(&mut self, ident: impl Into<DataIdent>, data: Data) {
        self.atoms.push(AtomData::new(ident.into(), data));
    }

    /// Removes the data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, FourCC};
    ///
    /// let mut tag = Tag::default();
    /// let test = FourCC(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(tag.data(&test).next().is_some());
    /// tag.remove_data(&test);
    /// assert!(tag.data(&test).next().is_none());
    /// ```
    pub fn remove_data(&mut self, ident: &impl Ident) {
        self.atoms.retain(|a| !idents_match(&a.ident, ident));
    }
}
