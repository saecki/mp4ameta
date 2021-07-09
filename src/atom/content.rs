use super::*;

/// A template representing the different types of content an atom template might have.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ContentT {
    /// A value containing a list of children atom templates.
    Atoms(Vec<AtomT>),
    /// A template for raw data containing a datatype definded by
    /// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
    RawData(u32),
    /// A template representing typed data that is defined by a
    /// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
    /// code prior to the data parsed.
    Empty,
}

impl Default for ContentT {
    fn default() -> Self {
        Self::Empty
    }
}

impl ContentT {
    /// Creates a new empty content template of type [`Self::Atoms`].
    pub const fn atoms_t() -> Self {
        Self::Atoms(Vec::new())
    }

    /// Creates a new content template of type [`Self::Atoms`] containing the atom template.
    pub fn atom_t(atom: AtomT) -> Self {
        Self::Atoms(vec![atom])
    }
}
