use std::fmt;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{
    atom::{self, AudioInfo},
    data, Atom, AtomT, Data, ErrorKind, FourCC,
};

/// An enum representing the different types of content an atom might have.
#[derive(Clone, Eq, PartialEq)]
pub enum Content {
    /// A value containing a list of children atoms.
    Atoms(Vec<Atom>),
    /// A value containing raw data.
    RawData(Data),
    /// A value containing data defined by a
    /// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
    /// code.
    TypedData(Data),
    /// A value containing audio information.
    AudioInfo(AudioInfo),
    /// Empty content.
    Empty,
}

impl Default for Content {
    fn default() -> Self {
        Self::Empty
    }
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Content::Atoms(a) => write!(f, "Content::Atoms({:#?})", a),
            Content::RawData(d) => write!(f, "Content::RawData({:?})", d),
            Content::TypedData(d) => write!(f, "Content::TypedData({:?})", d),
            Content::AudioInfo(d) => write!(f, "Content::AudioInfo({:?})", d),
            Content::Empty => write!(f, "Content::Empty"),
        }
    }
}

impl IntoIterator for Content {
    type Item = Atom;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Atoms(v) => v.into_iter(),
            _ => Vec::new().into_iter(),
        }
    }
}

impl Content {
    /// Creates new empty content of type [Self::Atoms](Self::Atoms).
    pub fn atoms() -> Self {
        Self::Atoms(Vec::new())
    }

    /// Creates new content of type [Self::Atoms](Self::Atoms) containing the
    /// atom.
    pub fn atom(atom: Atom) -> Self {
        Self::Atoms(vec![atom])
    }

    /// Creates new content of type [Self::Atoms](Self::Atoms) containing a data
    /// [`Atom`](struct.Atom.html) with the data.
    pub fn data_atom_with(data: Data) -> Self {
        Self::atom(Atom::data_atom_with(data))
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        match self {
            Self::Atoms(v) => v.iter().map(|a| a.len()).sum(),
            Self::RawData(d) => d.len(),
            Self::TypedData(d) => 8 + d.len(),
            Self::AudioInfo(_) => 0,
            Self::Empty => 0,
        }
    }

    /// Returns whether the content is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Atoms(v) => v.is_empty(),
            Self::RawData(d) => d.is_empty(),
            Self::TypedData(d) => d.is_empty(),
            Self::AudioInfo(_) => true,
            Self::Empty => true,
        }
    }

    /// Returns an iterator over the children atoms.
    pub fn iter(&self) -> std::slice::Iter<Atom> {
        match self {
            Self::Atoms(v) => v.iter(),
            _ => [].iter(),
        }
    }

    /// Returns a mutable iterator over the children atoms.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Atom> {
        match self {
            Self::Atoms(v) => v.iter_mut(),
            _ => [].iter_mut(),
        }
    }

    /// Returns a reference to the first children atom matching the `identifier`, if present.
    pub fn child(&self, ident: FourCC) -> Option<&Atom> {
        self.iter().find(|a| a.ident == ident)
    }

    /// Return a reference to the first children atom, if present.
    pub fn first_child(&self) -> Option<&Atom> {
        if let Self::Atoms(v) = self {
            return v.first();
        }
        None
    }

    /// Returns a mutable reference to the first children atom matching the `identfier`, if present.
    pub fn child_mut(&mut self, ident: FourCC) -> Option<&mut Atom> {
        self.iter_mut().find(|a| a.ident == ident)
    }

    /// Returns a mutable reference to the first children atom, if present.
    pub fn first_child_mut(&mut self) -> Option<&mut Atom> {
        if let Self::Atoms(v) = self {
            return v.first_mut();
        }
        None
    }

    /// Consumes self and returns the first children atom matching the `identfier`, if present.
    pub fn take_child(self, ident: FourCC) -> Option<Atom> {
        self.into_iter().find(|a| a.ident == ident)
    }

    /// Consumes self and returns the first children atom, if present.
    pub fn take_first_child(self) -> Option<Atom> {
        self.into_iter().next()
    }

    /// Return a data reference if `self` is of type [`Content::RawData`](crate::Content::RawData)
    /// or [`Content::TypedData`](crate::Content::TypedData).
    pub fn data(&self) -> Option<&Data> {
        match self {
            Self::TypedData(d) => Some(d),
            Self::RawData(d) => Some(d),
            _ => None,
        }
    }

    /// Return a data reference if `self` is of type [`Content::RawData`](crate::Content::RawData)
    /// or [`Content::TypedData`](crate::Content::TypedData).
    pub fn data_mut(&mut self) -> Option<&mut Data> {
        match self {
            Self::TypedData(d) => Some(d),
            Self::RawData(d) => Some(d),
            _ => None,
        }
    }

    /// Replaces `self` with it's default value and returns the data, if present.
    pub fn take_data(self) -> Option<Data> {
        match self {
            Self::TypedData(d) => Some(d),
            Self::RawData(d) => Some(d),
            _ => None,
        }
    }

    /// Attempts to write the content to the `writer`.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self {
            Self::Atoms(v) => {
                for a in v {
                    a.write_to(writer)?;
                }
            }
            Self::RawData(d) => d.write_raw(writer)?,
            Self::TypedData(d) => d.write_typed(writer)?,
            Self::AudioInfo(_) => {
                return Err(crate::Error::new(crate::ErrorKind::UnwritableData, "".to_owned()))
            }
            Self::Empty => (),
        }

        Ok(())
    }
}

