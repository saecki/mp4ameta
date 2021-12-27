//! A meta item can either have a plain fourcc as it's identifier:
//! **** (any fourcc)
//! └─ data
//!
//! Or it can contain a mean and name children atom which make up the identifier.
//! ---- (freeform fourcc)
//! ├─ mean
//! ├─ name
//! └─ data
use super::*;

/// A struct representing a metadata item, containing data that is associated with an identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaItem {
    /// The identifier of the atom.
    pub ident: DataIdent,
    /// The data contained in the atom.
    pub data: Vec<Data>,
}

impl MetaItem {
    /// Creates a meta item with the identifier and data.
    pub const fn new(ident: DataIdent, data: Vec<Data>) -> Self {
        Self { ident, data }
    }

    /// Returns the external length of the atom in bytes.
    pub fn len(&self) -> u64 {
        let parent_len = 8;
        let data_len: u64 = self.data.iter().map(|d| d.size().len()).sum();

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

    pub fn parse(reader: &mut (impl Read + Seek), parent: Fourcc, len: u64) -> crate::Result<Self> {
        let mut data = Vec::new();
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < len {
            let head = parse_head(reader)?;

            match head.fourcc() {
                DATA => data.push(Data::parse(reader, head.size())?),
                MEAN => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    mean = Some(reader.read_utf8(head.content_len() - 4)?);
                }
                NAME => {
                    let (version, _) = parse_full_head(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading data atom (data)".to_owned(),
                        ));
                    }

                    name = Some(reader.read_utf8(head.content_len() - 4)?);
                }
                _ => {
                    reader.seek(SeekFrom::Current(head.content_len() as i64))?;
                }
            }

            parsed_bytes += head.len();
        }

        let ident = match (parent, mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (fourcc, _, _) => DataIdent::Fourcc(fourcc),
        };

        if data.is_empty() {
            return Err(crate::Error::new(
                crate::ErrorKind::AtomNotFound(DATA),
                format!("Error constructing meta item '{}', missing data atom", parent),
            ));
        }

        Ok(MetaItem { ident, data })
    }

    /// Attempts to write the meta item to the writer.
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
                writer.write_all(&[0; 4])?;
                writer.write_all(mean.as_bytes())?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_all(&u32::to_be_bytes(name_len))?;
                writer.write_all(NAME.deref())?;
                writer.write_all(&[0; 4])?;
                writer.write_all(name.as_bytes())?;
            }
        }

        for d in self.data.iter() {
            d.write(writer)?;
        }

        Ok(())
    }
}
