//! Object binary format for all XX-series Guilty Gears

use std::io::SeekFrom;
use binrw::binread;
use binrw::file_ptr::parse_from_iter;
use binrw::helpers::{until_eof, until_exclusive};
use serde::{Deserialize, Serialize};
use crate::helpers;
use std::iter::Peekable;
use crate::ggacpr::script::{GGXXObjScriptData, GGXXPlayerScriptData};

struct SkipLastIterator<I: Iterator>(Peekable<I>);
impl<I: Iterator> Iterator for SkipLastIterator<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.0.next();
        match self.0.peek() {
            Some(_) => Some(item.unwrap()),
            None => None,
        }
    }
}
trait SkipLast: Iterator + Sized {
    fn skip_last(self) -> SkipLastIterator<Self> {
        SkipLastIterator(self.peekable())
    }
}
impl<I: Iterator> SkipLast for I {}

helpers::impl_open!(GGXXObjBin);

#[binread]
#[br(little, stream = s)]
#[derive(Clone)]
pub struct GGXXObjBin {
    #[br(temp)]
    player_ptr: u32,
    #[br(seek_before(SeekFrom::Start(player_ptr as u64)))]
    pub player: GGXXPlayerEntry,
    #[br(temp)]
    #[br(
        parse_with = until_exclusive(|&dword| dword == 0xffffffff),
        seek_before(SeekFrom::Start(4))
    )]
    obj_pointers: Vec<u32>,
    #[br(try_calc = s.stream_position())]
    unk_offset: u64,
    #[br(
        parse_with = parse_from_iter(obj_pointers.iter().skip_last().copied()),
        seek_before(SeekFrom::Start(0))
    )]
    pub objects: Vec<GGXXObjEntry>,
    #[br(temp)]
    #[br(
        seek_before(SeekFrom::Start(unk_offset - 8)),
    )]
    unk_ptr: u32,
    #[br(
        parse_with = until_eof,
        seek_before(SeekFrom::Start(unk_ptr as u64))
    )]
    pub unk_section: Vec<u8>,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXPlayerEntry {
    #[br(try_calc = s.stream_position())]
    data_offset: u64,
    #[br(temp)]
    cell_pointer: u32,
    #[br(temp)]
    sprite_pointer: u32,
    #[br(temp)]
    script_pointer: u32,
    #[br(temp)]
    palette_pointer: u32,
    #[br(seek_before(SeekFrom::Start(data_offset + cell_pointer as u64)))]
    pub cell_array: GGXXCellArray,
    #[br(seek_before(SeekFrom::Start(data_offset + sprite_pointer as u64)))]
    pub sprite_array: GGXXSpriteArray,
    #[br(seek_before(SeekFrom::Start(data_offset + script_pointer as u64)))]
    pub script_data: GGXXPlayerScriptData,
    #[br(seek_before(SeekFrom::Start(data_offset + palette_pointer as u64)))]
    pub palette_array: GGXXPaletteArray,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXObjEntry {
    #[br(try_calc = s.stream_position())]
    data_offset: u64,
    #[br(temp)]
    cell_pointer: u32,
    #[br(temp)]
    sprite_pointer: u32,
    #[br(temp)]
    script_pointer: u32,
    #[br(temp)]
    unused: u32,
    #[br(seek_before(SeekFrom::Start(data_offset + cell_pointer as u64)))]
    pub cell_array: GGXXCellArray,
    #[br(seek_before(SeekFrom::Start(data_offset + sprite_pointer as u64)))]
    pub sprite_array: GGXXSpriteArray,
    #[br(seek_before(SeekFrom::Start(data_offset + script_pointer as u64)))]
    pub script_data: GGXXObjScriptData,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXCellArray {
    #[br(try_calc = s.stream_position())]
    data_offset: u64,
    #[br(temp)]
    #[br(parse_with = until_exclusive(|&dword| dword == 0xffffffff))]
    cell_pointers: Vec<u32>,
    #[br(
        parse_with = parse_from_iter(cell_pointers.iter().copied()),
        seek_before(SeekFrom::Start(data_offset))
    )]
    pub cell_entries: Vec<GGXXCellEntry>,
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXCellEntry {
    #[br(temp)]
    box_count: u32,
    #[br(count = box_count)]
    pub boxes: Vec<GGXXBox>,
    pub sprite_info: GGXXSpriteInfo,
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXBox {
    pub x_offset: i16,
    pub y_offset: i16,
    pub width: u16,
    pub height: u16,
    pub box_type: u32,
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXSpriteInfo {
    pub x_offset: i16,
    pub y_offset: i16,
    pub unk: u32,
    pub index: u16,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXSpriteArray {
    #[br(try_calc = s.stream_position())]
    data_offset: u64,
    #[br(temp)]
    #[br(parse_with = until_exclusive(|&dword| dword == 0xffffffff))]
    sprite_pointers: Vec<u32>,
    #[br(
        parse_with = parse_from_iter(sprite_pointers.iter().copied()),
        seek_before(SeekFrom::Start(data_offset))
    )]
    pub sprite_entries: Vec<GGXXSpriteData>,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXSpriteData {
    #[br(parse_with = until_exclusive(|&dword| dword == 0xffffffff))]
    pub data: Vec<u32>,
}

#[binread]
#[br(stream = s)]
#[derive(Clone)]
pub struct GGXXPaletteArray {
    #[br(try_calc = s.stream_position())]
    data_offset: u64,
    #[br(temp)]
    #[br(parse_with = until_exclusive(|&dword| dword == 0xffffffff))]
    palette_pointers: Vec<u32>,
    #[br(
        parse_with = parse_from_iter(palette_pointers.iter().copied()),
        seek_before(SeekFrom::Start(data_offset))
    )]
    pub palette_entries: Vec<GGXXPaletteEntry>,
}

#[binread]
#[derive(Clone, Serialize, Deserialize)]
pub struct GGXXPaletteEntry {
    pub unk: u16,
    pub unk1: u16,
    pub unk2: u16,
    pub unk3: u16,
    pub unk4: u16,
    pub unk5: u16,
    pub unk6: u16,
    pub unk7: u16,
    #[br(count = 256)]
    pub palette: Vec<u32>,
}

