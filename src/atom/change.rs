use std::borrow::Cow;

use super::*;

pub trait CollectChanges {
    /// Recursively collect changes and return the length difference when applied.
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64;
}

impl<T: CollectChanges> CollectChanges for Option<T> {
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        self.as_ref().map_or(0, |a| a.collect_changes(insert_pos, level, changes))
    }
}

pub trait SimpleCollectChanges: AtomSize + Atom {
    fn state(&self) -> &State;

    /// Add changes, if any, and return the length difference when applied.
    fn existing<'a>(
        &'a self,
        level: u8,
        bounds: &'a AtomBounds,
        changes: &mut Vec<Change<'a>>,
    ) -> i64;

    fn atom_ref(&self) -> AtomRef<'_>;
}

impl<T: SimpleCollectChanges> CollectChanges for T {
    fn collect_changes<'a>(
        &'a self,
        insert_pos: u64,
        level: u8,
        changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        match &self.state() {
            State::Existing(bounds) => {
                let len_diff = self.existing(level + 1, bounds, changes);
                if len_diff != 0 {
                    changes.push(Change::UpdateLen(UpdateAtomLen {
                        bounds,
                        fourcc: Self::FOURCC,
                        len_diff,
                    }));
                }
                len_diff
            }
            State::Remove(bounds) => {
                changes.push(Change::Remove(RemoveAtom { bounds, level: level + 1 }));
                -(bounds.len() as i64)
            }
            State::Replace(bounds) => {
                let len_diff = (self.len() as i64) - (bounds.len() as i64);
                let r = ReplaceAtom { bounds, atom: self.atom_ref(), level: level + 1 };
                changes.push(Change::Replace(r));
                len_diff
            }
            State::Insert => {
                changes.push(Change::Insert(InsertAtom {
                    pos: insert_pos,
                    atom: self.atom_ref(),
                    level: level + 1,
                }));
                self.len() as i64
            }
        }
    }
}

pub trait LeafAtomCollectChanges: SimpleCollectChanges {
    fn state(&self) -> &State;

    fn atom_ref(&self) -> AtomRef<'_>;
}

impl<T: LeafAtomCollectChanges> SimpleCollectChanges for T {
    fn state(&self) -> &State {
        LeafAtomCollectChanges::state(self)
    }

    fn existing<'a>(
        &'a self,
        _level: u8,
        _bounds: &'a AtomBounds,
        _changes: &mut Vec<Change<'a>>,
    ) -> i64 {
        0
    }

    fn atom_ref(&self) -> AtomRef<'_> {
        LeafAtomCollectChanges::atom_ref(self)
    }
}

#[derive(Debug)]
pub enum Change<'a> {
    UpdateLen(UpdateAtomLen<'a>),
    UpdateChunkOffset(UpdateChunkOffsets<'a>),
    Remove(RemoveAtom<'a>),
    Replace(ReplaceAtom<'a>),
    Insert(InsertAtom<'a>),
    RemoveMdat(u64, u64),
    AppendMdat(u64, Vec<u8>),
}

impl std::fmt::Display for Change<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        match self {
            Change::UpdateLen(UpdateAtomLen { fourcc, .. }) => write!(f, "UpdateLen   {fourcc}  "),
            Change::UpdateChunkOffset(_)                    => write!(f, "UpdateChunkOffset "),
            Change::Remove(_)                               => write!(f, "RemoveAtom        "),
            Change::Replace(r)                              => write!(f, "ReplaceAtom {}  ", r.atom.fourcc()),
            Change::Insert(i)                               => write!(f, "InsertAtom  {}  ", i.atom.fourcc()),
            Change::RemoveMdat(..)                          => write!(f, "RemoveMdat        "),
            Change::AppendMdat(..)                          => write!(f, "AppendMdat        "),
        }?;
        write!(
            f,
            "old_pos: {:6}, old_end: {:6}, len_diff: {:6}, level: {}",
            self.old_pos(),
            self.old_end(),
            self.len_diff(),
            self.level()
        )
    }
}

impl Change<'_> {
    pub fn old_pos(&self) -> u64 {
        match self {
            Self::UpdateLen(c) => c.bounds.pos(),
            Self::UpdateChunkOffset(c) => c.bounds.content_pos() + stco::HEADER_SIZE,
            Self::Remove(c) => c.bounds.pos(),
            Self::Replace(c) => c.bounds.pos(),
            Self::Insert(c) => c.pos,
            Self::RemoveMdat(pos, _) => *pos,
            Self::AppendMdat(pos, _) => *pos,
        }
    }

    pub fn old_end(&self) -> u64 {
        match self {
            Self::UpdateLen(c) => c.bounds.content_pos(),
            Self::UpdateChunkOffset(c) => c.bounds.end(),
            Self::Remove(c) => c.bounds.end(),
            Self::Replace(c) => c.bounds.end(),
            Self::Insert(c) => c.pos,
            Self::RemoveMdat(pos, len) => *pos + *len,
            Self::AppendMdat(pos, _) => *pos,
        }
    }

    pub fn len_diff(&self) -> i64 {
        match self {
            Self::UpdateLen(_) => 0,
            Self::UpdateChunkOffset(_) => 0,
            Self::Remove(c) => -(c.bounds.len() as i64),
            Self::Replace(c) => (c.atom.len() as i64) - (c.bounds.len() as i64),
            Self::Insert(c) => c.atom.len() as i64,
            Self::RemoveMdat(_, len) => -(*len as i64),
            Self::AppendMdat(_, d) => d.len() as i64,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Self::UpdateLen(_) => 0,
            Self::UpdateChunkOffset(_) => 6,
            Self::Remove(c) => c.level,
            Self::Replace(c) => c.level,
            Self::Insert(c) => c.level,
            Self::RemoveMdat(_, _) => u8::MAX,
            Self::AppendMdat(_, _) => u8::MAX,
        }
    }
}

#[derive(Debug)]
pub struct UpdateAtomLen<'a> {
    pub bounds: &'a AtomBounds,
    pub fourcc: Fourcc,
    pub len_diff: i64,
}

