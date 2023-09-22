use std::io::{Cursor, Read, SeekFrom};

use binrw::{binread, io::NoSeek, NullString};
use bitflags::bitflags;
use flate2::read::ZlibDecoder;

use crate::helpers;

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

#[binread]
#[br(magic = b"ZCMP", little)]
pub struct Zcmp {
    #[br(temp)]
    original_size: u32,
    #[br(temp, align_after = 0x10)]
    _compressed_size: u32,
    #[br(args { count: original_size.try_into().unwrap() }, map_stream = |reader| NoSeek::new(ZlibDecoder::new(reader)) )]
    pub data: Vec<u8>,
}

#[binread]
#[br(magic = b"DFASFPAC", little)]
pub struct DfasFPac {
    #[br(temp)]
    original_size: u32,
    #[br(temp, align_after = 0x10)]
    _compressed_size: u32,
    #[br(args { count: original_size.try_into().unwrap() }, map_stream = |reader| NoSeek::new(ZlibDecoder::new(reader)) )]
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
}

impl InternalPacReader {
    fn as_pac(self) -> InternalPac {
        match self {
            Self::Uncompressed(p) => p,
            Self::Zcmp(p) => p,
            Self::DfasFPac(p) => p,
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
    #[br(temp, align_after = 0x10)]
    string_size: u32,
    #[br(args {
        count: file_count as usize,
        inner: (pac_style, string_size, data_start),
    })]
    pub entries: Vec<PacEntry>,
}

#[binread]
#[derive(Clone)]
#[br(import(pac_style: PacStyle, string_size: u32, data_start: u32))]
pub struct PacEntry {
    #[br(if(!pac_style.intersects(PacStyle::ID_ONLY) && string_size > 0), pad_size_to = string_size, map = |x: Option<NullString>| x.map(|s| s.to_string()) )]
    pub name: Option<String>,
    #[br(temp)]
    _id: u32,
    #[br(temp)]
    file_offset: u32,
    #[br(temp)]
    file_size: u32,
    #[br(align_after = 0x10)]
    name_hash: u32,
    #[br(count = file_size, restore_position, seek_before = SeekFrom::Start((data_start + file_offset) as u64))]
    pub contents: Vec<u8>,
}

impl PacEntry {
    pub fn name_hash(&self) -> u32 {
        if let Some(name) = &self.name {
            crate::arcsys_filename_hash(name)
        } else {
            self.name_hash
        }
    }
}

impl std::fmt::Debug for PacEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacEntry")
            .field("name", &format_args!("{:?}", self.name))
            // always represent has as a 4-byte hexadecimal value
            .field("name_hash", &format_args!("0x{:0>8X}", &self.name_hash))
            // exclude vec contents and just list size
            .field("file_size", &self.contents.len())
            .finish_non_exhaustive()
    }
}

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
