//! PAC archive format support for most modern arcsys fighters
//! Currently rebuilds bit-perfect on most files with edge cases like [`PacStyle::PATH_CUT`]s hash function unsupported

use std::io::{Cursor, Read, SeekFrom};

use binrw::{binread, io::NoSeek, NullString};
use bitflags::bitflags;
use byteorder::WriteBytesExt;
use flate2::read::ZlibDecoder;

use crate::{arcsys_filename_hash, helpers};

helpers::impl_open!(Pac);
helpers::impl_open!(Zcmp);
helpers::impl_open!(DfasFPac);

#[binread]
#[br(little)]
#[derive(Clone, Debug)]
pub struct Pac {
    // hack to parse compressed pacs transparently
    // without exposing any extra info in the public API
    #[br(temp, map = |pac_reader: InternalPacReader| pac_reader.as_pac())]
    _pac: InternalPac,

    // actual struct fields:
    #[br(calc = _pac.compression)]
    pub compression: Compression,
    #[br(calc = _pac.pac_style)]
    pub pac_style: PacStyle,
    #[br(calc = _pac.entries)]
    pub entries: Vec<PacEntry>,
}

impl Pac {
    pub fn to_bytes(&self) -> Vec<u8> {
        fpac_to_bytes(self)
    }
}

fn fpac_to_bytes(pac: &Pac) -> Vec<u8> {
    use std::io::Write;

    use byteorder::LE;

    const HEADER_SIZE: usize = 0x20;

    let mut buffer = Vec::new();

    buffer.write_all(b"FPAC").unwrap();

    let mut meta_buffer: Vec<u8> = Vec::new();
    let mut file_buffer = Vec::new();

    let string_size = if !pac.pac_style.contains(PacStyle::ID_ONLY) {
        // max string size including null byte
        let max = pac
            .entries
            .iter()
            .map(|a| a.name().map_or(0, |a| a.len()))
            .max()
            .unwrap_or(0);

        if pac.pac_style.contains(PacStyle::VERSION2) {
            helpers::pad_to_nearest_with_excess(max + 1, 0x20)
        } else {
            helpers::pad_to_nearest_with_excess(max + 1, 0x4)
        }
    } else {
        0
    };

    let entry_count = pac.entries.len();

    for (entry_index, entry) in pac.entries.iter().enumerate() {
        // write header contents

        if !pac.pac_style.contains(PacStyle::ID_ONLY) {
            let fixed_name = helpers::string_to_fixed_bytes(
                entry.name().expect("PAC entry should have a name"),
                string_size,
            );
            meta_buffer.write_all(&fixed_name).unwrap();
        }

        meta_buffer.write_u32::<LE>(entry_index as u32).unwrap();
        // offset in the file section
        meta_buffer
            .write_u32::<LE>(file_buffer.len() as u32)
            .unwrap();
        meta_buffer
            .write_u32::<LE>(entry.contents.len() as u32)
            .unwrap();

        if pac.pac_style.contains(PacStyle::VERSION2) {
            meta_buffer.write_u32::<LE>(entry.hash_id()).unwrap();
        }

        let padding = if pac.pac_style.contains(PacStyle::HASH_SORT) {
            0
        } else {
            if pac.pac_style.contains(PacStyle::VERSION2) {
                helpers::needed_to_align(meta_buffer.len(), 0x10)
            } else {
                helpers::needed_to_align_with_excess(meta_buffer.len(), 0x10)
            }
        };

        (0..padding).for_each(|_| meta_buffer.write_u8(0).unwrap());

        // write file contents and pad
        file_buffer.write_all(&entry.contents).unwrap();

        let padding = if pac.pac_style.contains(PacStyle::ID_ONLY) {
            helpers::needed_to_align(file_buffer.len(), 0x4)
        } else {
            helpers::needed_to_align(file_buffer.len(), 0x10)
        };
        (0..padding).for_each(|_| file_buffer.write_u8(0).unwrap());
    }

    let header_size = HEADER_SIZE + meta_buffer.len();
    let total_size = header_size + file_buffer.len();

    buffer.write_u32::<LE>(header_size as u32).unwrap();
    buffer.write_u32::<LE>(total_size as u32).unwrap();
    buffer.write_u32::<LE>(entry_count as u32).unwrap();
    buffer.write_u32::<LE>(pac.pac_style.bits()).unwrap();
    buffer.write_u32::<LE>(string_size as u32).unwrap();

    // pad to 0x20
    buffer.write_u64::<LE>(0).unwrap();

    buffer.append(&mut meta_buffer);

    buffer.append(&mut file_buffer);

    buffer
}

#[binread]
#[br(magic = b"ZCMP", little)]
pub struct Zcmp {
    #[br(temp)]
    original_size: u32,
    #[br(temp, align_after = 0x10)]
    _compressed_size: u32,
    #[br(count = original_size, map_stream = |reader| NoSeek::new(ZlibDecoder::new(reader)) )]
    pub data: Vec<u8>,
}

#[binread]
#[br(magic = b"DFASFPAC", little)]
pub struct DfasFPac {
    #[br(temp)]
    original_size: u32,
    #[br(temp, align_after = 0x10)]
    _compressed_size: u32,
    #[br(count = original_size, map_stream = |reader| NoSeek::new(ZlibDecoder::new(reader)) )]
    pub data: Vec<u8>,
}

