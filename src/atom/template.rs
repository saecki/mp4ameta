use super::*;

use lazy_static::lazy_static;

lazy_static! {
    /// Lazily initialized static reference of a filetype (`ftyp`) atom template.
    pub(crate) static ref FILETYPE_ATOM_T: AtomT = filetype_atom_t();
    /// Lazily initialized static reference of an atom template hierarchy needed to write metadata.
    pub(crate) static ref METADATA_WRITE_ATOM_T: [AtomT; 2] = metadata_write_atom_t();
    /// Lazily initialized static reference of an atom template hierarchy needed to read metadata.
    pub(crate) static ref METADATA_READ_ATOM_T: AtomT = metadata_read_atom_t();
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
                    AtomT::new(MEDIA_INFORMATION, 0, ContentT::atom_t(
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
        AtomT::new(MOVIE_HEADER, 0, ContentT::MovieHeader),
        AtomT::new(TRACK, 0, ContentT::atom_t(
            AtomT::new(MEDIA, 0, ContentT::atom_t(
                AtomT::new(MEDIA_INFORMATION, 0, ContentT::atom_t(
                    AtomT::new(SAMPLE_TABLE, 0, ContentT::atom_t(
                        AtomT::new(SAMPLE_TABLE_SAMPLE_DESCRIPTION, 8, ContentT::atom_t(
                            AtomT::new(MP4_AUDIO, 0, ContentT::Mp4Audio)
                        )),
                    ))
                )),
            )),
        )),
        AtomT::new(USER_DATA, 0, ContentT::atom_t(
            AtomT::new(METADATA, 4, ContentT::atom_t(
                AtomT::new(ITEM_LIST, 0, ContentT::Atoms(vec![
                    AtomT::new(FREE, 0, ContentT::Ignore),
                    AtomT::new(WILDCARD, 0, ContentT::Atoms(vec![
                        AtomT::data_atom(),
                        AtomT::mean_atom(),
                        AtomT::name_atom(),
                    ])),
                ])),
            )),
        )),
    ]))
}
