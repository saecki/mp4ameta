use std::fmt::{Debug, Formatter, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{data, Content, Data, Tag, ErrorKind};

/// A list of valid file types defined by the `ftyp` atom.
pub const VALID_FILE_TYPES: [&str; 5] = ["M4A ", "M4B ", "M4P ", "M4V ", "isom"];

/// Identifier of an atom information about the filetype.
pub const FILE_TYPE: [u8; 4] = *b"ftyp";
/// Identifier of an atom containing a sturcture of children storing metadata.
pub const MOVIE: [u8; 4] = *b"moov";
/// Identifier of an atom containing information about a single track.
pub const TRACK: [u8; 4] = *b"trak";
/// Identifier of an atom containing inforamtion about a tracks media type and data.
pub const MEDIA: [u8; 4] = *b"mdia";
/// Identifier of an atom specifying the characteristics of a media atom.
pub const MEDIA_HEADER: [u8; 4] = *b"mdhd";
/// Identifier of an atom containing user metadata.
pub const USER_DATA: [u8; 4] = *b"udta";
/// Identifier of an atom containing a metadata item list.
pub const METADATA: [u8; 4] = *b"meta";
/// Identifier of an atom containing a list of metadata atoms.
pub const ITEM_LIST: [u8; 4] = *b"ilst";

// ITunes 4.0 atoms
pub const ALBUM: [u8; 4] = *b"\xa9alb";
pub const ALBUM_ARTIST: [u8; 4] = *b"aART";
pub const ARTIST: [u8; 4] = *b"\xa9ART";
pub const ARTWORK: [u8; 4] = *b"covr";
pub const BPM: [u8; 4] = *b"tmpo";
pub const COMMENT: [u8; 4] = *b"\xa9cmt";
pub const COMPILATION: [u8; 4] = *b"cpil";
pub const COMPOSER: [u8; 4] = *b"\xa9wrt";
pub const COPYRIGHT: [u8; 4] = *b"cprt";
pub const CUSTOM_GENRE: [u8; 4] = *b"\xa9gen";
pub const DISK_NUMBER: [u8; 4] = *b"disk";
pub const ENCODER: [u8; 4] = *b"\xa9too";
pub const ADVISORY_RATING: [u8; 4] = *b"rtng";
pub const STANDARD_GENRE: [u8; 4] = *b"gnre";
pub const TITLE: [u8; 4] = *b"\xa9nam";
pub const TRACK_NUMBER: [u8; 4] = *b"trkn";
pub const YEAR: [u8; 4] = *b"\xa9day";

// ITunes 4.2 atoms
pub const GROUPING: [u8; 4] = *b"\xa9grp";
pub const MEDIA_TYPE: [u8; 4] = *b"stik";

// ITunes 4.9 atoms
pub const CATEGORY: [u8; 4] = *b"catg";
pub const KEYWORD: [u8; 4] = *b"keyw";
pub const PODCAST: [u8; 4] = *b"pcst";
pub const PODCAST_EPISODE_GLOBAL_UNIQUE_ID: [u8; 4] = *b"egid";
pub const PODCAST_URL: [u8; 4] = *b"purl";

// ITunes 5.0
pub const DESCRIPTION: [u8; 4] = *b"desc";
pub const LYRICS: [u8; 4] = *b"\xa9lyr";

// ITunes 6.0
pub const TV_EPISODE: [u8; 4] = *b"tves";
pub const TV_EPISODE_NUMBER: [u8; 4] = *b"tven";
pub const TV_NETWORK_NAME: [u8; 4] = *b"tvnn";
pub const TV_SEASON: [u8; 4] = *b"tvsn";
pub const TV_SHOW_NAME: [u8; 4] = *b"tvsh";

// ITunes 6.0.2
pub const PURCHASE_DATE: [u8; 4] = *b"purd";

// ITunes 7.0
pub const GAPLESS_PLAYBACK: [u8; 4] = *b"pgap";

// Work, Movement
pub const MOVEMENT_NAME: [u8; 4] = *b"\xa9mvn";
pub const MOVEMENT_COUNT: [u8; 4] = *b"\xa9mvc";
pub const MOVEMENT_INDEX: [u8; 4] = *b"\xa9mvi";
pub const WORK: [u8; 4] = *b"\xa9wrk";
pub const SHOW_MOVEMENT: [u8; 4] = *b"shwm";

/// A structure that represents a MPEG-4 audio metadata atom.
#[derive(Clone, PartialEq)]
pub struct Atom {
    /// The 4 byte identifier of the atom.
    pub ident: [u8; 4],
    /// The offset in bytes separating the head from the content.
    pub offset: usize,
    /// The content of an atom.
    pub content: Content,
}

impl Atom {
    /// Creates an atom containing the provided content at a n byte offset.
    pub fn with(ident: [u8; 4], offset: usize, content: Content) -> Atom {
        Atom { ident, offset, content }
    }

    /// Creates an atom containing `Content::RawData` with the provided data.
    pub fn with_raw_data(ident: [u8; 4], offset: usize, data: Data) -> Atom {
        Atom::with(ident, offset, Content::RawData(data))
    }

    /// Creates a data atom containing unparsed `Content::TypedData`.
    pub fn data_atom() -> Atom {
        Atom::with(*b"data", 0, Content::TypedData(Data::Unparsed(data::TYPED)))
    }

    /// Creates a data atom containing `Content::TypedData` with the provided data.
    pub fn data_atom_with(data: Data) -> Atom {
        Atom::with(*b"data", 0, Content::TypedData(data))
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        8 + self.offset + self.content.len()
    }

    /// Returns true if the atom has no content and only consists of it's 8 byte head.
    pub fn is_empty(&self) -> bool {
        self.offset + self.content.len() == 0
    }

    /// Attempts to read MPEG-4 audio metadata from the reader.
    pub fn read_from(reader: &mut (impl Read + Seek)) -> crate::Result<Tag> {
        let mut ftyp = filetype_atom();
        let mut moov = metadata_atom();

        let mut tag_atoms = Vec::new();
        let mut tag_readonly_atoms = Vec::new();

        ftyp.parse(reader)?;
        ftyp.check_filetype()?;
        tag_readonly_atoms.push(ftyp);

        moov.parse(reader)?;

        if let Some(trak) = moov.child(TRACK) {
            if let Some(mdia) = trak.child(MEDIA) {
                if let Some(mdhd) = mdia.child(MEDIA_HEADER) {
                    tag_readonly_atoms.push(mdhd.clone());
                }
            }
        }

        if let Some(udta) = moov.child(USER_DATA) {
            if let Some(meta) = udta.first_child() {
                if let Some(ilst) = meta.first_child() {
                    if let Content::Atoms(atoms) = &ilst.content {
                        tag_atoms = atoms.to_vec();
                    }
                }
            }
        }

        Ok(Tag::with(tag_atoms, tag_readonly_atoms))
    }

    /// Attempts to write the metadata atoms to the file inside the item list atom.
    pub fn write_to_file(file: &File, atoms: &[Atom]) -> crate::Result<()> {
        let mut reader = BufReader::new(file);
        let mut writer = BufWriter::new(file);

        let mut atom_pos_and_len = Vec::new();
        let mut destination = &item_list_atom();
        let mut ftyp = filetype_atom();

        ftyp.parse(&mut reader)?;
        ftyp.check_filetype()?;

        while let Ok((length, ident)) = parse_head(&mut reader) {
            if ident == destination.ident {
                let pos = reader.seek(SeekFrom::Current(0))? as usize - 8;
                atom_pos_and_len.push((pos, length));

                reader.seek(SeekFrom::Current(destination.offset as i64))?;

                match destination.first_child() {
                    Some(a) => destination = a,
                    None => break,
                }
            } else {
                reader.seek(SeekFrom::Current(length as i64 - 8))?;
            }
        }

        let old_file_length = reader.seek(SeekFrom::End(0))?;
        let metadata_position = atom_pos_and_len[atom_pos_and_len.len() - 1].0 + 8;
        let old_metadata_length = atom_pos_and_len[atom_pos_and_len.len() - 1].1 - 8;
        let new_metadata_length = atoms.iter().map(|a| a.len()).sum::<usize>();
        let metadata_length_difference = new_metadata_length as i32 - old_metadata_length as i32;

        // reading additional data after metadata
        let mut additional_data = Vec::with_capacity(old_file_length as usize - (metadata_position + old_metadata_length));
        reader.seek(SeekFrom::Start((metadata_position + old_metadata_length) as u64))?;
        reader.read_to_end(&mut additional_data)?;

        // adjusting the file length
        file.set_len((old_file_length as i64 + metadata_length_difference as i64) as u64)?;

        // adjusting the atom lengths
        for (pos, len) in atom_pos_and_len {
            writer.seek(SeekFrom::Start(pos as u64))?;
            writer.write_u32::<BigEndian>((len as i32 + metadata_length_difference) as u32)?;
        }

        // writing metadata
        writer.seek(SeekFrom::Current(4))?;
        for a in atoms {
            a.write_to(&mut writer)?;
        }

        // writing additional data after metadata
        writer.write_all(&additional_data)?;
        writer.flush()?;

        Ok(())
    }

    /// Attempts to write the atom to the writer.
    pub fn write_to(&self, writer: &mut impl Write) -> crate::Result<()> {
        writer.write_u32::<BigEndian>(self.len() as u32)?;
        writer.write_all(&self.ident)?;
        writer.write_all(&vec![0u8; self.offset])?;

        self.content.write_to(writer)?;

        Ok(())
    }

    /// Attempts to parse itself from the reader.
    pub fn parse(&mut self, reader: &mut (impl Read + Seek)) -> crate::Result<()> {
        loop {
            let (length, ident) = match parse_head(reader) {
                Ok(h) => h,
                Err(e) => {
                    if let ErrorKind::Io(ioe) = &e.kind {
                        if ioe.kind() == std::io::ErrorKind::UnexpectedEof {
                            return Err(crate::Error::new(
                                ErrorKind::AtomNotFound(self.ident),
                                format!(
                                    "Reached EOF without finding an atom matching {}:",
                                    format_ident(self.ident)
                                ),
                            ));
                        }
                    }

                    return Err(e);
                }
            };

            if ident == self.ident {
                return match self.parse_content(reader, length) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(crate::Error::new(
                        e.kind,
                        format!(
                            "Error reading {}: {}",
                            format_ident(ident),
                            e.description
                        ),
                    )),
                };
            } else if length > 8 {
                reader.seek(SeekFrom::Current((length - 8) as i64))?;
            }
        }
    }

    /// Attempts to parse the list of atoms from the reader.
    pub fn parse_atoms(atoms: &mut Vec<Atom>, reader: &mut (impl Read + Seek), length: usize) -> crate::Result<()> {
        let mut parsed_atoms = 0;
        let mut parsed_bytes = 0;
        let atom_count = atoms.len();

        while parsed_bytes < length && parsed_atoms < atom_count {
            let (atom_length, atom_ident) = parse_head(reader)?;

            let mut parsed = false;
            for a in atoms.iter_mut() {
                if atom_ident == a.ident {
                    if let Err(e) = a.parse_content(reader, atom_length) {
                        return Err(crate::Error::new(
                            e.kind,
                            format!(
                                "Error reading {}: {}",
                                format_ident(atom_ident),
                                e.description
                            ),
                        ));
                    }
                    parsed = true;
                    parsed_atoms += 1;
                    break;
                }
            }

            if atom_length > 8 && !parsed {
                reader.seek(SeekFrom::Current((atom_length - 8) as i64))?;
            }

            parsed_bytes += atom_length;
        }

        if parsed_bytes < length {
            reader.seek(SeekFrom::Current((length - parsed_bytes) as i64))?;
        }

        Ok(())
    }

    /// Attempts to parse the content of the provided length from the reader.
    pub fn parse_content(&mut self, reader: &mut (impl Read + Seek), length: usize) -> crate::Result<()> {
        if length > 8 {
            if self.offset != 0 {
                reader.seek(SeekFrom::Current(self.offset as i64))?;
            }
            self.content.parse(reader, length - 8)?;
        } else {
            self.content = Content::Empty;
        };

        Ok(())
    }

    /// Attempts to return a reference to the first children atom matching the identifier.
    pub fn child(&self, ident: [u8; 4]) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Attempts to return a mutable reference to the first children atom matching the identifier.
    pub fn mut_child(&mut self, ident: [u8; 4]) -> Option<&mut Atom> {
        if let Content::Atoms(v) = &mut self.content {
            for a in v {
                if a.ident == ident {
                    return Some(a);
                }
            }
        }

        None
    }

    /// Attempts to return a reference to the first children atom.
    pub fn first_child(&self) -> Option<&Atom> {
        if let Content::Atoms(v) = &self.content {
            return v.first();
        }

        None
    }

    /// Attempts to return a mutable reference to the first children atom.
    pub fn mut_first_child(&mut self) -> Option<&mut Atom> {
        if let Content::Atoms(v) = &mut self.content {
            return v.first_mut();
        }

        None
    }

    /// Checks if the filetype is valid, returns an error otherwise.
    pub fn check_filetype(&self) -> crate::Result<()> {
        match &self.content {
            Content::RawData(Data::Utf8(s)) => {
                for f in &VALID_FILE_TYPES {
                    if s.starts_with(f) {
                        return Ok(());
                    }
                }

                Err(crate::Error::new(
                    ErrorKind::InvalidFiletype(s.clone()),
                    "Invalid filetype.".into(),
                ))
            }
            _ => Err(crate::Error::new(
                ErrorKind::NoTag,
                "No filetype atom found.".into(),
            )),
        }
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let ident_string = format_ident(self.ident);
        write!(f, "Atom{{ {}, {}, {:#?} }}", ident_string, self.offset, self.content)
    }
}

