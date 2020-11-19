use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;

use crate::{atom, AdvisoryRating, Atom, AtomData, Content, Data, Ident, MediaType};

pub mod genre;
pub mod tuple;

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    /// The `ftyp` atom.
    pub ftyp: Option<String>,
    /// The `mdhd` atom.
    pub mdhd: Option<Vec<u8>>,
    /// A vector containing metadata atoms
    pub atoms: Vec<Atom>,
}

impl IntoIterator for Tag {
    type Item = AtomData;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.atoms
            .into_iter()
            .filter_map(AtomData::try_from_typed)
            .collect::<Vec<AtomData>>()
            .into_iter()
    }
}

impl Tag {
    /// Creates a new MPEG-4 audio tag containing the atom.
    pub const fn new(ftyp: Option<String>, mdhd: Option<Vec<u8>>, atoms: Vec<Atom>) -> Self {
        Self { ftyp, mdhd, atoms }
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
        atom::dump_tag_to(writer, self.atoms.clone())
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
    /// Returns all artwork images of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`).
    pub fn artworks(&self) -> impl Iterator<Item = &Data> {
        self.image(atom::ARTWORK)
    }

    /// Returns the first artwork image of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`).
    pub fn artwork(&self) -> Option<&Data> {
        self.image(atom::ARTWORK).next()
    }

    /// Consumes and returns all artwork images of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`).
    pub fn take_artworks(&mut self) -> impl Iterator<Item = Data> + '_ {
        self.take_image(atom::ARTWORK)
    }

    /// Consumes all and returns the first artwork image of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`).
    pub fn take_artwork(&mut self) -> Option<Data> {
        self.take_image(atom::ARTWORK).next()
    }

    /// Sets the artwork image data of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`). This will remove all other artworks.
    pub fn set_artwork(&mut self, image: Data) {
        if image.is_image() {
            self.set_data(atom::ARTWORK, image);
        }
    }

    /// Adds artwork image data of type [`Data::Jpeg`](enum.Data.html#variant.Jpeg) or
    /// [`Data::Png`](enum.Data.html#variant.Png) (`covr`).
    pub fn add_artwork(&mut self, image: Data) {
        if image.is_image() {
            self.add_data(atom::ARTWORK, image);
        }
    }

    /// Removes all artworks (`covr`).
    pub fn remove_artwork(&mut self) {
        self.remove_data(atom::ARTWORK);
    }
}

/// ### Media type
impl Tag {
    /// Returns the media type (`stik`).
    pub fn media_type(&self) -> Option<MediaType> {
        let vec = self.bytes(atom::MEDIA_TYPE).next()?;

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
        self.remove_data(atom::MEDIA_TYPE);
    }
}

/// ### Advisory rating
impl Tag {
    /// Returns the advisory rating (`rtng`).
    pub fn advisory_rating(&self) -> Option<AdvisoryRating> {
        let vec = self.bytes(atom::ADVISORY_RATING).next()?;

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
        self.remove_data(atom::ADVISORY_RATING);
    }
}

// ## Readonly values
/// ### Duration
impl Tag {
    /// Returns the duration in seconds.
    pub fn duration(&self) -> Option<f64> {
        // https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html#//apple_ref/doc/uid/TP40000939-CH204-SW34
        // https://wiki.multimedia.cx/index.php/QuickTime_container#mdhd

        let vec = self.mdhd.as_ref()?;
        let version = vec.get(0)?;

        match version {
            0 => {
                // Version 0
                // 1 byte    version
                // 3 bytes   flags
                // 4 bytes   creation time
                // 4 bytes   modification time
                // 4 bytes   time scale
                // 4 bytes   duration
                // 2 bytes   language
                // 2 bytes   quality

                let timescale_unit = be_int!(vec, 12, u32)?;
                let duration_units = be_int!(vec, 16, u32)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Some(duration)
            }
            1 => {
                // Version 1
                // 1 byte    version
                // 3 bytes   flags
                // 8 bytes   creation time
                // 8 bytes   modification time
                // 4 bytes   time scale
                // 8 bytes   duration
                // 2 bytes   language
                // 2 bytes   quality

                let timescale_unit = be_int!(vec, 20, u32)?;
                let duration_units = be_int!(vec, 24, u64)?;

                let duration = duration_units as f64 / timescale_unit as f64;

                Some(duration)
            }
            _ => None,
        }
    }
}

/// ### Filetype
impl Tag {
    /// returns the filetype (`ftyp`).
    pub fn filetype(&self) -> Option<&str> {
        self.ftyp.as_deref()
    }
}

/// ## Data accessors
impl Tag {
    /// Returns all byte data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::BeSigned(b"data".to_vec()));
    /// assert_eq!(tag.bytes(Ident(*b"test")).next().unwrap(), b"data");
    /// ```
    pub fn bytes(&self, ident: Ident) -> impl Iterator<Item = &Vec<u8>> {
        self.data(ident).filter_map(Data::bytes)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Reserved(b"data".to_vec()));
    /// tag.bytes_mut(Ident(*b"test")).next().unwrap().push(49);
    /// assert_eq!(tag.bytes(Ident(*b"test")).next().unwrap(), b"data1");
    /// ```
    pub fn bytes_mut(&mut self, ident: Ident) -> impl Iterator<Item = &mut Vec<u8>> {
        self.data_mut(ident).filter_map(Data::bytes_mut)
    }

