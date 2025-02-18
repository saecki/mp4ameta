//! A metadata item can either have a plain fourcc as it's identifier:
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
    /// Creates a metadata item with the identifier and data.
    pub const fn new(ident: DataIdent, data: Vec<Data>) -> Self {
        Self { ident, data }
    }

    /// Returns the external length of the atom in bytes.
    pub fn len(&self) -> u64 {
        let parent_len = 8;
        let data_len: u64 = self.data.iter().map(Data::len).sum();

        match &self.ident {
            DataIdent::Fourcc(_) => parent_len + data_len,
            DataIdent::Freeform { mean, name } => {
                let mean_len = 12 + mean.len() as u64;
                let name_len = 12 + name.len() as u64;

                parent_len + mean_len + name_len + data_len
            }
        }
    }

    pub fn parse(
        reader: &mut (impl Read + Seek),
        cfg: &ParseConfig<'_>,
        head: Head,
    ) -> crate::Result<Self> {
        let mut data = Vec::new();
        let mut mean: Option<String> = None;
        let mut name: Option<String> = None;
        let mut parsed_bytes = 0;

        while parsed_bytes < head.content_len() {
            let head = head::parse(reader)?;

            match head.fourcc() {
                DATA => data.push(Data::parse(reader, cfg, head.size())?),
                MEAN => {
                    let (version, _) = head::parse_full(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading mean atom (mean)",
                        ));
                    }

                    mean = Some(reader.read_utf8(head.content_len() - 4)?);
                }
                NAME => {
                    let (version, _) = head::parse_full(reader)?;
                    if version != 0 {
                        return Err(crate::Error::new(
                            crate::ErrorKind::UnknownVersion(version),
                            "Error reading name atom (name)",
                        ));
                    }

                    name = Some(reader.read_utf8(head.content_len() - 4)?);
                }
                _ => reader.skip(head.content_len() as i64)?,
            }

            parsed_bytes += head.len();
        }

        let ident = match (head.fourcc(), mean, name) {
            (FREEFORM, Some(mean), Some(name)) => DataIdent::Freeform { mean, name },
            (fourcc, _, _) => DataIdent::Fourcc(fourcc),
        };

        Ok(MetaItem { ident, data })
    }

    /// Attempts to write the metadata item to the writer.
    pub fn write(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_be_u32(self.len() as u32)?;

        match &self.ident {
            DataIdent::Fourcc(ident) => writer.write_all(ident.deref())?,
            _ => {
                let (mean, name) = match &self.ident {
                    DataIdent::Freeform { mean, name } => (mean.as_str(), name.as_str()),
                    DataIdent::Fourcc(_) => unreachable!(),
                };
                writer.write_all(FREEFORM.deref())?;

                let mean_len: u32 = 12 + mean.len() as u32;
                writer.write_be_u32(mean_len)?;
                writer.write_all(&*MEAN)?;
                writer.write_all(&[0; 4])?;
                writer.write_utf8(mean)?;

                let name_len: u32 = 12 + name.len() as u32;
                writer.write_be_u32(name_len)?;
                writer.write_all(&*NAME)?;
                writer.write_all(&[0; 4])?;
                writer.write_utf8(name)?;
            }
        }

        for d in self.data.iter() {
            d.write(writer)?;
        }

        Ok(())
    }
}
