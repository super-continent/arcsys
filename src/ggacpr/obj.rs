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

impl GGXXObjBin {
    pub fn to_bytes(&self) -> Vec<u8> {obj_to_bytes(self)}
}

fn obj_to_bytes(obj: &GGXXObjBin) -> Vec<u8> {
    let mut buffer = Vec::new();

    let mut obj_pointers = Vec::new();
    let obj_num = 2 + obj.objects.len();
    let mut obj_buffer = Vec::new();

    let mut player_buffer = Vec::new();

    let mut player_pointers: Vec<u32> = Vec::new();
    player_pointers.push(0x20);

    let mut player_cell_pointers: Vec<u32> = Vec::new();
    let mut player_cell_buffer = Vec::new();

    let padding = helpers::needed_to_align(obj.player.cell_array.cell_entries.len() * 4, 0x10);

    for cell in obj.player.cell_array.cell_entries.iter() {
        player_cell_pointers.push(
            (obj.player.cell_array.cell_entries.len() * 4 + player_cell_buffer.len()) as u32);
        let mut cell_bin : Vec<u8> = Vec::new();

        cell_bin.append(&mut (cell.boxes.len() as u32).to_le_bytes().to_vec());

        for col_box in cell.boxes.iter() {
            cell_bin.append(&mut col_box.x_offset.to_le_bytes().to_vec());
            cell_bin.append(&mut col_box.y_offset.to_le_bytes().to_vec());
            cell_bin.append(&mut col_box.width.to_le_bytes().to_vec());
            cell_bin.append(&mut col_box.height.to_le_bytes().to_vec());
            cell_bin.append(&mut col_box.box_type.to_le_bytes().to_vec());
        }

        cell_bin.append(&mut cell.sprite_info.x_offset.to_le_bytes().to_vec());
        cell_bin.append(&mut cell.sprite_info.y_offset.to_le_bytes().to_vec());
        cell_bin.append(&mut cell.sprite_info.unk.to_le_bytes().to_vec());
        cell_bin.append(&mut cell.sprite_info.index.to_le_bytes().to_vec());

        player_cell_buffer.append(&mut cell_bin);
    };

    (0..padding / 4).for_each(|_| player_cell_pointers.push(0xFFFFFFFF));

    let padding = helpers::needed_to_align(player_cell_buffer.len(), 0x10);
    (0..padding).for_each(|_| player_cell_buffer.push(0xFF));

    player_pointers.push((0x20 + player_cell_pointers.len() * 4 + player_cell_buffer.len()) as u32);

    let mut player_sprite_pointers: Vec<u32> = Vec::new();
    let mut player_sprite_buffer = Vec::new();

    let padding = helpers::needed_to_align(obj.player.sprite_array.sprite_entries.len() * 4, 0x10);

    for sprite in obj.player.sprite_array.sprite_entries.iter() {
        player_sprite_pointers.push(
            (obj.player.sprite_array.sprite_entries.len() * 4 + player_sprite_buffer.len() + padding) as u32);
        let vec32 = &sprite.data;
        let mut byte_array = unsafe { vec32.align_to::<u8>().1 }.to_vec();
        player_sprite_buffer.append(&mut byte_array);

        let padding = helpers::needed_to_align(player_sprite_buffer.len(), 0x10);
        (0..padding).for_each(|_| player_sprite_buffer.push(0xFF));
    };

    (0..padding / 4).for_each(|_| player_sprite_pointers.push(0xFFFFFFFF));

    player_pointers.push(
        (0x20 + player_cell_pointers.len() * 4 + player_cell_buffer.len()
            + player_sprite_pointers.len() * 4 + player_sprite_buffer.len()) as u32);

    let mut play_data_buffer: Vec<u16> = Vec::new();

    let play_data = &obj.player.script_data.play_data;
    {
        play_data_buffer.push((play_data.unk as u16) << 8 | 0xE5);
        play_data_buffer.push(play_data.fwalk_vel as u16);
        play_data_buffer.push(play_data.bwalk_vel as u16);
        play_data_buffer.push(play_data.fdash_init_vel as u16);
        play_data_buffer.push(play_data.bdash_x_vel as u16);
        play_data_buffer.push(play_data.bdash_y_vel as u16);
        play_data_buffer.push(play_data.bdash_gravity as u16);
        play_data_buffer.push(play_data.fjump_x_vel as u16);
        play_data_buffer.push(play_data.bjump_x_vel as u16);
        play_data_buffer.push(play_data.jump_y_vel as u16);
        play_data_buffer.push(play_data.jump_gravity as u16);
        play_data_buffer.push(play_data.fsuperjump_x_vel as u16);
        play_data_buffer.push(play_data.bsuperjump_x_vel as u16);
        play_data_buffer.push(play_data.superjump_y_vel as u16);
        play_data_buffer.push(play_data.superjump_gravity as u16);
        play_data_buffer.push(play_data.fdash_accel as u16);
        play_data_buffer.push(play_data.fdash_reduce as u16);
        play_data_buffer.push(play_data.init_homingjump_y_vel as u16);
        play_data_buffer.push(play_data.init_homingjump_x_vel as u16);
        play_data_buffer.push(play_data.init_homingjump_x_reduce as u16);
        play_data_buffer.push(play_data.init_homingjump_y_offset as u16);
        play_data_buffer.push(play_data.airdash_minimum_height as u16);
        play_data_buffer.push(play_data.fairdash_time as u16);
        play_data_buffer.push(play_data.bairdash_time as u16);
        play_data_buffer.push(play_data.stun_res as u16);
        play_data_buffer.push(play_data.defense as u16);
        play_data_buffer.push(play_data.guts as u16);
        play_data_buffer.push(play_data.critical as u16);
        play_data_buffer.push(play_data.weight as u16);
        play_data_buffer.push(play_data.airdash_count as u16);
        play_data_buffer.push(play_data.airjump_count as u16);
        play_data_buffer.push(play_data.fairdash_no_attack_time as u16);
        play_data_buffer.push(play_data.bairdash_no_attack_time as u16);
        play_data_buffer.push(play_data.fwalk_tension as u16);
        play_data_buffer.push(play_data.fjump_tension as u16);
        play_data_buffer.push(play_data.fdash_tension as u16);
        play_data_buffer.push(play_data.fairdash_tension as u16);
        play_data_buffer.push(play_data.guardbalance_defense as u16);
        play_data_buffer.push(play_data.guardbalance_tension as u16);
        play_data_buffer.push(play_data.instantblock_tension as u16);
    }

    let mut player_script_buffer = Vec::new();
    player_script_buffer.append(&mut unsafe { play_data_buffer.align_to::<u8>().1 }.to_vec());
    player_script_buffer.append(&mut obj.player.script_data.unk_data.clone());
    for action in obj.player.script_data.actions.iter() {
        for instruction in action.instructions.iter() {
            let mut instruction_buffer = bincode::serialize(&instruction).unwrap();
            player_script_buffer.append(&mut instruction_buffer);
        }
    }

    let script_padding = if player_script_buffer.len() % 0x1000 > 0xA00 {
        0x1000 - player_script_buffer.len() % 0x1000 + 0xA00
    } else {
        0xA00 - player_script_buffer.len() % 0x1000
    };
    (0..script_padding).for_each(|_| player_script_buffer.push(0x00));

    player_pointers.push(
        (0x20 + player_cell_buffer.len() + player_cell_pointers.len() * 4
            + player_sprite_pointers.len() * 4 + player_sprite_buffer.len()
            + player_script_buffer.len()) as u32);

    for _ in 0..4 {
        player_pointers.push(0xFFFFFFFF);
    }

    let mut player_palette_pointers: Vec<u32> = Vec::new();
    let mut player_palette_buffer = Vec::new();

    let padding = helpers::needed_to_align(obj.player.palette_array.palette_entries.len() * 4, 0x10);

    for palette in obj.player.palette_array.palette_entries.iter() {
        player_palette_pointers.push(
            (obj.player.palette_array.palette_entries.len() * 4 + player_palette_buffer.len() + padding) as u32);
        player_palette_buffer.append(&mut bincode::serialize(&palette).unwrap());

        let padding = helpers::needed_to_align(player_palette_buffer.len(), 0x10);
        (0..padding).for_each(|_| player_palette_buffer.push(0xFF));
    };

    (0..padding / 4).for_each(|_| player_palette_pointers.push(0xFFFFFFFF));

    player_buffer.append(&mut unsafe { player_pointers.align_to::<u8>().1 }.to_vec());
    player_buffer.append(&mut unsafe { player_cell_pointers.align_to::<u8>().1 }.to_vec());
    player_buffer.append(&mut player_cell_buffer);
    player_buffer.append(&mut unsafe { player_sprite_pointers.align_to::<u8>().1 }.to_vec());
    player_buffer.append(&mut player_sprite_buffer);
    player_buffer.append(&mut player_script_buffer);
    player_buffer.append(&mut unsafe { player_palette_pointers.align_to::<u8>().1 }.to_vec());
    player_buffer.append(&mut player_palette_buffer);

    let padding = if obj_num % 4 == 0 {
        0x10
    } else {
        0x10 - obj_num * 4
    };
    let initial_offset = obj_num * 4 + padding;

    obj_pointers.push(initial_offset as u32);

    for game_object in obj.objects.iter() {
        obj_pointers.push((initial_offset + player_buffer.len() + obj_buffer.len()) as u32);
        let mut game_object_pointers: Vec<u32> = Vec::new();
        game_object_pointers.push(0x10);

        let mut game_object_cell_pointers: Vec<u32> = Vec::new();
        let mut game_object_cell_buffer = Vec::new();

        let padding = helpers::needed_to_align(game_object.cell_array.cell_entries.len() * 4, 0x10);

        for cell in game_object.cell_array.cell_entries.iter() {
            game_object_cell_pointers.push(
                (game_object.cell_array.cell_entries.len() * 4 + game_object_cell_buffer.len() + padding) as u32);
            let mut cell_bin : Vec<u8> = Vec::new();

            cell_bin.append(&mut (cell.boxes.len() as u32).to_le_bytes().to_vec());

            for col_box in cell.boxes.iter() {
                cell_bin.append(&mut col_box.x_offset.to_le_bytes().to_vec());
                cell_bin.append(&mut col_box.y_offset.to_le_bytes().to_vec());
                cell_bin.append(&mut col_box.width.to_le_bytes().to_vec());
                cell_bin.append(&mut col_box.height.to_le_bytes().to_vec());
                cell_bin.append(&mut col_box.box_type.to_le_bytes().to_vec());
            }

            cell_bin.append(&mut cell.sprite_info.x_offset.to_le_bytes().to_vec());
            cell_bin.append(&mut cell.sprite_info.y_offset.to_le_bytes().to_vec());
            cell_bin.append(&mut cell.sprite_info.unk.to_le_bytes().to_vec());
            cell_bin.append(&mut cell.sprite_info.index.to_le_bytes().to_vec());

            game_object_cell_buffer.append(&mut cell_bin);
        };

        (0..padding / 4).for_each(|_| game_object_cell_pointers.push(0xFFFFFFFF));

        let padding = helpers::needed_to_align(game_object_cell_buffer.len(), 0x10);
        (0..padding).for_each(|_| game_object_cell_buffer.push(0xFF));

        game_object_pointers.push((0x10 + game_object_cell_pointers.len() * 4 + game_object_cell_buffer.len()) as u32);

        let mut game_object_sprite_pointers: Vec<u32> = Vec::new();
        let mut game_object_sprite_buffer = Vec::new();

        let padding = helpers::needed_to_align(game_object.sprite_array.sprite_entries.len() * 4, 0x10);

        for sprite in game_object.sprite_array.sprite_entries.iter() {
            game_object_sprite_pointers.push(
                (game_object.sprite_array.sprite_entries.len() * 4 + game_object_sprite_buffer.len() + padding) as u32);
            let vec32 = &sprite.data;
            let mut byte_array = unsafe { vec32.align_to::<u8>().1 }.to_vec();
            game_object_sprite_buffer.append(&mut byte_array);

            let padding = helpers::needed_to_align(game_object_sprite_buffer.len(), 0x10);
            (0..padding).for_each(|_| game_object_sprite_buffer.push(0xFF));
        };

        (0..padding / 4).for_each(|_| game_object_sprite_pointers.push(0xFFFFFFFF));

        game_object_pointers.push(
            (0x10 + game_object_cell_pointers.len() * 4 + game_object_cell_buffer.len()
                + game_object_sprite_pointers.len() * 4 + game_object_sprite_buffer.len()) as u32);

        let mut game_object_script_buffer = Vec::new();
        for action in game_object.script_data.actions.iter() {
            for instruction in action.instructions.iter() {
                let mut instruction_buffer = bincode::serialize(&instruction).unwrap();
                game_object_script_buffer.append(&mut instruction_buffer);
            }
        }

        let script_padding = if game_object_script_buffer.len() % 0x1000 > 0xA00 {
            0x1000 - game_object_script_buffer.len() % 0x1000 + 0xA00
        } else {
            0xA00 - game_object_script_buffer.len() % 0x1000
        };
        (0..script_padding).for_each(|_| game_object_sprite_buffer.push(0x00));

        obj_buffer.append(&mut unsafe { game_object_pointers.align_to::<u8>().1 }.to_vec());
        obj_buffer.append(&mut unsafe { game_object_cell_pointers.align_to::<u8>().1 }.to_vec());
        obj_buffer.append(&mut game_object_cell_buffer);
        obj_buffer.append(&mut unsafe { game_object_sprite_pointers.align_to::<u8>().1 }.to_vec());
        obj_buffer.append(&mut game_object_sprite_buffer);
        obj_buffer.append(&mut game_object_script_buffer);
    }

    obj_pointers.push((initial_offset + player_buffer.len() + obj_buffer.len()) as u32);
    (0..padding / 4).for_each(|_| obj_pointers.push(0xFFFFFFFF));

    buffer.append(&mut unsafe { obj_pointers.align_to::<u8>().1 }.to_vec());
    buffer.append(&mut player_buffer);
    buffer.append(&mut obj_buffer);
    buffer.append(&mut obj.unk_section.clone());

    buffer
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
    #[br(parse_with = until_exclusive(|&dword| dword == 0xffff))]
    pub data: Vec<u16>,
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

