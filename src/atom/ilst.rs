use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ilst<'a> {
    Owned(Vec<AtomData>),
    Borrowed(&'a [AtomData]),
}

impl Deref for Ilst<'_> {
    type Target = [AtomData];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(a) => &a,
            Self::Borrowed(a) => a,
        }
    }
}

impl TempAtom for Ilst<'_> {
    const FOURCC: Fourcc = ITEM_LIST;
}

impl ParseAtom for Ilst<'_> {
    fn parse_atom(reader: &mut (impl Read + Seek), size: Size) -> crate::Result<Self> {
        let mut ilst = Vec::<AtomData>::new();
        let mut parsed_bytes = 0;

        while parsed_bytes < size.content_len() {
            let head = parse_head(reader)?;

            match head.fourcc() {
                FREE => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
                _ => {
                    let atom = AtomData::parse(reader, head.fourcc(), head.content_len())?;
                    let other = ilst.iter_mut().find(|o| atom.ident == o.ident);

                    match other {
                        Some(other) => other.data.extend(atom.data),
                        None => ilst.push(atom),
                    }
                }
            }

            parsed_bytes += head.len();
        }

        Ok(Self::Owned(ilst))
    }
}

impl WriteAtom for Ilst<'_> {
    fn write_atom(&self, writer: &mut impl Write) -> crate::Result<()> {
        self.write_head(writer)?;
        for a in self.iter() {
            a.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        let content_len = self.iter().map(|a| a.len()).sum();
        Size::from(content_len)
    }
}

impl Ilst<'_> {
    pub fn owned(self) -> Option<Vec<AtomData>> {
        match self {
            Self::Owned(a) => Some(a),
            Self::Borrowed(_) => None,
        }
    }
}

/// A struct representing data that is associated with an atom identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomData {
    /// The identifier of the atom.
    pub ident: DataIdent,
    /// The data contained in the atom.
    pub data: Vec<Data>,
}

impl AtomData {
    /// Creates atom data with the identifier and data.
    pub const fn new(ident: DataIdent, data: Vec<Data>) -> Self {
        Self { ident, data }
    }

    /// Returns the external length of the atom in bytes.
    pub fn len(&self) -> u64 {
        let parent_len = 8;
        let data_len: u64 = self.data.iter().map(|d| 16 + d.len()).sum();

        match &self.ident {
            DataIdent::Fourcc(_) => parent_len + data_len,
            DataIdent::Freeform { mean, name } => {
                let mean_len = 12 + mean.len() as u64;
                let name_len = 12 + name.len() as u64;

                parent_len + mean_len + name_len + data_len
            }
        }
    }

    /// Returns whether the inner data atom is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() || self.data.iter().all(|d| d.is_empty())
    }

    fn parse(reader: &mut (impl Read + Seek), parent: Fourcc, len: u64) -> crate::Result<Self> {
        let mut data = Vec::new();
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                DATA => {
                    let (version, flags) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }
                    let [b2, b1, b0] = flags;
                    let datatype = u32::from_be_bytes([0, b2, b1, b0]);

                    // Skipping 4 byte locale indicator
                    reader.seek(SeekFrom::Current(4))?;

                    data.push(data::parse_data(reader, datatype, head.content_len() - 8)?);
                }
                MEAN => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    mean = Some(data::read_utf8(reader, head.content_len() - 4)?);
                }
                NAME => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    name = Some(data::read_utf8(reader, head.content_len() - 4)?);
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        let ident = match (parent, mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (ident, _, _) => DataIdent::Fourcc(ident),
        };

        if data.is_empty() {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(DATA),
                format!("Error constructing atom data '{}', missing data atom", parent),
            ));
        }

        Ok(AtomData { ident, data })
    }

    /// Attempts to write the atom data to the writer.
    pub fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_all(&u32::to_be_bytes(self.len() as u32))?;

        match &self.ident {
            DataIdent::Fourcc(ident) => writer.write_all(ident.deref())?,
            _ => {
                let (mean, name) = match &self.ident {
                    DataIdent::Freeform { mean, name } => (mean.as_str(), name.as_str()),
                    DataIdent::Fourcc(_) => unreachable!(),
                };
                writer.write_all(FREEFORM.deref())?;

                let mean_len: u32 = 12 + mean.len() as u32;
                writer.write_all(&u32::to_be_bytes(mean_len))?;
                writer.write_all(MEAN.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(mean.as_bytes())?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_all(&u32::to_be_bytes(name_len))?;
                writer.write_all(NAME.deref())?;
                writer.write_all(&[0u8; 4])?;
                writer.write_all(name.as_bytes())?;
            }
        }

        for d in self.data.iter() {
            let data_len: u32 = 16 + d.len() as u32;
            writer.write_all(&u32::to_be_bytes(data_len))?;
            writer.write_all(DATA.deref())?;
            d.write_typed(writer)?;
        }

        Ok(())
    }
}
