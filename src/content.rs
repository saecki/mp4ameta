use std::fmt::{Debug, Formatter, Result};
use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{Atom, Data, ErrorKind};
use crate::atom::AtomTemplate;
use crate::data::DataTemplate;

/// An enum representing the different types of content an Atom might have.
#[derive(Clone, PartialEq)]
pub enum Content {
    /// A value containing `Vec<Atom>`.
    Atoms(Vec<Atom>),
    /// A value containing raw `Data`.
    RawData(Data),
    /// A value containing `Data` defined by a [Table 3-5 Well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34) code.
    TypedData(Data),
    /// Empty `Content`.
    Empty,
}

#[derive(Clone, PartialEq)]
pub enum ContentTemplate {
    Atoms(Vec<AtomTemplate>),
    RawData(DataTemplate),
    TypedData,
    Empty,
}

impl Debug for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Content::Atoms(a) => write!(f, "Content::Atoms{{ {:#?} }}", a),
            Content::TypedData(d) => write!(f, "Content::TypedData{{ {:?} }}", d),
            Content::RawData(d) => write!(f, "Content::RawData{{ {:?} }}", d),
            Content::Empty => write!(f, "Content::Empty"),
        }
    }
}

impl Content {
    /// Creates a new content of type `Content::Atoms` containing an empty `Vec`.
    pub fn atoms() -> Content {
        Content::Atoms(Vec::new())
    }

    /// Creates a new content of type `Content::Atoms` containing the atom.
    pub fn atom(atom: Atom) -> Content {
        Content::Atoms(vec![atom])
    }

    /// Creates a new content of type `Content::Atoms` containing a data `Atom` with the data.
    pub fn data_atom_with(data: Data) -> Content {
        Content::atom(Atom::data_atom_with(data))
    }

    /// Creates a new `Content` of type `Content::Atoms` containing a new `Atom` with the identifier,
    /// offset and content.
    pub fn atom_with(ident: [u8; 4], offset: usize, content: Content) -> Content {
        Content::atom(Atom::with(ident, offset, content))
    }

    /// Adds the atom to the list of children atoms if `self` is of type `Content::Atoms`.
    pub fn add_atom(self, atom: Atom) -> Content {
        if let Content::Atoms(mut atoms) = self {
            atoms.push(atom);
            Content::Atoms(atoms)
        } else {
            self
        }
    }

    /// Adds a new `Atom` with the provided identifier, offset and content to the list of children if
    /// `self` is of type `Content::Atoms`.
    pub fn add_atom_with(self, ident: [u8; 4], offset: usize, content: Content) -> Content {
        self.add_atom(Atom::with(ident, offset, content))
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        match self {
            Content::Atoms(v) => v.iter().map(|a| a.len()).sum(),
            Content::TypedData(d) => 8 + d.len(),
            Content::RawData(d) => d.len(),
            Content::Empty => 0,
        }
    }

    /// Returns true if the content is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Attempts to write the content to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        match self {
            Content::Atoms(v) => {
                for a in v {
                    a.write_to(writer)?;
                }
            }
            Content::RawData(d) => d.write_raw(writer)?,
            Content::TypedData(d) => d.write_typed(writer)?,
            Content::Empty => (),
        }

        Ok(())
    }
}

impl Debug for ContentTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ContentTemplate::Atoms(a) => write!(f, "ContentTemplate::Atoms{{ {:#?} }}", a),
            ContentTemplate::TypedData => write!(f, "ContentTemplate::TypedData"),
            ContentTemplate::RawData(d) => write!(f, "ContentTemplate::RawData{{ {:?} }}", d),
            ContentTemplate::Empty => write!(f, "ContentTemplate::Empty"),
        }
    }
}

impl ContentTemplate {
    /// Creates a new content of type `Content::Atoms` containing an empty `Vec`.
    pub fn atoms_template() -> Self {
        Self::Atoms(Vec::new())
    }

    /// Creates a new content of type `Self::Atoms` containing the atom.
    pub fn atom(atom: AtomTemplate) -> Self {
        Self::Atoms(vec![atom])
    }

    /// Creates a new content of type `Self::Atoms` containing a data `Atom`.
    pub fn data_atom_template() -> Self {
        Self::atom(AtomTemplate::data_atom())
    }

    /// Creates a new `Self` of type `Self::Atoms` containing a new `Atom` with the identifier,
    /// offset and content.
    pub fn atom_with(ident: [u8; 4], offset: usize, content: Self) -> Self {
        Self::atom(AtomTemplate::with(ident, offset, content))
    }

    /// Adds the atom to the list of children atoms if `self` is of type `Self::Atoms`.
    pub fn add_atom(self, atom: AtomTemplate) -> Self {
        if let Self::Atoms(mut atoms) = self {
            atoms.push(atom);
            Self::Atoms(atoms)
        } else {
            self
        }
    }

    /// Adds a data `Atom` to the list of children if `self` is of type `Self::Atoms`.
    pub fn add_data_atom(self) -> Self {
        self.add_atom(AtomTemplate::data_atom())
    }

    /// Adds a new atom with the provided identifier, offset and content to the list of children
    /// if `self` is of type `ContentTemplate::Atoms`.
    pub fn add_atom_template_with(self, ident: [u8; 4], offset: usize, content: Self) -> Self {
        self.add_atom(AtomTemplate::with(ident, offset, content))
    }

    /// Attempts to parse itself from the reader.
    pub fn parse(&self, reader: &mut (impl Read + Seek), length: usize) -> crate::Result<Content> {
        Ok(match self {
            ContentTemplate::Atoms(v) => Content::Atoms(AtomTemplate::parse_atoms(v, reader, length)?),
            ContentTemplate::RawData(d) => Content::RawData(d.parse(reader, length)?),
            ContentTemplate::TypedData => {
                if length >= 8 {
                    let datatype = match reader.read_u32::<BigEndian>() {
                        Ok(d) => d,
                        Err(e) => return Err(crate::Error::new(
                            crate::ErrorKind::Io(e),
                            "Error reading typed data head".into(),
                        )),
                    };

                    // Skipping 4 byte locale indicator
                    reader.seek(SeekFrom::Current(4))?;

                    Content::TypedData(DataTemplate::with(datatype).parse(reader, length - 8)?)
                } else {
                    return Err(crate::Error::new(
                        ErrorKind::Parsing,
                        "Typed data head to short".into(),
                    ));
                }
            }
            ContentTemplate::Empty => Content::Empty,
        })
    }
}