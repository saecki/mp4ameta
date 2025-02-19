use std::convert::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::rc::Rc;

use crate::{
    atom, ident, AdvisoryRating, Chapter, Data, DataIdent, Ident, Img, ImgBuf, ImgFmt, ImgMut,
    ImgRef, MediaType, MetaItem, WriteConfig,
};

pub use genre::*;

#[rustfmt::skip]
mod generated;
mod genre;
mod tuple;

/// User defined MPEG-4 audio metadata that can be modified.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Userdata {
    // TODO: maybe use indexmap for metaitems?
    pub(crate) metaitems: Vec<MetaItem>,
    pub(crate) chapter_list: Vec<Chapter>,
    pub(crate) chapter_track: Vec<Chapter>,
}

impl Userdata {
    /// Attempts to write the MPEG-4 audio tag to the writer.
    pub fn write_with(&self, file: &File, cfg: &WriteConfig) -> crate::Result<()> {
        atom::write_tag(file, cfg, self)
    }

    /// Attempts to write the MPEG-4 audio tag to the writer. This will overwrite any metadata
    /// previously present on the file.
    pub fn write_to(&self, file: &File) -> crate::Result<()> {
        self.write_with(file, &WriteConfig::DEFAULT)
    }

    /// Attempts to write the MPEG-4 audio tag to the path.
    pub fn write_with_path(&self, path: impl AsRef<Path>, cfg: &WriteConfig) -> crate::Result<()> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        self.write_with(&file, cfg)
    }

    /// Attempts to write the MPEG-4 audio tag to the path. This will overwrite any metadata
    /// previously present on the file.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        self.write_with_path(path, &WriteConfig::DEFAULT)
    }
}

// ## Custom values
/// ### Artwork
impl Userdata {
    /// Returns all artwork images (`covr`).
    pub fn artworks(&self) -> impl Iterator<Item = ImgRef<'_>> {
        self.images_of(&ident::ARTWORK)
    }

    /// Returns the first artwork image (`covr`).
    pub fn artwork(&self) -> Option<ImgRef<'_>> {
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
    pub fn set_artwork(&mut self, image: Img<impl Into<Vec<u8>>>) {
        self.set_data(ident::ARTWORK, image.into());
    }

    /// Sets all artwork image data (`covr`). This will remove all other artworks.
    pub fn set_artworks(&mut self, images: impl IntoIterator<Item = ImgBuf>) {
        self.set_all_data(ident::ARTWORK, images.into_iter().map(Img::into));
    }

    /// Adds artwork image data (`covr`).
    pub fn add_artwork(&mut self, image: Img<impl Into<Vec<u8>>>) {
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
    pub(crate) fn format_artworks(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_artwork(f: &mut fmt::Formatter<'_>, i: ImgRef<'_>) -> fmt::Result {
            match i.fmt {
                ImgFmt::Png => write!(f, "png")?,
                ImgFmt::Jpeg => write!(f, "jpeg")?,
                ImgFmt::Bmp => write!(f, "bmp")?,
            };

            let len = i.data.len();

            if len < 1024 {
                writeln!(f, " {len}")?;
            } else if len < 1024 * 1024 {
                let size = len / 1024;
                writeln!(f, " {size}k")?;
            } else {
                let size = len / (1024 * 1024);
                writeln!(f, " {size}M")?;
            }
            Ok(())
        }

        if self.artworks().count() > 1 {
            writeln!(f, "artworks:")?;
            for a in self.artworks() {
                write!(f, "    ")?;
                format_artwork(f, a)?;
            }
        } else if let Some(a) = self.artwork() {
            write!(f, "artwork: ")?;
            format_artwork(f, a)?;
        }
        Ok(())
    }
}

/// ### Media type
impl Userdata {
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
        self.set_data(ident::MEDIA_TYPE, Data::Reserved(vec![media_type.code()]));
    }

    /// Removes the media type (`stik`).
    pub fn remove_media_type(&mut self) {
        self.remove_data_of(&ident::MEDIA_TYPE);
    }

    pub(crate) fn format_media_type(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.media_type() {
            Some(m) => writeln!(f, "media type: {m}"),
            None => Ok(()),
        }
    }
}

