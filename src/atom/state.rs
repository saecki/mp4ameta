use super::*;

/// The state of an atom.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    /// The atom already exists. Contains the current bounds the atom.
    Existing(AtomBounds),
    /// The atom already existed and will be replaced. Contains the old bounds the atom.
    Replace(AtomBounds),
    /// The atom already existed and will be removed. Contains the old bounds the atom.
    Remove(AtomBounds),
    /// The atom will be added.
    Insert,
}

impl Default for State {
    fn default() -> Self {
        Self::Insert
    }
}

impl State {
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
