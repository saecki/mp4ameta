use super::*;

pub trait ChangeBounds {
    fn old_pos(&self) -> u64;

    fn old_end(&self) -> u64;

    fn len_diff(&self) -> i64;

    fn level(&self) -> u8;
}

#[derive(Debug)]
pub enum Change<'a> {
    UpdateLen(UpdateAtomLen<'a>),
    UpdateChunkOffset(UpdateChunkOffsets<'a>),
    Remove(RemoveAtom<'a>),
    Replace(ReplaceAtom<'a>),
    Insert(InsertAtom<'a>),
    AppendMdat(u64, &'a [u8]),
}

impl ChangeBounds for Change<'_> {
    fn old_pos(&self) -> u64 {
        match self {
            Self::UpdateLen(c) => c.old_pos(),
            Self::UpdateChunkOffset(c) => c.old_pos(),
            Self::Remove(c) => c.old_pos(),
            Self::Replace(c) => c.old_pos(),
            Self::Insert(c) => c.old_pos(),
            Self::AppendMdat(pos, _) => *pos,
        }
    }

    fn old_end(&self) -> u64 {
        match self {
            Self::UpdateLen(c) => c.old_end(),
            Self::UpdateChunkOffset(c) => c.old_end(),
            Self::Remove(c) => c.old_end(),
            Self::Replace(c) => c.old_end(),
            Self::Insert(c) => c.old_end(),
            Self::AppendMdat(pos, _) => *pos,
        }
    }

    fn len_diff(&self) -> i64 {
        match self {
            Self::UpdateLen(c) => c.len_diff(),
            Self::UpdateChunkOffset(c) => c.len_diff(),
            Self::Remove(c) => c.len_diff(),
            Self::Replace(c) => c.len_diff(),
            Self::Insert(c) => c.len_diff(),
            Self::AppendMdat(_, d) => d.len() as i64,
        }
    }

    fn level(&self) -> u8 {
        match self {
            Self::UpdateLen(c) => c.level(),
            Self::UpdateChunkOffset(c) => c.level(),
            Self::Remove(c) => c.level(),
            Self::Replace(c) => c.level(),
            Self::Insert(c) => c.level(),
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
        write_head(writer, head)?;
        Ok(())
    }
}

impl ChangeBounds for UpdateAtomLen<'_> {
    fn old_pos(&self) -> u64 {
        self.bounds.pos()
    }

    fn old_end(&self) -> u64 {
        self.bounds.content_pos()
    }

    fn len_diff(&self) -> i64 {
        0
    }

    fn level(&self) -> u8 {
        0
    }
}

#[derive(Debug)]
pub struct UpdateChunkOffsets<'a> {
    pub bounds: &'a AtomBounds,
    pub offsets: ChunkOffsets<'a>,
}

#[derive(Debug)]
pub enum ChunkOffsets<'a> {
    Stco(&'a [u32]),
    Co64(&'a [u64]),
}

impl ChunkOffsets<'_> {
    pub fn update_offsets(&self, writer: &mut impl Write, mdat_shift: i64) -> crate::Result<()> {
        match self {
            Self::Stco(offsets) => {
                for o in offsets.iter() {
                    writer.write_be_u32((*o as i64 + mdat_shift) as u32)?;
                }
            }
            Self::Co64(offsets) => {
                for o in offsets.iter() {
                    writer.write_be_u64((*o as i64 + mdat_shift) as u64)?;
                }
            }
        }
        Ok(())
    }
}

impl ChangeBounds for UpdateChunkOffsets<'_> {
    fn old_pos(&self) -> u64 {
        self.bounds.content_pos() + stco::HEADER_SIZE
    }

    fn old_end(&self) -> u64 {
        self.bounds.end()
    }

    fn len_diff(&self) -> i64 {
        0
    }

    fn level(&self) -> u8 {
        6
    }
}

#[derive(Debug)]
pub struct RemoveAtom<'a> {
    pub bounds: &'a AtomBounds,
    pub level: u8,
}

impl ChangeBounds for RemoveAtom<'_> {
    fn old_pos(&self) -> u64 {
        self.bounds.pos()
    }

    fn old_end(&self) -> u64 {
        self.bounds.end()
    }

    fn len_diff(&self) -> i64 {
        -(self.bounds.len() as i64)
    }

    fn level(&self) -> u8 {
        self.level
    }
}

#[derive(Debug)]
pub struct ReplaceAtom<'a> {
    pub bounds: &'a AtomBounds,
    pub atom: AtomRef<'a>,
    pub level: u8,
}

impl ChangeBounds for ReplaceAtom<'_> {
    fn old_pos(&self) -> u64 {
        self.bounds.pos()
    }

    fn old_end(&self) -> u64 {
        self.bounds.end()
    }

    fn len_diff(&self) -> i64 {
        self.atom.len() as i64 - self.bounds.len() as i64
    }

    fn level(&self) -> u8 {
        self.level
    }
}

#[derive(Debug)]
pub struct InsertAtom<'a> {
    pub pos: u64,
    pub atom: AtomRef<'a>,
    pub level: u8,
}

impl ChangeBounds for InsertAtom<'_> {
    fn old_pos(&self) -> u64 {
        self.pos
    }

    fn old_end(&self) -> u64 {
        self.pos
    }

    fn len_diff(&self) -> i64 {
        self.atom.len() as i64
    }

    fn level(&self) -> u8 {
        self.level
    }
}

macro_rules! atom_ref {
    ($($name:ident $(<$lifetime:lifetime>)?,)+) => {
        #[derive(Debug)]
        pub enum AtomRef<'a> {
            $($name(&'a $name $(<$lifetime>)?)),+
        }

        impl AtomRef<'_> {
            pub fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
                match self {
                    $(Self::$name(a) => a.write(writer)),+
                }
            }

            fn len(&self) -> u64 {
                match self {
                    $(Self::$name(a) => a.len()),+
                }
            }
        }
    };
}

atom_ref!(
    Moov<'a>,
    Udta<'a>,
    Chpl<'a>,
    Meta<'a>,
    Hdlr,
    Ilst<'a>,
    Trak,
    Mdia,
    Minf,
    Stbl,
    Stco,
    Co64,
);