    /// Consumes all byte data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Reserved(b"data".to_vec()));
    /// assert_eq!(tag.take_bytes(Ident(*b"test")).next(), Some(b"data".to_vec()));
    /// assert_eq!(tag.bytes(Ident(*b"test")).next(), None);
    /// ```
    pub fn take_bytes(&mut self, ident: Ident) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.take_data(ident).filter_map(Data::take_bytes)
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
    pub fn string(&self, ident: Ident) -> impl Iterator<Item = &str> {
        self.data(ident).filter_map(Data::string)
    }

    /// Returns all mutable string references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// tag.string_mut(Ident(*b"test")).next().unwrap().push('1');
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data1");
    /// ```
    pub fn string_mut(&mut self, ident: Ident) -> impl Iterator<Item = &mut String> {
        self.data_mut(ident).filter_map(Data::string_mut)
    }

    /// Consumes all strings corresponding to the identifier and returns them.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// assert_eq!(tag.take_string(Ident(*b"test")).next(), Some("data".into()));
    /// assert_eq!(tag.string(Ident(*b"test")).next(), None);
    /// ```
    pub fn take_string(&mut self, ident: Ident) -> impl Iterator<Item = String> + '_ {
        self.take_data(ident).filter_map(Data::take_string)
    }

    /// Returns all image data references of type [Data::Jpeg](enum.Data.html#variant.Jpeg)
    /// or [Data::Jpeg](enum.Data.html#variant.Png) corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Jpeg(b"<the image data>".to_vec()));
    /// match tag.image(Ident(*b"test")).next().unwrap() {
    ///     Data::Jpeg(v) => assert_eq!(*v, b"<the image data>"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn image(&self, ident: Ident) -> impl Iterator<Item = &Data> {
        self.data(ident).filter_map(Data::image)
    }

    /// Returns all mutable image data references of type [Data::Jpeg](enum.Data.html#variant.Jpeg)
    /// or [Data::Jpeg](enum.Data.html#variant.Png) corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Jpeg(b"<the image data>".to_vec()));
    /// match tag.image_mut(Ident(*b"test")).next().unwrap() {
    ///     Data::Jpeg(v) => v.push(49u8),
    ///     _ => panic!("data type does match"),
    /// }
    /// match tag.image(Ident(*b"test")).next().unwrap() {
    ///     Data::Jpeg(v) => assert_eq!(*v, b"<the image data>1"),
    ///     _ => panic!("data does not match"),
    /// };
    /// ```
    pub fn image_mut(&mut self, ident: Ident) -> impl Iterator<Item = &mut Data> {
        self.data_mut(ident).filter_map(Data::image_mut)
    }

    /// Consumes all data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Png(b"<the image data>".to_vec()));
    /// match tag.take_data(Ident(*b"test")).next().unwrap() {
    ///     Data::Png(s) =>  assert_eq!(s, b"<the image data>".to_vec()),
    ///     _ => panic!("data does not match"),
    /// };
    /// assert_eq!(tag.string(Ident(*b"test")).next(), None);
    /// ```
    pub fn take_image(&mut self, ident: Ident) -> impl Iterator<Item = Data> + '_ {
        self.take_data(ident).filter_map(Data::take_image)
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
    pub fn data(&self, ident: Ident) -> impl Iterator<Item = &Data> {
        self.atoms.iter().filter_map(move |a| {
            if a.ident == ident {
                if let Some(d) = a.child(atom::DATA) {
                    if let Content::TypedData(data) = &d.content {
                        return Some(data);
                    }
                }
            }
            None
        })
    }

    /// Returns all mutable data references corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// if let Data::Utf8(s) = tag.data_mut(Ident(*b"test")).next().unwrap() {
    ///     s.push('1');
    /// }
    /// assert_eq!(tag.string(Ident(*b"test")).next().unwrap(), "data1");
    /// ```
    pub fn data_mut(&mut self, ident: Ident) -> impl Iterator<Item = &mut Data> {
        self.atoms.iter_mut().filter_map(move |a| {
            if a.ident == ident {
                if let Some(d) = a.child_mut(atom::DATA) {
                    if let Content::TypedData(data) = &mut d.content {
                        return Some(data);
                    }
                }
            }
            None
        })
    }

    /// Consumes all data corresponding to the identifier and returns it.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Tag, Data, Ident};
    ///
    /// let mut tag = Tag::default();
    /// tag.set_data(Ident(*b"test"), Data::Utf8("data".into()));
    /// match tag.take_data(Ident(*b"test")).next().unwrap() {
    ///     Data::Utf8(s) =>  assert_eq!(s, "data".to_string()),
    ///     _ => panic!("data does not match"),
    /// };
    /// assert_eq!(tag.string(Ident(*b"test")).next(), None);
    /// ```
    pub fn take_data(&mut self, ident: Ident) -> impl Iterator<Item = Data> + '_ {
        self.atoms.iter_mut().filter_map(move |a| {
            if a.ident == ident {
                if let Some(d) = a.child_mut(atom::DATA) {
                    return d.content.take_data();
                }
            }
            None
        })
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
        self.atoms.push(Atom::new(ident, 0, Content::data_atom_with(data)));
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
    /// assert_eq!(strings.next(), Some("data1"));
    /// assert_eq!(strings.next(), Some("data2"));
    /// assert_eq!(strings.next(), None)
    /// ```
    pub fn add_data(&mut self, ident: Ident, data: Data) {
        self.atoms.push(Atom::new(ident, 0, Content::data_atom_with(data)));
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
