use std::convert::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;
use std::rc::Rc;

pub use genre::*;
pub use readonly::*;
pub use tuple::*;

use crate::Img;
use crate::ImgMut;
use crate::{
    atom, be_int, ident, AdvisoryRating, AtomData, AudioInfo, Data, DataIdent, Ident, ImgBuf,
    ImgFmt, ImgRef, MediaType,
};

mod genre;
mod readonly;
mod tuple;

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    /// The `ftyp` atom.
    ftyp: String,
    /// Readonly audio information
    info: AudioInfo,
    /// A vector containing metadata atoms
    atoms: Vec<AtomData>,
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
        if let Some(s) = self.format_tv_show_name() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_tv_network_name() {
            string.push_str(&s);
        }
        if let Some(s) = self.format_tv_episode_name() {
            string.push_str(&s);
        }
        if let Some(e) = self.tv_episode() {
            string.push_str(&format!("tv episode: {}\n", e));
        }
        if let Some(s) = self.tv_season() {
            string.push_str(&format!("tv season: {}\n", s));
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
        if let Some(s) = self.format_duration() {
            string.push_str(&s);
        }
        if let Some(c) = self.channel_config() {
            string.push_str(&format!("channel config: {}\n", c));
        }
        if let Some(s) = self.sample_rate() {
            string.push_str(&format!("sample rate: {}\n", s));
        }
        if let Some(a) = self.avg_bitrate() {
            string.push_str(&format!("average bitrate: {}kbps\n", a / 1024));
        }
        if let Some(m) = self.max_bitrate() {
            string.push_str(&format!("maximum bitrate: {}kbps\n", m / 1024));
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
            if let DataIdent::Freeform { .. } = &a.ident {
                string.push_str(&format!("{}:\n", a.ident));
                a.data.iter().filter_map(|a| a.string()).for_each(|s| {
                    string.push_str(s);
                    string.push('\n');
                });
            }
        }
        string.push_str("filetype: ");
        string.push_str(self.filetype());
        string.push('\n');

        write!(f, "{}", string)
    }
}

impl Tag {
    /// Creates a new MPEG-4 audio tag containing the atom.
    pub const fn new(ftyp: String, info: AudioInfo, atoms: Vec<AtomData>) -> Self {
        Self { ftyp, info, atoms }
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
        atom::dump_tag_to(writer, &self.atoms)
    }

    /// Attempts to dump the MPEG-4 audio tag to the writer.
    pub fn dump_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let mut file = File::create(path)?;
        self.dump_to(&mut file)
    }
}

// ## Individual string values
mp4ameta_proc::single_string_value_accessor!("album", "©alb");
mp4ameta_proc::single_string_value_accessor!("copyright", "cprt");
mp4ameta_proc::single_string_value_accessor!("encoder", "©too");
mp4ameta_proc::single_string_value_accessor!("lyrics", "©lyr");
mp4ameta_proc::single_string_value_accessor!("movement", "©mvn");
mp4ameta_proc::single_string_value_accessor!("title", "©nam");
mp4ameta_proc::single_string_value_accessor!("tv_episode_name", "tven");
mp4ameta_proc::single_string_value_accessor!("tv_network_name", "tvnn");
mp4ameta_proc::single_string_value_accessor!("tv_show_name", "tvsh");
mp4ameta_proc::single_string_value_accessor!("work", "©wrk");
mp4ameta_proc::single_string_value_accessor!("year", "©day");
mp4ameta_proc::single_string_value_accessor!("isrc", "----:com.apple.iTunes:ISRC");

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
mp4ameta_proc::multiple_string_values_accessor!("lyricist", "----:com.apple.iTunes:LYRICIST");

// ## Flags
mp4ameta_proc::flag_value_accessor!("compilation", "cpil");
mp4ameta_proc::flag_value_accessor!("gapless_playback", "pgap");
mp4ameta_proc::flag_value_accessor!("show_movement", "shwm");

// ## Integer values
mp4ameta_proc::u16_value_accessor!("bpm", "tmpo");
mp4ameta_proc::u16_value_accessor!("movement_count", "©mvc");
mp4ameta_proc::u16_value_accessor!("movement_index", "©mvi");

