use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Gmin {
    pub state: State,
    pub version: u8,
    pub flags: [u8; 3],
    pub graphics_mode: u16,
    pub op_color: [u16; 3],
    pub balance: u16,
}

impl Atom for Gmin {
    const FOURCC: Fourcc = BASE_MEDIA_INFORMATION;
}

impl ParseAtom for Gmin {
    fn parse_atom(
        reader: &mut (impl Read + Seek),
        _cfg: &ParseConfig<'_>,
        size: Size,
    ) -> crate::Result<Self> {
        let bounds = find_bounds(reader, size)?;
        let mut gmin = Self {
            state: State::Existing(bounds),
            ..Default::default()
        };

        let (version, flags) = parse_full_head(reader)?;
        gmin.version = version;
        gmin.flags = flags;
        if version != 0 {
            return Err(crate::Error::new(
                crate::ErrorKind::UnknownVersion(version),
                format!("Unknown base media information (gmin) version {version}"),
            ));
        }

        gmin.graphics_mode = reader.read_be_u16()?;
        for c in gmin.op_color.iter_mut() {
            *c = reader.read_be_u16()?;
        }
        gmin.balance = reader.read_be_u16()?;
        reader.skip(2)?; // reserved

        Ok(gmin)
    }
}

impl WriteAtom for Gmin {
    fn write_atom(&self, writer: &mut impl Write, _changes: &[Change<'_>]) -> crate::Result<()> {
        self.write_head(writer)?;
        write_full_head(writer, self.version, self.flags)?;

        writer.write_be_u16(self.graphics_mode)?;
        for c in self.op_color {
            writer.write_be_u16(c)?;
        }
        writer.write_be_u16(self.balance)?;
        writer.write_be_u16(0)?; // reserved

        Ok(())
    }

    fn size(&self) -> Size {
        Size::from(16)
    }
}

impl Gmin {
    pub fn chapter() -> Self {
        Self {
            state: State::Insert,
            version: 0,
            flags: [0; 3],
            graphics_mode: 0x0040,
            op_color: [0x8000; 3],
            balance: 0,
        }
    }
}
