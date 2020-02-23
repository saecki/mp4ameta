use byteorder::{ReadBytesExt, BigEndian};
use std::io::{BufReader, Read};
use std::fs::File;
use crate::{Data, Content};
use crate::{Error, ErrorKind};
use crate::content::Content::Atoms;

pub struct Atom {
    header: [u8; 4],
    offset: u32,
    content: Content,
}

impl Atom {
    pub fn read_from(reader: &mut BufReader<File>) -> crate::Result<Atom> {
        let ftyp = Atom::parse_atoms(vec!(Atom::filetype_atom()), reader);

        if let Ok(mut v) = ftyp {
            for a in v {
                if let Content::Data(Data::UTF8(Ok(s))) = a.content {
                    debug_assert!(s.starts_with("M4A "));
                }
            }
        }

        if let Content::Atoms(atoms) = Atom::metadata_atom().content {
            let moov = Atom::parse_atoms(atoms, reader);
        }

        Err(Error::new(ErrorKind::InvalidInput, "no ftyp atom"))
    }

    pub fn parse(self, reader: &mut BufReader<File>) -> crate::Result<Atom> {
        if let Some(a) = Atom::parse_atoms(vec![self], reader)?.first() {
            Ok(Atom::from(a.header, a.offset, a.content))
        } else {
            Err(Error::new(ErrorKind::NoTag, ""))
        }
    }

    pub fn parse_atoms(atoms: Vec<Atom>, reader: &mut BufReader<File>) -> crate::Result<Vec<Atom>> {
        let mut parsed_atoms = Vec::new();

        loop {
            let length = match reader.read_u32::<BigEndian>() {
                Ok(l) => l,
                Err(e) => {
                    println!("error reading atom length");
                    return Err(crate::Error::from(e));
                }
            };
            println!("length: {:?}", length);

            let mut header = [0_u8; 4];
            let r = reader.read_exact(&mut header);

            println!("{:?}", header);
            println!("{:?}", String::from_utf8(header.to_vec()));

            if let Err(_e) = r {
                println!("error reading header");
                break;
            }

            let mut parsed = false;
            for a in &atoms {
                if header == a.header {
                    if length > 8 {
                        if a.offset != 0 {
                            Data::read_to_u8_vec(reader, a.offset);
                        }
                        let content = a.content.parse(reader, length - 8)?;
                        parsed_atoms.push(Atom::from(
                            a.header,
                            a.offset,
                            content,
                        ))
                    } else {
                        parsed_atoms.push(Atom::from(
                            a.header,
                            0,
                            Content::Empty,
                        ));
                    };

                    parsed = true;
                    break;
                }
            }

            if length > 0 && !parsed {
                Data::read_to_u8_vec(reader, length - 8);
            }
        }
        Err(crate::Error::new(
            ErrorKind::InvalidInput,
            "File has no ftyp atom",
        ))
    }

    pub fn new() -> Atom {
        Atom { header: *b"    ", offset: 0, content: Content::Data(Data::empty_unknown()) }
    }

    pub fn from(header: [u8; 4], offset: u32, content: Content) -> Atom {
        Atom { header, offset, content }
    }

    pub fn with_data(header: [u8; 4], offset: u32, data: Data) -> Atom {
        Atom::from(header, offset, Content::Data(data))
    }

    fn filetype_atom() -> Atom {
        Atom::with_data(*b"ftyp", 0, Data::empty_utf8())
    }

    fn metadata_atom() -> Atom {
        Atom::from(
            *b"moov", 0, Content::atom(
                *b"udta", 0, Content::atom(
                    *b"meta", 4, Content::atom(
                        *b"ilst", 0, Content::atoms()
                            .add_atom(*b"\xa9alb", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"\xa9ART", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"aART", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"\xa9cmt", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"disk", 4, Content::data_atom(0, Data::empty_unknown()))
                            .add_atom(*b"\xa9wrt", 4, Content::data_atom(0, Data::empty_unknown()))
                            .add_atom(*b"\xa9day", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"\xa9gen", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"gnre", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"\xa9nam", 4, Content::data_atom(0, Data::empty_utf8()))
                            .add_atom(*b"trkn", 4, Content::data_atom(0, Data::empty_utf8())),
                    ),
                ),
            ),
        )
    }
}