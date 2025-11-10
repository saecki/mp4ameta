use crate::atom::head::AtomBounds;

/// The state of an atom.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum State {
    /// The atom already exists. Contains the current bounds the atom.
    Existing(AtomBounds),
    /// The atom already existed and will be replaced. Contains the old bounds the atom.
    Replace(AtomBounds),
    /// The atom already existed and will be removed. Contains the old bounds the atom.
    Remove(AtomBounds),
    /// The atom will be added.
    #[default]
    Insert,
}

impl State {
    pub fn has_existed(&self) -> bool {
        matches!(self, Self::Existing(_) | Self::Replace(_) | Self::Remove(_))
    }

    pub fn is_existing(&self) -> bool {
        matches!(self, Self::Existing(_))
    }

    pub fn replace_existing(&mut self) {
        if let Self::Existing(b) = self {
            *self = Self::Replace(b.clone())
        }
    }

    pub fn remove_existing(&mut self) {
        if let Self::Existing(b) = self {
            *self = Self::Remove(b.clone())
        }
    }
}