#[binread]
#[derive(Clone, Debug)]
#[br(little)]
enum InternalPacReader {
    Uncompressed(#[br(args(Compression::None))] InternalPac),
    #[br(magic = b"ZCMP")]
    Zcmp(
        // original size
        #[br(temp)] u32,
        // compressed size
        #[br(temp, align_after = 0x10)] u32,
        #[br(
            // awful hack because ZlibDecoder doesnt implement Seek
            // and NoSeek doesn't allow us to align or pad.
            // maybe implement some fake caching thing that emulates Seek over the stream?
            map_stream = |reader| {
                let mut decoder = ZlibDecoder::new(reader);
                let mut buf: Vec<u8> = Vec::new();
                decoder.read_to_end(&mut buf).expect("decoder should read to end");
                Cursor::new(buf)
            }
        )]
        #[br(args(Compression::Zcmp))]
        InternalPac,
    ),
    #[br(magic = b"DFASFPAC")]
    DfasFPac(
        #[br(temp)] u32,
        #[br(temp)] u32,
        #[br(
        // same hack again
            map_stream = |reader| {
                let mut decoder = ZlibDecoder::new(reader);
                let mut buf: Vec<u8> = Vec::new();
                decoder.read_to_end(&mut buf).expect("decoder should read to end");
                Cursor::new(buf)
            }
        )]
        #[br(args(Compression::DfasFPac))]
        InternalPac,
    ),
    #[br(magic = b"TXAC", pre_assert(false))]
    Txac,
}

impl InternalPacReader {
    fn as_pac(self) -> InternalPac {
        match self {
            Self::Uncompressed(p) => p,
            Self::Zcmp(p) => p,
            Self::DfasFPac(p) => p,
            Self::Txac => panic!("TXAC is unsupported!"),
        }
    }
}

#[binread]
#[derive(Clone, Debug)]
#[br(magic = b"FPAC", little)]
#[br(import(compression: Compression))]
struct InternalPac {
    #[br(calc = compression)]
    pub compression: Compression,
    #[br(temp)]
    data_start: u32,
    #[br(temp)]
    _total_size: u32,
    #[br(temp)]
    file_count: u32,
    #[br(map = |x: u32| PacStyle::from_bits_retain(x))]
    pub pac_style: PacStyle,
    #[br(temp, align_after = 0x10, dbg)]
    string_size: u32,
    #[br(args {
        count: file_count as usize,
        inner: (pac_style, string_size, data_start),
    })]
    pub entries: Vec<PacEntry>,
}

#[derive(Clone)]
enum EntryIdentifier {
    Name(String),
    Hash(u32),
}

#[binread]
#[derive(Clone)]
#[br(import(pac_style: PacStyle, string_size: u32, data_start: u32))]
pub struct PacEntry {
    #[br(
        temp,
        if(!pac_style.intersects(PacStyle::ID_ONLY) && string_size > 0),
        pad_size_to = string_size,
        map = |x: Option<NullString>| x.map(|s| encoding_rs::SHIFT_JIS.decode(&s.0).0.into_owned())
    )]
    name: Option<String>,
    #[br(temp)]
    _id: u32,
    #[br(temp)]
    file_offset: u32,
    #[br(temp)]
    file_size: u32,
    #[br(temp, align_after = 0x10)]
    hash: u32,
    #[br(calc(name.map(|x| EntryIdentifier::Name(x)).unwrap_or(EntryIdentifier::Hash(hash))))]
    identifier: EntryIdentifier,
    #[br(count = file_size, restore_position, seek_before = SeekFrom::Start((data_start + file_offset) as u64))]
    pub contents: Vec<u8>,
}

impl PacEntry {
    pub fn new_named(name: String, contents: impl Into<Vec<u8>>) -> Self {
        Self {
            identifier: EntryIdentifier::Name(name),
            contents: contents.into(),
        }
    }

    /// Create a new PacEntry that does not have a set filename, only using a hash for identification
    pub fn new_unnamed(hash: u32, contents: Vec<u8>) -> Self {
        Self {
            identifier: EntryIdentifier::Hash(hash),
            contents,
        }
    }

    /// Get the hash identifier for the entry, usually a hash of the filename, unkown for ID_ONLY pacs
    pub fn hash_id(&self) -> u32 {
        match self.identifier {
            EntryIdentifier::Name(ref name) => arcsys_filename_hash(name),
            EntryIdentifier::Hash(hash) => hash,
        }
    }

    /// Get the filename of the entry, returns None if the entry is for an ID_ONLY pac
    pub fn name(&self) -> Option<&str> {
        if let EntryIdentifier::Name(ref name) = self.identifier {
            return Some(name);
        }

        None
    }
}

impl std::fmt::Debug for PacEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacEntry")
            .field("name", &format_args!("{:?}", self.name()))
            // always represent has as a 4-byte hexadecimal value
            .field("hash", &format_args!("0x{:0>8X}", self.hash_id()))
            // exclude vec contents and just list size
            .field("file_size", &self.contents.len())
            .finish_non_exhaustive()
    }
}

// enum FPACK_STYLE
// FPACST_NORMAL    = 0
// FPACST_AUTOLEN_FILENAME  = 1
// FPACST_ID_ONLY   = 2
// FPACST_PATHCUT   = 10h
// FPACST_LONGNAME  = 20000000h
// FPACST_HASHSORT  = 40000000h
// FPACST_VERSION2  = 80000000h
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PacStyle: u32 {
        const NORMAL = 0x0;
        const AUTO_LEN_FILENAME = 0x1;
        const ID_ONLY = 0x2;
        const PATH_CUT = 0x10;
        const LONG_NAME = 0x20000000;
        const HASH_SORT = 0x40000000;
        const VERSION2 = 0x80000000;
    }
}

/// The compression wrapper used on the pac.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compression {
    None,
    /// Found in Bragon Ball Z: Extreme Butoden
    /// and One Piece: Great Pirate Colusseum
    Zcmp,
    /// Found in Blazblue Centralfiction
    DfasFPac,
}
