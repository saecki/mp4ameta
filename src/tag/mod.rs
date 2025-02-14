use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::{atom, util, AudioInfo, ReadConfig};

pub use userdata::*;

mod readonly;
mod userdata;

/// A MPEG-4 audio tag containing metadata atoms
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    /// The `ftyp` atom.
    pub(crate) ftyp: String,
    /// Readonly audio information.
    pub(crate) info: AudioInfo,
    /// User data.
    pub(crate) userdata: Userdata,
}

impl Deref for Tag {
    type Target = Userdata;

    fn deref(&self) -> &Self::Target {
        &self.userdata
    }
}

impl DerefMut for Tag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.userdata
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_album_artists(f)?;
        self.format_artists(f)?;
        self.format_composers(f)?;
        self.format_lyricists(f)?;
        self.format_album(f)?;
        self.format_title(f)?;
        self.format_genres(f)?;
        self.format_year(f)?;
        self.format_track(f)?;
        self.format_disc(f)?;
        self.format_artworks(f)?;
        self.format_advisory_rating(f)?;
        self.format_media_type(f)?;
        self.format_groupings(f)?;
        self.format_descriptions(f)?;
        self.format_comments(f)?;
        self.format_categories(f)?;
        self.format_keywords(f)?;
        self.format_copyright(f)?;
        self.format_encoder(f)?;
        self.format_tv_show_name(f)?;
        self.format_tv_network_name(f)?;
        self.format_tv_episode_name(f)?;
        self.format_tv_episode(f)?;
        self.format_tv_season(f)?;
        self.format_bpm(f)?;
        self.format_movement(f)?;
        self.format_work(f)?;
        self.format_movement_count(f)?;
        self.format_movement_index(f)?;
        self.format_duration(f)?;
        self.format_channel_config(f)?;
        self.format_sample_rate(f)?;
        self.format_avg_bitrate(f)?;
        self.format_max_bitrate(f)?;
        self.format_show_movement(f)?;
        self.format_gapless_playback(f)?;
        self.format_compilation(f)?;
        self.format_isrc(f)?;
        self.format_lyrics(f)?;
        self.format_chapter_list(f)?;
        self.format_chapter_track(f)?;
        writeln!(f, "filetype: {}", self.filetype())
    }
}

impl Tag {
    fn format_chapter_list(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.userdata.chapter_list.is_empty() {
            writeln!(f, "chapter list:")?;
            util::format_chapters(f, &self.chapter_list, self.info.duration)?;
        }
        Ok(())
    }

    fn format_chapter_track(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.userdata.chapter_track.is_empty() {
            writeln!(f, "chapter track:")?;
            util::format_chapters(f, &self.chapter_track, self.info.duration)?;
        }
        Ok(())
    }
}

impl Tag {
    /// Attempts to read a MPEG-4 audio tag from the reader.
    pub fn read_with(reader: &mut (impl Read + Seek), cfg: &ReadConfig) -> crate::Result<Self> {
        atom::read_tag(reader, cfg)
    }

    /// Attempts to read a MPEG-4 audio tag from the reader.
    pub fn read_from(reader: &mut (impl Read + Seek)) -> crate::Result<Self> {
        Self::read_with(reader, &ReadConfig::DEFAULT)
    }

    /// Attempts to read a MPEG-4 audio tag from the file at the indicated path.
    pub fn read_with_path(path: impl AsRef<Path>, cfg: &ReadConfig) -> crate::Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        Self::read_with(&mut file, cfg)
    }

    /// Attempts to read a MPEG-4 audio tag from the file at the indicated path.
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        Self::read_with_path(path, &ReadConfig::DEFAULT)
    }
}
