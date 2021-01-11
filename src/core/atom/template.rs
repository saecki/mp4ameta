use super::*;

lazy_static! {
    /// Lazily initialized static reference of a filetype (`ftyp`) atom template.
    pub static ref FILETYPE_ATOM_T: AtomT = filetype_atom_t();
    /// Lazily initialized static reference of an atom template hierarchy needed to write metadata.
    pub static ref METADATA_WRITE_ATOM_T: [AtomT; 2] = metadata_write_atom_t();
    /// Lazily initialized static reference of an atom template hierarchy needed to read metadata.
    pub static ref METADATA_READ_ATOM_T: AtomT = metadata_read_atom_t();
}

/// Returns an file type (`ftyp`) atom template.
fn filetype_atom_t() -> AtomT {
    AtomT::new(FILETYPE, 0, ContentT::RawData(data::UTF8))
}

/// Returns an atom template hierarchy needed to write metadata.
#[rustfmt::skip]
fn metadata_write_atom_t() -> [AtomT; 2] {
    [
        AtomT::new(MEDIA_DATA, 0, ContentT::RawData(data::RESERVED)),
        AtomT::new(MOVIE, 0, ContentT::Atoms(vec![
            AtomT::new(TRACK, 0, ContentT::atom_t(
                AtomT::new(MEDIA, 0, ContentT::atom_t(
                    AtomT::new(METADATA_INFORMATION, 0, ContentT::atom_t(
                        AtomT::new(SAMPLE_TABLE, 0, ContentT::atom_t(
                            AtomT::new(SAMPLE_TABLE_CHUNK_OFFSET, 0, ContentT::RawData(data::RESERVED))
                        ))
                    )),
                )),
            )),
            AtomT::new(USER_DATA, 0, ContentT::atom_t(
                AtomT::new(METADATA, 4, ContentT::atom_t(
                    AtomT::new(ITEM_LIST, 0, ContentT::atoms_t())
                ))
            ))
        ])),
    ]
}

/// Returns an atom template hierarchy needed to read metadata.
#[rustfmt::skip]
fn metadata_read_atom_t() -> AtomT {
    AtomT::new(MOVIE, 0, ContentT::Atoms(vec![
        AtomT::new(MOVIE_HEADER, 0, ContentT::RawData(data::RESERVED)),
        AtomT::new(USER_DATA, 0, ContentT::atom_t(
            AtomT::new(METADATA, 4, ContentT::atom_t(
                AtomT::new(ITEM_LIST, 0, ContentT::Atoms(vec![
                    AtomT::new(FREEFORM, 0, ContentT::Atoms(vec![
                        AtomT::data_atom(),
                        AtomT::mean_atom(),
                        AtomT::name_atom(),
                    ])),
                    AtomT::new(ADVISORY_RATING, 0, ContentT::data_atom_t()),
                    AtomT::new(ALBUM, 0, ContentT::data_atom_t()),
                    AtomT::new(ALBUM_ARTIST, 0, ContentT::data_atom_t()),
                    AtomT::new(ARTIST, 0, ContentT::data_atom_t()),
                    AtomT::new(BPM, 0, ContentT::data_atom_t()),
                    AtomT::new(CATEGORY, 0, ContentT::data_atom_t()),
                    AtomT::new(COMMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(COMPILATION, 0, ContentT::data_atom_t()),
                    AtomT::new(COMPOSER, 0, ContentT::data_atom_t()),
                    AtomT::new(COPYRIGHT, 0, ContentT::data_atom_t()),
                    AtomT::new(CUSTOM_GENRE, 0, ContentT::data_atom_t()),
                    AtomT::new(DESCRIPTION, 0, ContentT::data_atom_t()),
                    AtomT::new(DISC_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(ENCODER, 0, ContentT::data_atom_t()),
                    AtomT::new(GAPLESS_PLAYBACK, 0, ContentT::data_atom_t()),
                    AtomT::new(GROUPING, 0, ContentT::data_atom_t()),
                    AtomT::new(KEYWORD, 0, ContentT::data_atom_t()),
                    AtomT::new(LYRICS, 0, ContentT::data_atom_t()),
                    AtomT::new(MEDIA_TYPE, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT_COUNT, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT_INDEX, 0, ContentT::data_atom_t()),
                    AtomT::new(MOVEMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST_EPISODE_GLOBAL_UNIQUE_ID, 0, ContentT::data_atom_t()),
                    AtomT::new(PODCAST_URL, 0, ContentT::data_atom_t()),
                    AtomT::new(PURCHASE_DATE, 0, ContentT::data_atom_t()),
                    AtomT::new(SHOW_MOVEMENT, 0, ContentT::data_atom_t()),
                    AtomT::new(STANDARD_GENRE, 0, ContentT::data_atom_t()),
                    AtomT::new(TITLE, 0, ContentT::data_atom_t()),
                    AtomT::new(TRACK_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_EPISODE, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_EPISODE_NUMBER, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_NETWORK_NAME, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_SEASON, 0, ContentT::data_atom_t()),
                    AtomT::new(TV_SHOW_NAME, 0, ContentT::data_atom_t()),
                    AtomT::new(WORK, 0, ContentT::data_atom_t()),
                    AtomT::new(YEAR, 0, ContentT::data_atom_t()),
                    AtomT::new(ARTWORK, 0, ContentT::data_atom_t()),
                ])),
            )),
        )),
    ]))
}