/// A template representing the different types of content an atom template might have.
#[derive(Clone, Eq, PartialEq)]
pub enum ContentT {
    /// A value containing a list of children atom templates.
    Atoms(Vec<AtomT>),
    /// A template for raw data containing a datatype definded by
    /// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
    RawData(u32),
    /// A template representing typed data that is defined by a
    /// [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
    /// code prior to the data parsed.
    TypedData,
    /// A template representing audio information.
    AudioInfo,
    /// A template for ignoring all data inside.
    Ignore,
    /// Empty content.
    Empty,
}

impl Default for ContentT {
    fn default() -> Self {
        Self::Empty
    }
}

impl fmt::Debug for ContentT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atoms(a) => write!(f, "ContentT::Atoms{{ {:#?} }}", a),
            Self::RawData(d) => write!(f, "ContentT::RawData{{ {:?} }}", d),
            Self::TypedData => write!(f, "ContentT::TypedData"),
            Self::AudioInfo => write!(f, "ContentT::AudioInfo"),
            Self::Ignore => write!(f, "ContentT::Ignore"),
            Self::Empty => write!(f, "ContentT::Empty"),
        }
    }
}

impl IntoIterator for ContentT {
    type Item = AtomT;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Atoms(v) => v.into_iter(),
            _ => Vec::new().into_iter(),
        }
    }
}

impl ContentT {
    /// Creates a new empty content template of type [Self::Atoms](Self::Atoms).
    pub const fn atoms_t() -> Self {
        Self::Atoms(Vec::new())
    }

    /// Creates a new content template of type [Self::Atoms](Self::Atoms)
    /// containing the `atom` template.
    pub fn atom_t(atom: AtomT) -> Self {
        Self::Atoms(vec![atom])
    }

    /// Creates a new content template of type [Self::Atoms](Self::Atoms)
    /// containing a data atom template.
    pub fn data_atom_t() -> Self {
        Self::atom_t(AtomT::data_atom())
    }

    /// Returns an iterator over the children atoms.
    pub fn iter(&self) -> std::slice::Iter<AtomT> {
        match self {
            Self::Atoms(v) => v.iter(),
            _ => [].iter(),
        }
    }

    /// Returns a mutable iterator over the children atoms.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<AtomT> {
        match self {
            Self::Atoms(v) => v.iter_mut(),
            _ => [].iter_mut(),
        }
    }

    /// Returns a reference to the first children atom matching the `identifier`, if present.
    pub fn child(&self, ident: FourCC) -> Option<&AtomT> {
        self.iter().find(|a| a.ident == ident)
    }

    /// Return a reference to the first children atom, if present.
    pub fn first_child(&self) -> Option<&AtomT> {
        if let Self::Atoms(v) = self {
            return v.first();
        }
        None
    }

    /// Returns a mutable reference to the first children atom matching the `identfier`, if present.
    pub fn child_mut(&mut self, ident: FourCC) -> Option<&mut AtomT> {
        self.iter_mut().find(|a| a.ident == ident)
    }

    /// Returns a mutable reference to the first children atom, if present.
    pub fn first_child_mut(&mut self) -> Option<&mut AtomT> {
        if let Self::Atoms(v) = self {
            return v.first_mut();
        }
        None
    }

    /// Consumes self and returns the first children atom matching the `identfier`, if present.
    pub fn take_child(self, ident: FourCC) -> Option<AtomT> {
        self.into_iter().find(|a| a.ident == ident)
    }

    /// Consumes self and returns the first children atom, if present.
    pub fn take_first_child(self) -> Option<AtomT> {
        self.into_iter().next()
    }

    /// Attempts to parse corresponding content from the `reader`.
    pub fn parse(&self, reader: &mut (impl Read + Seek), len: usize) -> crate::Result<Content> {
        Ok(match self {
            ContentT::Atoms(v) => Content::Atoms(atom::parse_atoms(reader, v, len)?),
            ContentT::RawData(d) => Content::RawData(data::parse_data(reader, *d, len)?),
            ContentT::TypedData => {
                if len >= 8 {
                    let datatype = match data::read_u32(reader) {
                        Ok(d) => d,
                        Err(e) => {
                            return Err(crate::Error::new(
                                e.kind,
                                "Error reading typed data head".to_owned(),
                            ));
                        }
                    };

                    // Skipping 4 byte locale indicator
                    reader.seek(SeekFrom::Current(4))?;

                    Content::TypedData(data::parse_data(reader, datatype, len - 8)?)
                } else {
                    return Err(crate::Error::new(
                        ErrorKind::Parsing,
                        "Typed data head to short".to_owned(),
                    ));
                }
            }
            ContentT::AudioInfo => Content::AudioInfo(AudioInfo::parse(reader, len)?),
            ContentT::Ignore => {
                reader.seek(SeekFrom::Current(len as i64))?;
                Content::Empty
            }
            ContentT::Empty => {
                if len != 0 {
                    return Err(crate::Error::new(
                        crate::ErrorKind::Parsing,
                        format!("Expected empty content found content of length: {}", len),
                    ));
                }
                Content::Empty
            }
        })
    }
}