/// Attempts to parse the atoms head containing a 32 bit unsigned integer determining the size
/// of the atom in bytes and the following 4 byte identifier from the reader.
pub fn parse_head(reader: &mut (impl Read + Seek)) -> crate::Result<(usize, [u8; 4])> {
    let length = match reader.read_u32::<BigEndian>() {
        Ok(l) => l as usize,
        Err(e) => {
            return Err(crate::Error::new(
                ErrorKind::Io(e),
                "Error reading atom length".into(),
            ));
        }
    };
    let mut ident = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut ident) {
        return Err(crate::Error::new(
            ErrorKind::Io(e),
            "Error reading atom identifier".into(),
        ));
    }

    Ok((length, ident))
}

/// Returns the identifier formatted as a string.
pub fn format_ident(ident: [u8; 4]) -> String {
    ident.iter().map(|b| char::from(*b)).collect()
}

/// Returns a atom filetype hierarchy needed to parse the filetype.
pub fn filetype_atom() -> Atom {
    Atom::with_raw_data(FILE_TYPE, 0, Data::Unparsed(data::UTF8))
}

/// Returns a atom metadata hierarchy needed to parse metadata.
pub fn metadata_atom() -> Atom {
    Atom::with(MOVIE, 0, Content::atoms()
        .add_atom_with(TRACK, 0, Content::atoms()
            .add_atom_with(MEDIA, 0, Content::atoms()
                .add_atom_with(MEDIA_HEADER, 0, Content::RawData(
                    Data::Unparsed(data::RESERVED)
                )),
            ),
        )
        .add_atom_with(USER_DATA, 0, Content::atoms()
            .add_atom_with(METADATA, 4, Content::atoms()
                .add_atom_with(ITEM_LIST, 0, Content::atoms()
                    .add_atom_with(ADVISORY_RATING, 0, Content::data_atom())
                    .add_atom_with(ALBUM, 0, Content::data_atom())
                    .add_atom_with(ALBUM_ARTIST, 0, Content::data_atom())
                    .add_atom_with(ARTIST, 0, Content::data_atom())
                    .add_atom_with(BPM, 0, Content::data_atom())
                    .add_atom_with(CATEGORY, 0, Content::data_atom())
                    .add_atom_with(COMMENT, 0, Content::data_atom())
                    .add_atom_with(COMPILATION, 0, Content::data_atom())
                    .add_atom_with(COMPOSER, 0, Content::data_atom())
                    .add_atom_with(COPYRIGHT, 0, Content::data_atom())
                    .add_atom_with(CUSTOM_GENRE, 0, Content::data_atom())
                    .add_atom_with(DESCRIPTION, 0, Content::data_atom())
                    .add_atom_with(DISK_NUMBER, 0, Content::data_atom())
                    .add_atom_with(ENCODER, 0, Content::data_atom())
                    .add_atom_with(GAPLESS_PLAYBACK, 0, Content::data_atom())
                    .add_atom_with(GROUPING, 0, Content::data_atom())
                    .add_atom_with(KEYWORD, 0, Content::data_atom())
                    .add_atom_with(LYRICS, 0, Content::data_atom())
                    .add_atom_with(MEDIA_TYPE, 0, Content::data_atom())
                    .add_atom_with(MOVEMENT_COUNT, 0, Content::data_atom())
                    .add_atom_with(MOVEMENT_INDEX, 0, Content::data_atom())
                    .add_atom_with(MOVEMENT_NAME, 0, Content::data_atom())
                    .add_atom_with(PODCAST, 0, Content::data_atom())
                    .add_atom_with(PODCAST_EPISODE_GLOBAL_UNIQUE_ID, 0, Content::data_atom())
                    .add_atom_with(PODCAST_URL, 0, Content::data_atom())
                    .add_atom_with(PURCHASE_DATE, 0, Content::data_atom())
                    .add_atom_with(SHOW_MOVEMENT, 0, Content::data_atom())
                    .add_atom_with(STANDARD_GENRE, 0, Content::data_atom())
                    .add_atom_with(TITLE, 0, Content::data_atom())
                    .add_atom_with(TRACK_NUMBER, 0, Content::data_atom())
                    .add_atom_with(TV_EPISODE, 0, Content::data_atom())
                    .add_atom_with(TV_EPISODE_NUMBER, 0, Content::data_atom())
                    .add_atom_with(TV_NETWORK_NAME, 0, Content::data_atom())
                    .add_atom_with(TV_SEASON, 0, Content::data_atom())
                    .add_atom_with(TV_SHOW_NAME, 0, Content::data_atom())
                    .add_atom_with(WORK, 0, Content::data_atom())
                    .add_atom_with(YEAR, 0, Content::data_atom())
                    .add_atom_with(ARTWORK, 0, Content::data_atom(),
                    ),
                ),
            ),
        ),
    )
}

/// Returns a atom metadata hierarchy.
pub fn item_list_atom() -> Atom {
    Atom::with(MOVIE, 0, Content::atom_with(
        USER_DATA, 0, Content::atom_with(
            METADATA, 4, Content::atom_with(
                ITEM_LIST, 0, Content::atoms(),
            ),
        ),
    ))
}