/// ### Advisory rating
impl Userdata {
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
        self.set_data(ident::ADVISORY_RATING, Data::Reserved(vec![rating.code()]));
    }

    /// Removes the advisory rating (`rtng`).
    pub fn remove_advisory_rating(&mut self) {
        self.remove_data_of(&ident::ADVISORY_RATING);
    }

    pub(crate) fn format_advisory_rating(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.advisory_rating() {
            Some(r) => writeln!(f, "advisory rating: {r}"),
            None => Ok(()),
        }
    }
}

/// ### Chapters
impl Userdata {
    pub fn chapters(&self) -> &[Chapter] {
        if !self.chapter_list.is_empty() {
            self.chapter_list()
        } else {
            self.chapter_track()
        }
    }

    pub fn chapters_mut(&mut self) -> &mut Vec<Chapter> {
        todo!()
    }

    pub fn take_chapters(&mut self) -> Vec<Chapter> {
        todo!()
    }
}

/// ### Chapter list
impl Userdata {
    pub fn chapter_list(&self) -> &[Chapter] {
        &self.chapter_list
    }

    pub fn chapter_list_mut(&mut self) -> &mut Vec<Chapter> {
        &mut self.chapter_list
    }

    pub fn take_chapter_list(&mut self) -> Vec<Chapter> {
        std::mem::take(&mut self.chapter_list)
    }
}

/// ### Chapter track
impl Userdata {
    pub fn chapter_track(&self) -> &[Chapter] {
        &self.chapter_track
    }

    pub fn chapter_track_mut(&mut self) -> &mut Vec<Chapter> {
        &mut self.chapter_track
    }

    pub fn take_chapter_track(&mut self) -> Vec<Chapter> {
        std::mem::take(&mut self.chapter_track)
    }
}