mp4ameta_proc::u32_value_accessor!("tv_episode", "tves");
mp4ameta_proc::u32_value_accessor!("tv_season", "tvsn");

// ## Custom values
/// ### Artwork
impl Tag {
    /// Returns all artwork images (`covr`).
    pub fn artworks(&self) -> impl Iterator<Item = ImgRef> {
        self.images_of(&ident::ARTWORK)
    }

    /// Returns the first artwork image (`covr`).
    pub fn artwork(&self) -> Option<ImgRef> {
        self.images_of(&ident::ARTWORK).next()
    }

    /// Removes and returns all artwork images (`covr`).
    pub fn take_artworks(&mut self) -> impl Iterator<Item = ImgBuf> + '_ {
        self.take_images_of(&ident::ARTWORK)
    }

    /// Removes all and returns the first artwork image (`covr`).
    pub fn take_artwork(&mut self) -> Option<ImgBuf> {
        self.take_images_of(&ident::ARTWORK).next()
    }

    /// Sets the artwork image data (`covr`). This will remove all other artworks.
    pub fn set_artwork(&mut self, image: ImgBuf) {
        self.set_data(ident::ARTWORK, image.into());
    }

    /// Sets all artwork image data (`covr`). This will remove all other artworks.
    pub fn set_artworks(&mut self, images: impl IntoIterator<Item = ImgBuf>) {
        self.set_all_data(ident::ARTWORK, images.into_iter().map(Img::into));
    }

    /// Adds artwork image data (`covr`).
    pub fn add_artwork(&mut self, image: ImgBuf) {
        self.add_data(ident::ARTWORK, image.into());
    }

    /// Adds artwork image data (`covr`).
    pub fn add_artworks(&mut self, images: impl IntoIterator<Item = ImgBuf>) {
        self.add_all_data(ident::ARTWORK, images.into_iter().map(Img::into));
    }

    /// Removes all artworks (`covr`).
    pub fn remove_artworks(&mut self) {
        self.remove_data_of(&ident::ARTWORK);
    }

    /// Returns information about all artworks formatted in an easily readable way.
    fn format_artworks(&self) -> Option<String> {
        let format_artwork = |i: ImgRef| {
            let mut string = String::new();
            match i.fmt {
                ImgFmt::Png => string.push_str("png"),
                ImgFmt::Jpeg => string.push_str("jpeg"),
                ImgFmt::Bmp => string.push_str("bmp"),
            }

            let len = i.data.len();

            if len < 1024 {
                string.push_str(&format!(" {}", len));
            } else if len < 1024 * 1024 {
                let size = len / 1024;
                string.push_str(&format!(" {}k", size));
            } else {
                let size = len / (1024 * 1024);
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
        let vec = self.bytes_of(&ident::MEDIA_TYPE).next()?;

        if vec.is_empty() {
            return None;
        }

        MediaType::try_from(vec[0]).ok()
    }

    /// Sets the media type (`stik`).
    pub fn set_media_type(&mut self, media_type: MediaType) {
        self.set_data(ident::MEDIA_TYPE, Data::Reserved(vec![media_type.value()]));
    }

    /// Removes the media type (`stik`).
    pub fn remove_media_type(&mut self) {
        self.remove_data_of(&ident::MEDIA_TYPE);
    }
}

/// ### Advisory rating
impl Tag {
    /// Returns the advisory rating (`rtng`).
    pub fn advisory_rating(&self) -> Option<AdvisoryRating> {
        let vec = self.bytes_of(&ident::ADVISORY_RATING).next()?;

        if vec.is_empty() {
            return None;
        }

        Some(AdvisoryRating::from(vec[0]))
    }

    /// Sets the advisory rating (`rtng`).
    pub fn set_advisory_rating(&mut self, rating: AdvisoryRating) {
        self.set_data(ident::ADVISORY_RATING, Data::Reserved(vec![rating.value()]));
    }

    /// Removes the advisory rating (`rtng`).
    pub fn remove_advisory_rating(&mut self) {
        self.remove_data_of(&ident::ADVISORY_RATING);
    }
}

/// ## Data accessors
impl Tag {
    /// Returns all byte data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::BeSigned(b"data".to_vec()));
    /// assert_eq!(tag.bytes_of(&test).next().unwrap(), b"data");
    /// ```
    pub fn bytes_of<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &[u8]> {
        self.data_of(ident).filter_map(Data::bytes)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.bytes_mut_of(&test).next().unwrap().push('1' as u8);
    /// assert_eq!(tag.bytes_of(&test).next().unwrap(), b"data1");
    /// ```
    pub fn bytes_mut_of<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = &mut Vec<u8>> {
        self.data_mut_of(ident).filter_map(Data::bytes_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's byte data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// assert_eq!(tag.take_bytes_of(&test).next(), Some(b"data".to_vec()));
    /// assert_eq!(tag.bytes_of(&test).next(), None);
    /// ```
    pub fn take_bytes_of<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.take_data_of(ident).filter_map(Data::into_bytes)
    }

    /// Returns all string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data");
    /// ```
    pub fn strings_of<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &str> {
        self.data_of(ident).filter_map(Data::string)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// tag.strings_mut_of(&test).next().unwrap().push('1');
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data1");
    /// ```
    pub fn strings_mut_of<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = &mut String> {
        self.data_mut_of(ident).filter_map(Data::string_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's strings.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.take_strings_of(&test).next(), Some("data".into()));
    /// assert_eq!(tag.strings_of(&test).next(), None);
    /// ```
    pub fn take_strings_of<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = String> {
        self.take_data_of(ident).filter_map(Data::into_string)
    }

    /// Returns all image references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"<the image data>".to_vec()));
    /// let img = tag.images_of(&test).next().unwrap();
    /// assert_eq!(img.data, b"<the image data>");
    /// ```
    pub fn images_of<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = ImgRef> {
        self.data_of(ident).filter_map(Data::image)
    }

    /// Returns all mutable image references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"<the image data>".to_vec()));
    /// let img = tag.images_mut_of(&test).next().unwrap();
    /// img.data.push(49);
    ///
    /// let img = tag.images_of(&test).next().unwrap();
    /// assert_eq!(img.data, b"<the image data>1");
    /// ```
    pub fn images_mut_of<'a>(
        &'a mut self,
        ident: &'a impl Ident,
    ) -> impl Iterator<Item = ImgMut<'a>> {
        self.data_mut_of(ident).filter_map(Data::image_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's images.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Png(b"<the image data>".to_vec()));
    /// let img = tag.take_images_of(&test).next().unwrap();
    /// assert_eq!(img.data, b"<the image data>".to_vec());
    /// ```
    pub fn take_images_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = ImgBuf> {
        self.take_data_of(ident).filter_map(Data::into_image)
    }

    /// Returns all data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// match tag.data_of(&test).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn data_of<'a>(&'a self, ident: &'a impl Ident) -> impl Iterator<Item = &Data> {
        match self.atoms.iter().find(|a| ident == &a.ident) {
            Some(a) => a.data.iter(),
            None => [].iter(),
        }
    }

    /// Returns all mutable data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// if let Data::Utf8(s) = tag.data_mut_of(&test).next().unwrap() {
    ///     s.push('1');
    /// }
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data1");
    /// ```
    pub fn data_mut_of<'a>(&'a mut self, ident: &'a impl Ident) -> impl Iterator<Item = &mut Data> {
        match self.atoms.iter_mut().find(|a| ident == &a.ident) {
            Some(a) => a.data.iter_mut(),
            None => [].iter_mut(),
        }
    }

    /// Removes the atom corresponding to the identifier and returns all of it's data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// match tag.take_data_of(&test).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data".to_string()),
    ///     _ => panic!("data does not match"),
    /// };
    /// assert_eq!(tag.strings_of(&test).next(), None);
    /// ```
    pub fn take_data_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = Data> {
        let mut i = 0;
        while i < self.atoms.len() {
            if ident == &self.atoms[i].ident {
                let removed = self.atoms.remove(i);
                return removed.data.into_iter();
            }

            i += 1;
        }

        Vec::new().into_iter()
    }

    /// TODO
    pub fn bytes(&self) -> impl Iterator<Item = (&DataIdent, &[u8])> {
        self.data().filter_map(|(i, d)| Some((i, d.bytes()?)))
    }

    /// TODO
    pub fn bytes_mut(&mut self) -> impl Iterator<Item = (&DataIdent, &mut Vec<u8>)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.bytes_mut()?)))
    }

    /// TODO
    pub fn take_bytes(self) -> impl Iterator<Item = (Rc<DataIdent>, Vec<u8>)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_bytes()?)))
    }

    /// TODO
    pub fn strings(&self) -> impl Iterator<Item = (&DataIdent, &str)> {
        self.data().filter_map(|(i, d)| Some((i, d.string()?)))
    }

    /// TODO
    pub fn strings_mut(&mut self) -> impl Iterator<Item = (&DataIdent, &mut String)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.string_mut()?)))
    }

    /// TODO
    pub fn take_strings(self) -> impl Iterator<Item = (Rc<DataIdent>, String)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_string()?)))
    }

    /// TODO
    pub fn images(&self) -> impl Iterator<Item = (&DataIdent, ImgRef)> {
        self.data().filter_map(|(i, d)| Some((i, d.image()?)))
    }

    /// TODO
    pub fn images_mut(&mut self) -> impl Iterator<Item = (&DataIdent, ImgMut)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.image_mut()?)))
    }

    /// TODO
    pub fn take_images(self) -> impl Iterator<Item = (Rc<DataIdent>, ImgBuf)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_image()?)))
    }

    /// TODO
    pub fn data(&self) -> impl Iterator<Item = (&DataIdent, &Data)> {
        self.atoms.iter().flat_map(|a| a.data.iter().map(move |d| (&a.ident, d)))
    }

    /// TODO
    pub fn data_mut<'b>(&'b mut self) -> impl Iterator<Item = (&'b DataIdent, &'b mut Data)> {
        self.atoms.iter_mut().flat_map(|a| {
            let ident = &a.ident;
            let data = &mut a.data;
            data.iter_mut().map(move |d| (ident, d))
        })
    }

    /// TODO
    pub fn take_data<'a>(self) -> impl Iterator<Item = (Rc<DataIdent>, Data)> {
        self.atoms.into_iter().flat_map(move |a| {
            let ident = Rc::new(a.ident);
            let data = a.data;
            data.into_iter().map(move |d| (ident.clone(), d))
        })
    }

    /// TODO
    pub fn remove_bytes_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_bytes());
    }

    /// TODO
    pub fn remove_strings_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_string());
    }

    /// TODO
    pub fn remove_images_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_image());
    }

    /// Removes all data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(tag.data_of(&test).next().is_some());
    /// tag.remove_data_of(&test);
    /// assert!(tag.data_of(&test).next().is_none());
    /// ```
    pub fn remove_data_of(&mut self, ident: &impl Ident) {
        self.atoms.retain(|a| ident != &a.ident);
    }

    /// Retains only the bytes, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::BeSigned(vec![4u8; 12]));
    /// tag.add_data(test, Data::Reserved(vec![6u8; 16]));
    ///
    /// let mut bytes = tag.bytes_of(&test);
    /// assert_eq!(bytes.next(), Some(&[4u8; 12][..]));
    /// assert_eq!(bytes.next(), Some(&[6u8; 16][..]));
    /// assert_eq!(bytes.next(), None);
    /// drop(bytes);
    ///
    /// tag.retain_bytes_of(&test, |b| b[2..4] == [4u8, 4u8]);
    ///
    /// let mut bytes = tag.bytes_of(&test);
    /// assert_eq!(bytes.next(), Some(&[4u8; 12][..]));
    /// assert_eq!(bytes.next(), None);
    /// ```
    pub fn retain_bytes_of(&mut self, ident: &impl Ident, predicate: impl Fn(&[u8]) -> bool) {
        self.retain_data_of(ident, |d| d.bytes().map_or(true, |b| predicate(b)))
    }

    /// Retains only the strings, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc, Img, ImgFmt};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string1".into()));
    /// tag.add_data(test, Data::Utf8("string2".into()));
    ///
    /// let mut strings = tag.strings_of(&test);
    /// assert_eq!(strings.next(), Some("string1"));
    /// assert_eq!(strings.next(), Some("string2"));
    /// assert_eq!(strings.next(), None);
    /// drop(strings);
    ///
    /// tag.retain_strings_of(&test, |s| s.ends_with("1"));
    ///
    /// let mut strings = tag.strings_of(&test);
    /// assert_eq!(strings.next(), Some("string1"));
    /// assert_eq!(strings.next(), None);
    /// ```
    pub fn retain_strings_of(&mut self, ident: &impl Ident, predicate: impl Fn(&str) -> bool) {
        self.retain_data_of(ident, |d| d.string().map_or(true, |s| predicate(s)))
    }

    /// Retains only the images, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc, Img, ImgFmt};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Png(vec![5u8; 4]));
    /// tag.add_data(test, Data::Jpeg(vec![6u8; 16]));
    ///
    /// let mut images = tag.images_of(&test);
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Png, &[5u8; 4][..])));
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Jpeg, &[6u8; 16][..])));
    /// assert_eq!(images.next(), None);
    /// drop(images);
    ///
    /// tag.retain_images_of(&test, |d| d.fmt == ImgFmt::Jpeg);
    ///
    /// let mut images = tag.images_of(&test);
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Jpeg, &[6u8; 16][..])));
    /// assert_eq!(images.next(), None);
    /// ```
    pub fn retain_images_of(&mut self, ident: &impl Ident, predicate: impl Fn(ImgRef) -> bool) {
        self.retain_data_of(ident, |d| d.image().map_or(true, |i| predicate(i)))
    }

    /// Retains only the data, of the atom corresponding to the identifier, that matches the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("short".into()));
    /// tag.add_data(test, Data::Reserved(vec![6u8; 16]));
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("short".into())));
    /// assert_eq!(data.next(), Some(&Data::Reserved(vec![6u8; 16])));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_data_of(&test, |d| d.len() < 10);
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("short".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_data_of(&mut self, ident: &impl Ident, predicate: impl Fn(&Data) -> bool) {
        let pos = self.atoms.iter().position(|a| ident == &a.ident);

        if let Some(i) = pos {
            self.atoms[i].data.retain(predicate);
            if self.atoms[i].data.is_empty() {
                self.atoms.remove(i);
            }
        }
    }

    /// TODO
    pub fn retain_bytes(&mut self, predicate: impl Fn(&DataIdent, &[u8]) -> bool) {
        self.retain_data(|i, d| d.bytes().map_or(true, |s| predicate(i, s)));
    }

    /// TODO
    pub fn retain_strings(&mut self, predicate: impl Fn(&DataIdent, &str) -> bool) {
        self.retain_data(|i, d| d.string().map_or(true, |s| predicate(i, s)));
    }

    /// TODO
    pub fn retain_images(&mut self, predicate: impl Fn(&DataIdent, ImgRef) -> bool) {
        self.retain_data(|i, d| d.image().map_or(true, |s| predicate(i, s)));
    }

    /// Retains only the data matching the predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let tst1 = Fourcc(*b"tst1");
    /// let tst2 = Fourcc(*b"tst2");
    ///
    /// tag.add_data(tst1, Data::Utf8("data1".into()));
    /// tag.add_data(tst1, Data::Utf8("data2".into()));
    /// tag.add_data(tst2, Data::Utf8("data3".into()));
    /// tag.add_data(tst2, Data::Utf8("data4".into()));
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data1".into())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data2".into())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data3".into())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data4".into())));
    /// drop(data);
    ///
    /// tag.retain_data(|i, d| &tst1 != i);
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data1".into())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data2".into())));
    /// ```
    pub fn retain_data(&mut self, predicate: impl Fn(&DataIdent, &Data) -> bool) {
        let mut i = 0;
        while i < self.atoms.len() {
            let a = &mut self.atoms[i];
            let mut j = 0;
            while j < a.data.len() {
                if predicate(&a.ident, &a.data[j]) {
                    a.data.remove(j);
                } else {
                    j += 1;
                }
            }

            if a.data.is_empty() {
                self.atoms.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Clears all data from the tag.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// assert!(tag.is_empty());
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(!tag.is_empty());
    /// tag.clear();
    /// assert!(tag.is_empty());
    pub fn clear(&mut self) {
        self.atoms.clear();
    }

    /// If an atom corresponding to the identifier exists, it's data will be replaced by the new
    /// data, otherwise a new atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data");
    /// ```
    pub fn set_data(&mut self, ident: (impl Ident + Into<DataIdent>), data: Data) {
        match self.atoms.iter_mut().find(|a| &ident == &a.ident) {
            Some(a) => {
                a.data.clear();
                a.data.push(data);
            }
            None => self.atoms.push(AtomData::new(ident.into(), vec![data])),
        }
    }

    /// If an atom corresponding to the identifier exists, it's data will be replaced by the new
    /// data, otherwise a new atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// let data = vec![
    ///     Data::Utf8("data1".into()),
    ///     Data::Utf8("data2".into()),
    /// ];
    /// tag.set_all_data(test, data);
    ///
    /// let mut strings = tag.strings_of(&test);
    /// assert_eq!(strings.next(), Some("data1"));
    /// assert_eq!(strings.next(), Some("data2"));
    /// assert_eq!(strings.next(), None);
    /// ```
    pub fn set_all_data(
        &mut self,
        ident: (impl Ident + Into<DataIdent>),
        data: impl IntoIterator<Item = Data>,
    ) {
        match self.atoms.iter_mut().find(|a| &ident == &a.ident) {
            Some(a) => {
                a.data.clear();
                a.data.extend(data);
            }
            None => {
                self.atoms.push(AtomData::new(ident.into(), data.into_iter().collect()));
            }
        }
    }

    /// If an atom corresponding to the identifier exists, the new data will be added to it,
    /// otherwise a new atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("data1".into()));
    /// tag.add_data(test, Data::Utf8("data2".into()));
    ///
    /// let mut strings = tag.strings_of(&test);
    /// assert_eq!(strings.next(), Some("data1"));
    /// assert_eq!(strings.next(), Some("data2"));
    /// assert_eq!(strings.next(), None)
    /// ```
    pub fn add_data(&mut self, ident: (impl Ident + Into<DataIdent>), data: Data) {
        match self.atoms.iter_mut().find(|a| &ident == &a.ident) {
            Some(a) => a.data.push(data),
            None => self.atoms.push(AtomData::new(ident.into(), vec![data])),
        }
    }

    /// If an atom corresponding to the identifier exists, the new data will be added to it,
    /// otherwise a new atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// let data = vec![
    ///     Data::Utf8("data1".into()),
    ///     Data::Utf8("data2".into()),
    /// ];
    /// tag.add_all_data(test, data);
    ///
    /// let mut strings = tag.strings_of(&test);
    /// assert_eq!(strings.next(), Some("data1"));
    /// assert_eq!(strings.next(), Some("data2"));
    /// assert_eq!(strings.next(), None)
    /// ```
    pub fn add_all_data(
        &mut self,
        ident: (impl Ident + Into<DataIdent>),
        data: impl IntoIterator<Item = Data>,
    ) {
        match self.atoms.iter_mut().find(|a| &ident == &a.ident) {
            Some(a) => a.data.extend(data),
            None => self.atoms.push(AtomData::new(ident.into(), data.into_iter().collect())),
        }
    }

    /// Returns wheter this tag contains no metadata atoms.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    ///
    /// let mut tag = Tag::default();
    /// let test = Fourcc(*b"test");
    ///
    /// assert!(tag.is_empty());
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(!tag.is_empty());
    /// tag.clear();
    /// assert!(tag.is_empty());
    /// ```
    /// use mp4ameta::{Tag, Data, Fourcc};
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
}
