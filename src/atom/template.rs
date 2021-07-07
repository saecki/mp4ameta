use super::*;

use lazy_static::lazy_static;

lazy_static! {
    /// Lazily initialized static reference to an atom template hierarchy needed to write metadata.
    pub(super) static ref METADATA_WRITE_ATOM_T: [AtomT; 2] = metadata_write_atom_t();
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
                        AtomT::new(SAMPLE_TABLE, 0, ContentT::Atoms(vec![
                            AtomT::new(SAMPLE_TABLE_CHUNK_OFFSET, 0, ContentT::RawData(data::RESERVED)),
                            AtomT::new(SAMPLE_TABLE_CHUNK_OFFSET_64, 0, ContentT::RawData(data::RESERVED)),
                        ]))
                    ))
                ))
            )),
            AtomT::new(USER_DATA, 0, ContentT::atom_t(
                AtomT::new(METADATA, 4, ContentT::Atoms(vec![
                    AtomT::new(HANDLER_REFERENCE, 0, ContentT::RawData(data::RESERVED)),
                    AtomT::new(ITEM_LIST, 0, ContentT::atoms_t()),
                ]))
            ))
        ])),
    ]
}

#[rustfmt::skip]
pub(super) fn meta_handler_reference_atom() -> Atom<'static> {
    Atom::new(HANDLER_REFERENCE, 0, Content::RawData(
        Data::Reserved(vec![
            0x00, 0x00, 0x00, 0x00, // version + flags
            0x00, 0x00, 0x00, 0x00, // component type
            0x6d, 0x64, 0x69, 0x72, // component subtype
            0x61, 0x70, 0x70, 0x6c, // component manufacturer
            0x00, 0x00, 0x00, 0x00, // component flags
            0x00, 0x00, 0x00, 0x00, // component flags mask
            0x00,                   // component name
        ])
    ))
}
