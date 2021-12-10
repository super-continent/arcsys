use std::io::Write;

use nom::{
    bytes::complete::{tag, take},
    multi::count,
    number::complete::{le_f32, le_u16, le_u32, le_u8},
    IResult,
};
use serde::{Serialize, Deserialize};

use crate::{helpers, traits::JonBin, Error};

#[derive(Debug)]
pub struct GGSTJonBin {
    names: Vec<String>,
    version: u16,
    editor_data: Vec<Vec<u8>>,
    hurtboxes: Vec<HitBox>,
    hitboxes: Vec<HitBox>,
    unk_boxes: Vec<Vec<HitBox>>,
}

impl GGSTJonBin {
    pub fn parse(jonbin: &[u8]) -> Result<GGSTJonBin, Error> {
        match parse_jonbin_impl(jonbin) {
            Ok((i, jonbin)) => {
                // dbg!(i);
                helpers::slice_consumed(i)?;
                Ok(jonbin)
            }
            Err(e) => Err(Error::Parser(e.to_string())),
        }
    }
}

impl JonBin for GGSTJonBin {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitBox {
    pub kind: u32,
    pub rect: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    x_offset: f32,
    y_offset: f32,
    width: f32,
    height: f32,
}

const UNK_BOX_COUNT: usize = 0x54 / 2;
fn parse_jonbin_impl(i: &[u8]) -> IResult<&[u8], GGSTJonBin> {
    let (i, _) = tag(GGSTJonBin::MAGIC_JONB)(i)?;

    let (i, name_count) = le_u16(i)?;
    // dbg!(name_count);

    let (i, names) = count(|i| helpers::take_str_of_size(i, 0x20), name_count as usize)(i)?;
    // dbg!(&names);

    let (i, version) = le_u16(i)?;
    // dbg!(version);

    let (i, _null) = le_u8(i)?;

    let (i, editor_data_count) = le_u32(i)?;
    let (i, hurtbox_count) = le_u16(i)?;
    let (i, hitbox_count) = le_u16(i)?;
    // dbg!(editor_data_count);
    // dbg!(hurtbox_count);
    // dbg!(hitbox_count);

    let (i, mut unk_boxes_header) = count(le_u16, UNK_BOX_COUNT)(i)?;

    let (i, editor_data) = count(parse_editor_data, editor_data_count as usize)(i)?;

    let (i, hurtboxes) = count(parse_box, hurtbox_count as usize)(i)?;
    let (i, hitboxes) = count(parse_box, hitbox_count as usize)(i)?;

    let unkbox_count = unk_boxes_header.len();
    let (i, unk_boxes) = count(
        |i| {
            let (i, hitboxes) = count(parse_box, unk_boxes_header.remove(0) as usize)(i)?;
            Ok((i, hitboxes))
        },
        unkbox_count,
    )(i)?;

    let jonbin = GGSTJonBin {
        names: names.into_iter().map(|n| n.to_string()).collect(),
        version,
        editor_data: editor_data.into_iter().map(|x| x.to_vec()).collect(),
        hurtboxes,
        hitboxes,
        unk_boxes,
    };

    Ok((i, jonbin))
}

fn parse_editor_data(i: &[u8]) -> IResult<&[u8], &[u8]> {
    // let (i, src_rect) = parse_rect(i)?;
    // dbg!(src_rect);

    // let (i, dest_rect) = parse_rect(i)?;
    // dbg!(dest_rect);

    let (i, bytes) = take(0x50usize)(i)?;

    Ok((i, bytes))
}

fn parse_box(i: &[u8]) -> IResult<&[u8], HitBox> {
    let (i, kind) = le_u32(i)?;
    let (i, rect) = parse_rect(i)?;

    let hitbox = HitBox { kind, rect };

    Ok((i, hitbox))
}

fn parse_rect(i: &[u8]) -> IResult<&[u8], Rect> {
    let (i, x) = le_f32(i)?;
    let (i, y) = le_f32(i)?;
    let (i, w) = le_f32(i)?;
    let (i, h) = le_f32(i)?;

    Ok((
        i,
        Rect {
            x_offset: x,
            y_offset: y,
            width: w,
            height: h,
        },
    ))
}

impl GGSTJonBin {
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{WriteBytesExt, LE};

        let mut rebuilt = Vec::new();

        // GG Strive Jonbin layout
        //
        // 00 b"JONB"
        // 04 filename count
        // 06..n filenames, fixed 0x20 length string
        // n version?
        // n+2 null byte? seems to always be 0
        // n+3 u32, number of editor data blocks
        // n+7 big array of u16s for the number of boxes of each category: hurtbox, hitbox, unknown...
        // n+5F editor data blocks, each one 0x50 long.
        // next data is hurtboxes, hitboxes, and then 0x54 of u16s specifying counts of unknown hitbox types
        // hitbox layout is u32 for ID? followed by f32, f32, f32, f32
        // for x, y, width, height.
        rebuilt.write_all(Self::MAGIC_JONB).unwrap();
        rebuilt.write_u16::<LE>(self.names.len() as u16).unwrap();

        self.names.iter().for_each(|name| {
            let fixed = helpers::string_to_fixed_bytes(name, Self::STRING_SIZE);

            rebuilt.write_all(&fixed).unwrap();
        });

        rebuilt.write_u16::<LE>(self.version).unwrap();
        rebuilt.write_u8(0).unwrap();

        rebuilt
            .write_u32::<LE>(self.editor_data.len() as u32)
            .unwrap();
        rebuilt
            .write_u16::<LE>(self.hurtboxes.len() as u16)
            .unwrap();
        rebuilt.write_u16::<LE>(self.hitboxes.len() as u16).unwrap();

        self.unk_boxes
            .iter()
            .for_each(|boxes| rebuilt.write_u16::<LE>(boxes.len() as u16).unwrap());

        self.editor_data
            .iter()
            .for_each(|data| rebuilt.write_all(&data).unwrap());

        let mut write_hitbox = |hitbox: &HitBox| {
            rebuilt.write_u32::<LE>(hitbox.kind).unwrap();
            rebuilt.write_f32::<LE>(hitbox.rect.x_offset).unwrap();
            rebuilt.write_f32::<LE>(hitbox.rect.y_offset).unwrap();
            rebuilt.write_f32::<LE>(hitbox.rect.width).unwrap();
            rebuilt.write_f32::<LE>(hitbox.rect.height).unwrap();
        };

        self.hurtboxes.iter().for_each(&mut write_hitbox);
        self.hitboxes.iter().for_each(&mut write_hitbox);

        self.unk_boxes.iter().for_each(|boxes| {
            for b in boxes {
                write_hitbox(b);
            }
        });

        rebuilt
    }
}
