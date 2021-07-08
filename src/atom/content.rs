use std::io::Write;

use super::*;

/// An enum representing the different types of content an atom might have.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Content<'a> {
    /// A value containing a list of children atoms.
    Atoms(Vec<Atom<'a>>),
    /// A value containing a list of children atoms.
    AtomDataRef(&'a [AtomData]),
    /// A value containing raw data.
    RawData(Data),
    /// Empty content.
    Empty,
}

impl Default for Content<'_> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<'a> Content<'a> {
    /// Returns the length in bytes.
    pub fn len(&self) -> u64 {
        match self {
            Self::Atoms(v) => v.iter().map(|a| a.len()).sum(),
            Self::AtomDataRef(v) => v.iter().map(|a| a.len()).sum(),
            Self::RawData(d) => d.len(),
            Self::Empty => 0,
        }
    }

    /// Attempts to write the content to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self {
            Self::Atoms(v) => {
                for a in v {
                    a.write_to(writer)?;
                }
            }
            Self::AtomDataRef(v) => {
                for a in *v {
                    a.write(writer)?;
                }
            }
            Self::RawData(d) => d.write_raw(writer)?,
            Self::Empty => (),
        }

        Ok(())
    }
}

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