/// ## Data accessors
impl Userdata {
    /// Returns references to all byte data corresponding to the identifier.\
    /// See [`Data::bytes`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::BeSigned(b"data".to_vec()));
    /// assert_eq!(tag.bytes_of(&test).next().unwrap(), b"data");
    /// ```
    pub fn bytes_of<'a>(&'a self, ident: &impl Ident) -> impl Iterator<Item = &'a [u8]> {
        self.data_of(ident).filter_map(Data::bytes)
    }

    /// Returns mutable references to all byte data corresponding to the identifier.\
    /// See [`Data::bytes_mut`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.bytes_mut_of(&test).next().unwrap().push('1' as u8);
    /// assert_eq!(tag.bytes_of(&test).next().unwrap(), b"data1");
    /// ```
    pub fn bytes_mut_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = &mut Vec<u8>> {
        self.data_mut_of(ident).filter_map(Data::bytes_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's byte data.\
    /// See [`Data::into_bytes`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Reserved(b"data".to_vec()));
    /// assert_eq!(tag.take_bytes_of(&test).next().unwrap(), b"data");
    /// assert_eq!(tag.bytes_of(&test).next(), None);
    /// ```
    pub fn take_bytes_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = Vec<u8>> {
        self.take_data_of(ident).filter_map(Data::into_bytes)
    }

    /// Returns references to all strings corresponding to the identifier.\
    /// See [`Data::string`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data");
    /// ```
    pub fn strings_of<'a>(&'a self, ident: &impl Ident) -> impl Iterator<Item = &'a str> {
        self.data_of(ident).filter_map(Data::string)
    }

    /// Returns all mutable string references corresponding to the identifier.\
    /// See [`Data::string_mut`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("string".into()));
    /// tag.strings_mut_of(&test).next().unwrap().push('1');
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "string1");
    /// ```
    pub fn strings_mut_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = &mut String> {
        self.data_mut_of(ident).filter_map(Data::string_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's strings.\
    /// See [`Data::into_string`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("string".into()));
    /// assert_eq!(tag.take_strings_of(&test).next().unwrap(), "string");
    /// assert_eq!(tag.strings_of(&test).next(), None);
    /// ```
    pub fn take_strings_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = String> {
        self.take_data_of(ident).filter_map(Data::into_string)
    }

    /// Returns references to all images corresponding to the identifier.\
    /// See [`Data::image`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"image".to_vec()));
    /// let img = tag.images_of(&test).next().unwrap();
    /// assert_eq!(img.data, b"image");
    /// ```
    pub fn images_of<'a>(&'a self, ident: &impl Ident) -> impl Iterator<Item = ImgRef<'a>> {
        self.data_of(ident).filter_map(Data::image)
    }

    /// Returns mutable references to all images corresponding to the identifier.\
    /// See [`Data::image_mut`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Jpeg(b"image".to_vec()));
    /// let img = tag.images_mut_of(&test).next().unwrap();
    /// img.data.push('1' as u8);
    ///
    /// let img = tag.images_of(&test).next().unwrap();
    /// assert_eq!(img.data, b"image1");
    /// ```
    pub fn images_mut_of<'a>(&'a mut self, ident: &impl Ident) -> impl Iterator<Item = ImgMut<'a>> {
        self.data_mut_of(ident).filter_map(Data::image_mut)
    }

    /// Removes the atom corresponding to the identifier and returns all of it's images.\
    /// See [`Data::into_image`].
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc, Img};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Png(b"image".to_vec()));
    /// assert_eq!(tag.take_images_of(&test).next().unwrap(), Img::png(b"image".to_vec()));
    /// assert_eq!(tag.images_of(&test).next(), None);
    /// ```
    pub fn take_images_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = ImgBuf> {
        self.take_data_of(ident).filter_map(Data::into_image)
    }

    /// Returns references to all data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.data_of(&test).next().unwrap().string(), Some("data"));
    /// ```
    pub fn data_of<'a>(&'a self, ident: &impl Ident) -> impl Iterator<Item = &'a Data> {
        match self.metaitems.iter().find(|a| ident == &a.ident) {
            Some(a) => a.data.iter(),
            None => [].iter(),
        }
    }

    /// Returns mutable references to all data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// let data = tag.data_mut_of(&test).next().unwrap();
    /// data.string_mut().unwrap().push('1');
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data1");
    /// ```
    pub fn data_mut_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = &mut Data> {
        match self.metaitems.iter_mut().find(|a| ident == &a.ident) {
            Some(a) => a.data.iter_mut(),
            None => [].iter_mut(),
        }
    }

    /// Removes the atom corresponding to the identifier and returns all of it's data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.take_data_of(&test).next().unwrap(), Data::Utf8("data".into()));
    /// assert_eq!(tag.data_of(&test).next(), None);
    /// ```
    pub fn take_data_of(&mut self, ident: &impl Ident) -> impl Iterator<Item = Data> {
        let mut i = 0;
        while i < self.metaitems.len() {
            if ident == &self.metaitems[i].ident {
                let removed = self.metaitems.remove(i);
                return removed.data.into_iter();
            }

            i += 1;
        }

        Vec::new().into_iter()
    }

    /// Returns an iterator over references to all byte data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data1".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::BeSigned(b"data2".to_vec()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut bytes = tag.bytes();
    /// assert_eq!(bytes.next().unwrap(), (&test, &b"data1"[..]));
    /// assert_eq!(bytes.next().unwrap(), (&test, &b"data2"[..]));
    /// assert_eq!(bytes.next(), None);
    /// ```
    pub fn bytes(&self) -> impl Iterator<Item = (&DataIdent, &[u8])> {
        self.data().filter_map(|(i, d)| Some((i, d.bytes()?)))
    }

    /// Returns an iterator over mutable references to all byte data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let (ident, data) = tag.bytes_mut().next().unwrap();
    /// data.push('1' as u8);
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut bytes = tag.bytes();
    /// assert_eq!(bytes.next().unwrap(), (&test, &b"data1"[..]));
    /// assert_eq!(bytes.next(), None);
    /// ```
    pub fn bytes_mut(&mut self) -> impl Iterator<Item = (&DataIdent, &mut Vec<u8>)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.bytes_mut()?)))
    }

    /// Consumes `self` and returns an iterator over all byte data.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let (ident, data) = tag.take_bytes().next().unwrap();
    /// assert_eq!(ident.as_ref(), &test);
    /// assert_eq!(data, b"data".to_vec());
    /// ```
    pub fn take_bytes(self) -> impl Iterator<Item = (Rc<DataIdent>, Vec<u8>)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_bytes()?)))
    }

    /// Returns an iterator over references to all strings.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string1".into()));
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf16("string2".into()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut strings = tag.strings();
    /// assert_eq!(strings.next().unwrap(), (&test, "string1"));
    /// assert_eq!(strings.next().unwrap(), (&test, "string2"));
    /// assert_eq!(strings.next(), None);
    /// ```
    pub fn strings(&self) -> impl Iterator<Item = (&DataIdent, &str)> {
        self.data().filter_map(|(i, d)| Some((i, d.string()?)))
    }

    /// Returns an iterator over mutable references to all strings.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let (ident, data) = tag.strings_mut().next().unwrap();
    /// data.push('1');
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut strings = tag.strings();
    /// assert_eq!(strings.next().unwrap(), (&test, "string1"));
    /// assert_eq!(strings.next(), None);
    /// ```
    pub fn strings_mut(&mut self) -> impl Iterator<Item = (&DataIdent, &mut String)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.string_mut()?)))
    }

    /// Consumes `self` and returns an iterator over all strings.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let (ident, data) = tag.take_strings().next().unwrap();
    /// assert_eq!(ident.as_ref(), &test);
    /// assert_eq!(data, "string".to_string());
    /// ```
    pub fn take_strings(self) -> impl Iterator<Item = (Rc<DataIdent>, String)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_string()?)))
    }

    /// Returns an iterator over references to all images.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc, Img};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Png(b"image1".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::Jpeg(b"image2".to_vec()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut images = tag.images();
    /// assert_eq!(images.next().unwrap(), (&test, Img::png(&b"image1"[..])));
    /// assert_eq!(images.next().unwrap(), (&test, Img::jpeg(&b"image2"[..])));
    /// assert_eq!(images.next(), None);
    /// ```
    pub fn images(&self) -> impl Iterator<Item = (&DataIdent, ImgRef<'_>)> {
        self.data().filter_map(|(i, d)| Some((i, d.image()?)))
    }

    /// Returns an iterator over mutable references to all images.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc, Img};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Bmp(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let (ident, image) = tag.images_mut().next().unwrap();
    /// image.data.push('1' as u8);
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut images = tag.images();
    /// assert_eq!(images.next().unwrap(), (&test, Img::bmp(&b"data1"[..])));
    /// assert_eq!(images.next(), None);
    /// ```
    pub fn images_mut(&mut self) -> impl Iterator<Item = (&DataIdent, ImgMut<'_>)> {
        self.data_mut().filter_map(|(i, d)| Some((i, d.image_mut()?)))
    }

    /// Consumes `self` and returns an iterator over all images.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc, Img};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Jpeg(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let (ident, image) = tag.take_images().next().unwrap();
    /// assert_eq!(ident.as_ref(), &test);
    /// assert_eq!(image, Img::jpeg(b"data".to_vec()));
    /// ```
    pub fn take_images(self) -> impl Iterator<Item = (Rc<DataIdent>, ImgBuf)> {
        self.take_data().filter_map(|(i, d)| Some((i, d.into_image()?)))
    }

    /// Returns an iterator over references to all data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Reserved(b"data".to_vec()));
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::Png(b"image".to_vec()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut data = tag.data();
    /// assert_eq!(data.next().unwrap(), (&test, &Data::Reserved(b"data".to_vec())));
    /// assert_eq!(data.next().unwrap(), (&test, &Data::Utf8("string".into())));
    /// assert_eq!(data.next().unwrap(), (&test, &Data::Png(b"image".to_vec())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn data(&self) -> impl Iterator<Item = (&DataIdent, &Data)> {
        self.metaitems.iter().flat_map(|a| a.data.iter().map(move |d| (&a.ident, d)))
    }

    /// Returns an iterator over mutable references to all data.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string".into()));
    ///
    /// let (ident, data) = tag.data_mut().next().unwrap();
    /// data.string_mut().unwrap().push('1');
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let mut strings = tag.strings();
    /// assert_eq!(strings.next().unwrap(), (&test, "string1"));
    /// assert_eq!(strings.next(), None);
    /// ```
    pub fn data_mut(&mut self) -> impl Iterator<Item = (&DataIdent, &mut Data)> {
        self.metaitems.iter_mut().flat_map(|a| {
            let ident = &a.ident;
            let data = &mut a.data;
            data.iter_mut().map(move |d| (ident, d))
        })
    }

    /// Consumes `self` and returns an iterator over all data.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use mp4ameta::{Userdata, Data, DataIdent, Fourcc, Img};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Jpeg(b"data".to_vec()));
    ///
    /// let test = DataIdent::Fourcc(test);
    /// let (ident, image) = tag.take_data().next().unwrap();
    /// assert_eq!(ident.as_ref(), &test);
    /// assert_eq!(image, Data::Jpeg(b"data".to_vec()));
    /// ```
    pub fn take_data(self) -> impl Iterator<Item = (Rc<DataIdent>, Data)> {
        self.metaitems.into_iter().flat_map(move |a| {
            let ident = Rc::new(a.ident);
            let data = a.data;
            data.into_iter().map(move |d| (ident.clone(), d))
        })
    }

    /// Removes all byte data corresponding to the identifier. Other data will remain unaffected.\
    /// See [`Data::is_bytes`]
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::BeSigned("data".into()));
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("string".into())));
    /// assert_eq!(data.next(), Some(&Data::BeSigned(b"data".to_vec())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.remove_bytes_of(&test);
    ///
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("string".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn remove_bytes_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_bytes());
    }

    /// Removes all strings corresponding to the identifier. Other data will remain unaffected.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::Bmp("image".into()));
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("string".into())));
    /// assert_eq!(data.next(), Some(&Data::Bmp(b"image".to_vec())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.remove_strings_of(&test);
    ///
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Bmp(b"image".to_vec())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn remove_strings_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_string());
    }

    /// Removes all images corresponding to the identifier. Other data will remain unaffected.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("string".into()));
    /// tag.add_data(test, Data::Bmp("image".into()));
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("string".into())));
    /// assert_eq!(data.next(), Some(&Data::Bmp(b"image".to_vec())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.remove_images_of(&test);
    ///
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("string".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn remove_images_of(&mut self, ident: &impl Ident) {
        self.retain_data_of(ident, |d| !d.is_image());
    }

    /// Removes all data corresponding to the identifier.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(tag.data_of(&test).next().is_some());
    /// tag.remove_data_of(&test);
    /// assert!(tag.data_of(&test).next().is_none());
    /// ```
    pub fn remove_data_of(&mut self, ident: &impl Ident) {
        self.metaitems.retain(|a| ident != &a.ident);
    }

    /// Retains only the bytes, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::BeSigned(vec![4; 12]));
    /// tag.add_data(test, Data::Reserved(vec![6; 16]));
    ///
    /// let mut bytes = tag.bytes_of(&test);
    /// assert_eq!(bytes.next(), Some(&[4; 12][..]));
    /// assert_eq!(bytes.next(), Some(&[6; 16][..]));
    /// assert_eq!(bytes.next(), None);
    /// drop(bytes);
    ///
    /// tag.retain_bytes_of(&test, |b| b[2..4] == [4, 4]);
    ///
    /// let mut bytes = tag.bytes_of(&test);
    /// assert_eq!(bytes.next(), Some(&[4; 12][..]));
    /// assert_eq!(bytes.next(), None);
    /// ```
    pub fn retain_bytes_of(&mut self, ident: &impl Ident, predicate: impl Fn(&[u8]) -> bool) {
        #[allow(clippy::redundant_closure)]
        self.retain_data_of(ident, |d| d.bytes().map_or(true, |b| predicate(b)));
    }

    /// Retains only the strings, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc, Img, ImgFmt};
    ///
    /// let mut tag = Userdata::default();
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
        #[allow(clippy::redundant_closure)]
        self.retain_data_of(ident, |d| d.string().map_or(true, |s| predicate(s)));
    }

    /// Retains only the images, of the atom corresponding to the identifier, that match the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc, Img, ImgFmt};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Png(vec![5; 4]));
    /// tag.add_data(test, Data::Jpeg(vec![6; 16]));
    ///
    /// let mut images = tag.images_of(&test);
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Png, &[5; 4][..])));
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Jpeg, &[6; 16][..])));
    /// assert_eq!(images.next(), None);
    /// drop(images);
    ///
    /// tag.retain_images_of(&test, |d| d.fmt == ImgFmt::Jpeg);
    ///
    /// let mut images = tag.images_of(&test);
    /// assert_eq!(images.next(), Some(Img::new(ImgFmt::Jpeg, &[6; 16][..])));
    /// assert_eq!(images.next(), None);
    /// ```
    pub fn retain_images_of(&mut self, ident: &impl Ident, predicate: impl Fn(ImgRef<'_>) -> bool) {
        #[allow(clippy::redundant_closure)]
        self.retain_data_of(ident, |d| d.image().map_or(true, |i| predicate(i)));
    }

    /// Retains only the data, of the atom corresponding to the identifier, that matches the
    /// predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.add_data(test, Data::Utf8("short".into()));
    /// tag.add_data(test, Data::Reserved(vec![6; 16]));
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("short".into())));
    /// assert_eq!(data.next(), Some(&Data::Reserved(vec![6; 16])));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_data_of(&test, |d| d.data_len() < 10);
    ///
    /// let mut data = tag.data_of(&test);
    /// assert_eq!(data.next(), Some(&Data::Utf8("short".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_data_of(&mut self, ident: &impl Ident, predicate: impl Fn(&Data) -> bool) {
        let pos = self.metaitems.iter().position(|a| ident == &a.ident);

        if let Some(i) = pos {
            self.metaitems[i].data.retain(predicate);
            if self.metaitems[i].data.is_empty() {
                self.metaitems.remove(i);
            }
        }
    }

    /// Retains only the byte data matching the predicate. Other data will remain unaffected.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let tst1 = Fourcc(*b"tst1");
    /// let tst2 = Fourcc(*b"tst2");
    ///
    /// tag.add_data(tst1, Data::Reserved(b"data1".to_vec()));
    /// tag.add_data(tst2, Data::Png(b"data2".to_vec()));
    /// tag.add_data(tst2, Data::BeSigned(b"data3".to_vec()));
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Reserved(b"data1".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Png(b"data2".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::BeSigned(b"data3".to_vec())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_bytes(|i, d| &tst1 == i);
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Reserved(b"data1".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Png(b"data2".to_vec())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_bytes(&mut self, predicate: impl Fn(&DataIdent, &[u8]) -> bool) {
        self.retain_data(|i, d| d.bytes().map_or(true, |s| predicate(i, s)));
    }

    /// Retains only the strings matching the predicate. Other data will remain unaffected.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let tst1 = Fourcc(*b"tst1");
    /// let tst2 = Fourcc(*b"tst2");
    ///
    /// tag.add_data(tst1, Data::Utf8("data1".into()));
    /// tag.add_data(tst2, Data::Png(b"data2".to_vec()));
    /// tag.add_data(tst2, Data::Utf8("data3".into()));
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data1".into())));
    /// assert_eq!(data.next(), Some(&Data::Png(b"data2".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data3".into())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_strings(|i, d| &tst1 == i);
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data1".into())));
    /// assert_eq!(data.next(), Some(&Data::Png(b"data2".to_vec())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_strings(&mut self, predicate: impl Fn(&DataIdent, &str) -> bool) {
        self.retain_data(|i, d| d.string().map_or(true, |s| predicate(i, s)));
    }

    /// Retains only the images matching the predicate. Other data will remain unaffected.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let tst1 = Fourcc(*b"tst1");
    /// let tst2 = Fourcc(*b"tst2");
    ///
    /// tag.add_data(tst1, Data::Jpeg(b"data1".to_vec()));
    /// tag.add_data(tst2, Data::Png(b"data2".to_vec()));
    /// tag.add_data(tst2, Data::Utf8("data3".into()));
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Jpeg(b"data1".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Png(b"data2".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data3".into())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_images(|i, d| &tst1 == i);
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Jpeg(b"data1".to_vec())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data3".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_images(&mut self, predicate: impl Fn(&DataIdent, ImgRef<'_>) -> bool) {
        self.retain_data(|i, d| d.image().map_or(true, |s| predicate(i, s)));
    }

    /// Retains only the data matching the predicate.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let tst1 = Fourcc(*b"tst1");
    /// let tst2 = Fourcc(*b"tst2");
    ///
    /// tag.add_data(tst1, Data::Utf8("data1".into()));
    /// tag.add_data(tst2, Data::Utf8("data2".into()));
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data1".into())));
    /// assert_eq!(data.next(), Some(&Data::Utf8("data2".into())));
    /// assert_eq!(data.next(), None);
    /// drop(data);
    ///
    /// tag.retain_data(|i, d| &tst1 != i);
    ///
    /// let mut data = tag.data().map(|(i, d)| d);
    /// assert_eq!(data.next(), Some(&Data::Utf8("data2".into())));
    /// assert_eq!(data.next(), None);
    /// ```
    pub fn retain_data(&mut self, predicate: impl Fn(&DataIdent, &Data) -> bool) {
        let mut i = 0;
        while i < self.metaitems.len() {
            let a = &mut self.metaitems[i];
            let mut j = 0;
            while j < a.data.len() {
                if predicate(&a.ident, &a.data[j]) {
                    j += 1;
                } else {
                    a.data.remove(j);
                }
            }

            if a.data.is_empty() {
                self.metaitems.remove(i);
            } else {
                i += 1;
            }
        }
    }

    // TODO: clear chapters too
    // TODO: also add clear_meta and clear_chapters?
    /// Removes all metadata atoms of the tag.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// assert!(tag.is_empty());
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(!tag.is_empty());
    /// tag.clear();
    /// assert!(tag.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.metaitems.clear();
    }

    /// If an atom corresponding to the identifier exists, it's data will be replaced by the new
    /// data, otherwise a new metadata item atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert_eq!(tag.strings_of(&test).next().unwrap(), "data");
    /// ```
    pub fn set_data(&mut self, ident: (impl Ident + Into<DataIdent>), data: Data) {
        match self.metaitems.iter_mut().find(|a| ident == a.ident) {
            Some(a) => {
                a.data.clear();
                a.data.push(data);
            }
            None => self.metaitems.push(MetaItem::new(ident.into(), vec![data])),
        }
    }

    /// If an atom corresponding to the identifier exists, it's data will be replaced by the new
    /// data, otherwise a new metadata item atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
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
        match self.metaitems.iter_mut().find(|a| ident == a.ident) {
            Some(a) => {
                a.data.clear();
                a.data.extend(data);
            }
            None => {
                self.metaitems.push(MetaItem::new(ident.into(), data.into_iter().collect()));
            }
        }
    }

    /// If an atom corresponding to the identifier exists, the new data will be added to it,
    /// otherwise a new atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
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
        match self.metaitems.iter_mut().find(|a| ident == a.ident) {
            Some(a) => a.data.push(data),
            None => self.metaitems.push(MetaItem::new(ident.into(), vec![data])),
        }
    }

    /// If an atom corresponding to the identifier exists, the new data will be added to it,
    /// otherwise a new metadata item atom containing the data will be created.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
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
        match self.metaitems.iter_mut().find(|a| ident == a.ident) {
            Some(a) => a.data.extend(data),
            None => self.metaitems.push(MetaItem::new(ident.into(), data.into_iter().collect())),
        }
    }

    /// Returns true if this tag contains any metadata atoms, false otherwise.
    ///
    /// # Example
    /// ```
    /// use mp4ameta::{Userdata, Data, Fourcc};
    ///
    /// let mut tag = Userdata::default();
    /// let test = Fourcc(*b"test");
    ///
    /// assert!(tag.is_empty());
    /// tag.set_data(test, Data::Utf8("data".into()));
    /// assert!(!tag.is_empty());
    /// tag.clear();
    /// assert!(tag.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.metaitems.is_empty()
    }
}
