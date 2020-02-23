use std::io::{BufReader, Read};
use std::fs::File;
use crate::Atom;
use crate::{Error, ErrorKind};

pub enum Content {
    Atoms(Vec<Atom>),
    Data(Data),
    Empty,
}

pub enum Data {
    UTF8(crate::Result<String>),
    UTF16(crate::Result<String>),
    Unknown(crate::Result<Vec<u8>>),
}

impl Content {
    pub fn parse(self, reader: &mut BufReader<File>, length: u32) -> crate::Result<Content> {
        match self {
            Content::Data(d) => Ok(Content::Data(d.parse(reader, length)?)),
            Content::Atoms(v) => Ok(Content::Atoms(Atom::parse_atoms(v, reader)?)),
            Content::Empty => Ok(Content::Empty),
        }
    }

    pub fn atoms() -> Content {
        Content::Atoms(Vec::new())
    }

    pub fn atom(header: [u8; 4], offset: u32, content: Content) -> Content {
        let mut v = Vec::new();
        v.push(Atom::from(header, offset, content));
        Content::Atoms(v)
    }

    pub fn data_atom(offset: u32, data: Data) -> Content {
        Content::atom(*b"data", offset, Content::Data(data))
    }

    pub fn add_atom(self, header: [u8; 4], offset: u32, content: Content) -> Content {
        if let Content::Atoms(mut atoms) = self {
            atoms.push(Atom::from(header, offset, content));
            Content::Atoms(atoms)
        } else {
            self
        }
    }

    pub fn add_data_atom(self, offset: u32, data: Data) -> Content {
        self.add_atom(*b"data", offset, Content::Data(data))
    }
}

impl Data {
    pub fn parse(self, reader: &mut BufReader<File>, length: u32) -> crate::Result<Data> {
        match self {
            Data::Unknown(_) =>
                Data::Unknown(Data::read_to_u8_vec(reader, length)),
            Data::UTF8(_) => {
                let s = String::from_utf8(Data::read_to_u8_vec(reader, length)?)?;

                println!("{:?}", s);

                Data::UTF8(Ok(s))
            }
            _ => Data::empty_unknown(),
        };

        Err(Error::new(ErrorKind::Parsing, "no such datatype"))
    }

    pub fn read_to_u8_vec(reader: &mut BufReader<File>, length: u32) -> crate::Result<Vec<u8>> {
        let mut buff = vec![0u8; length as usize];
        reader.read_exact(&mut buff)?;

        Ok(buff)
    }

    pub fn empty_utf8() -> Data {
        Data::UTF8(Err(Error::new(ErrorKind::Parsing, "empty")))
    }

    pub fn empty_utf16() -> Data {
        Data::UTF16(Err(Error::new(ErrorKind::Parsing, "empty")))
    }

    pub fn empty_unknown() -> Data {
        Data::Unknown(Err(Error::new(ErrorKind::Parsing, "empty")))
    }
}