impl UpdateAtomLen<'_> {
    pub fn update_len(&self, writer: &mut impl Write) -> crate::Result<()> {
        let len = (self.bounds.len() as i64 + self.len_diff) as u64;
        let head = Head::new(self.bounds.ext(), len, self.fourcc);
        head::write(writer, head)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RemoveAtom<'a> {
    pub bounds: &'a AtomBounds,
    pub level: u8,
}

#[derive(Debug)]
pub struct ReplaceAtom<'a> {
    pub bounds: &'a AtomBounds,
    pub atom: AtomRef<'a>,
    pub level: u8,
}

#[derive(Debug)]
pub struct InsertAtom<'a> {
    pub pos: u64,
    pub atom: AtomRef<'a>,
    pub level: u8,
}

#[derive(Debug)]
pub struct UpdateChunkOffsets<'a> {
    pub bounds: &'a AtomBounds,
    pub offsets: ChunkOffsets<'a>,
}

#[derive(Debug)]
pub enum ChunkOffsets<'a> {
    Stco(Cow<'a, [u32]>),
    Co64(Cow<'a, [u64]>),
}

impl ChunkOffsets<'_> {
    pub fn update_offsets(
        &self,
        writer: &mut impl Write,
        changes: &[Change<'_>],
    ) -> crate::Result<()> {
        match self {
            ChunkOffsets::Stco(offsets) => write_shifted_offsets(writer, offsets, changes),
            ChunkOffsets::Co64(offsets) => write_shifted_offsets(writer, offsets, changes),
        }
    }
}

pub trait ChunkOffsetInt: Sized + Copy + Into<u64> {
    fn shift(&self, shift: i64) -> Self;
    fn write(&self, writer: &mut impl Write) -> crate::Result<()>;
}

impl ChunkOffsetInt for u32 {
    fn shift(&self, shift: i64) -> Self {
        (*self as i64 + shift) as u32
    }

    fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_be_u32(*self)?;
        Ok(())
    }
}
impl ChunkOffsetInt for u64 {
    fn shift(&self, shift: i64) -> Self {
        (*self as i64 + shift) as u64
    }

    fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_be_u64(*self)?;
        Ok(())
    }
}

pub fn write_shifted_offsets<T: ChunkOffsetInt>(
    writer: &mut impl Write,
    offsets: &[T],
    changes: &[Change<'_>],
) -> crate::Result<()> {
    let mut changes_iter = changes.iter().peekable();

    let mut mdat_shift = 0;
    for o in offsets.iter().copied() {
        loop {
            if let Some(change) = changes_iter.peek() {
                if change.old_pos() <= o.into() {
                    mdat_shift += change.len_diff();
                    changes_iter.next();
                    continue;
                }
            }
            break;
        }

        o.shift(mdat_shift).write(writer)?;
    }
    Ok(())
}

macro_rules! write_or_ignore {
    (nowrite, $($write:tt)*) => {
        Ok(())
    };
    (, $($write:tt)*) => {
        $($write)*
    };
}

// false positive
#[allow(unused)]
macro_rules! test_or_ignore {
    (nowrite, $($write:tt)*) => {};
    (, $($write:tt)*) => {
        $($write)*
    };
}

macro_rules! atom_ref {
    ($($name:ident $(<$lifetime:lifetime>)? $($nowrite:ident)? ,)+) => {
        #[derive(Debug)]
        pub enum AtomRef<'a> {
            $($name(&'a $name $(<$lifetime>)?)),+
        }

        impl AtomRef<'_> {
            pub fn write(&self, writer: &mut impl Write, changes: &[Change<'_>]) -> crate::Result<()> {
                match self {
                    #[allow(unused)]
                    $(Self::$name(a) => write_or_ignore!($($nowrite)?, {a.write(writer, changes)}),)+
                }
            }

            pub fn fourcc(&self) -> Fourcc {
                match self {
                    $(Self::$name(_) => $name::FOURCC,)+
                }
            }

            fn len(&self) -> u64 {
                match self {
                    $(Self::$name(a) => a.len(),)+
                }
            }
        }

        #[cfg(test)]
        mod verify_written_length {
            use super::*;

            $(
                test_or_ignore! { $($nowrite)?,
                    #[test]
                    #[allow(non_snake_case)]
                    fn $name() {
                        let atom = $name::default();
                        let changes = [];

                        let mut buf: Vec<u8> = Vec::new();
                        let mut cursor = std::io::Cursor::new(&mut buf);
                        atom.write(&mut cursor, &changes).unwrap();

                        let buf_size = buf.len() as u64;

                        let mut cursor = std::io::Cursor::new(&buf);
                        cursor.seek(SeekFrom::Start(0)).unwrap();
                        let head = head::parse(&mut cursor, buf_size).unwrap();

                        assert_eq!(atom.len(), head.len());
                        assert_eq!(atom.len(), buf_size);
                    }
                }
            )+
        }
    };
}

atom_ref!(
    Moov<'a> nowrite,
    Udta<'a>,
    Chpl<'a>,
    Meta<'a>,
    Hdlr,
    Ilst<'a>,
    Trak,
    Tref,
    Chap,
    Mdia,
    Minf,
    Dinf,
    Dref,
    Url,
    Gmhd,
    Gmin,
    Text,
    Stbl,
    Stsd,
    Stts,
    Stsc,
    Stsz,
    Stco,
    Co64,
);
