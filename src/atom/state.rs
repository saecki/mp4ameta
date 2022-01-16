use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    Existing(AtomBounds),
    //Replace(AtomBounds),
    New,
}

impl Default for State {
    fn default() -> Self {
        Self::New
    }
}
