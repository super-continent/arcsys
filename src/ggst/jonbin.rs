use std::{io::Write};

use nom::{
    bytes::complete::{tag, take},
    multi::count,
    number::complete::{le_f32, le_u16, le_u32, le_u8},
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::{helpers, traits::JonBin, Error};

/// Hitbox data for GGST
#[derive(Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct GGSTJonBin {
    pub names: Vec<String>,
    pub version: u16,
    pub editor_data: Vec<Vec<u8>>,
    pub boxes: Vec<Vec<HitBox>>,
}

impl GGSTJonBin {
    pub fn parse(jonbin: &[u8], is_gbvs: bool) -> Result<GGSTJonBin, Error> {
        match parse_jonbin_impl(jonbin, is_gbvs) {
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

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct HitBox {
    pub kind: u32,
    pub rect: Rect,
    pub extra: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Rect {
    pub x_offset: f32,
    pub y_offset: f32,
    pub width: f32,
    pub height: f32,
}

const BOX_LAYER_COUNT: usize = 0x2C;
fn parse_jonbin_impl(i: &[u8], is_gbvs: bool) -> IResult<&[u8], GGSTJonBin> {
    let (i, _) = tag(GGSTJonBin::MAGIC_JONB)(i)?;

    let (i, name_count) = le_u16(i)?;
    // dbg!(name_count);

    let (i, names) = count(|i| helpers::take_str_of_size(i, 0x20), name_count as usize)(i)?;
    // dbg!(&names);

    let (i, version) = le_u8(i)?;
    // dbg!(version);
    let (i, _) = le_u8(i)?;

    let (i, _null) = le_u8(i)?;

    let (i, editor_data_count) = le_u32(i)?;
    // dbg!(editor_data_count);
    // dbg!(hurtbox_count);
    // dbg!(hitbox_count);

    let (i, mut box_layer_sizes) = version;

    let (i, editor_data) = count(parse_editor_data, editor_data_count as usize)(i)?;

    let unkbox_count = box_layer_sizes.len();
    let (i, boxes) = if !is_gbvs {
        count(
            |i| {
                let (i, hitboxes) = count(parse_box, box_layer_sizes.remove(0) as usize)(i)?;
                Ok((i, hitboxes))
            },
            unkbox_count,
        )(i)?
    } else {
        count(
            |i| {
                let (i, hitboxes) = count(parse_box_gbvs, box_layer_sizes.remove(0) as usize)(i)?;
                Ok((i, hitboxes))
            },
            unkbox_count,
        )(i)?
    };

    let jonbin = GGSTJonBin {
        names: names.into_iter().map(|n| n.to_string()).collect(),
        version,
        editor_data: editor_data.into_iter().map(|x| x.to_vec()).collect(),
        boxes: boxes,
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
    let (i, extra) = (i, None);

    let hitbox = HitBox { kind, rect, extra };

    Ok((i, hitbox))
}

fn parse_box_gbvs(i: &[u8]) -> IResult<&[u8], HitBox> {
    let (i, kind) = le_u32(i)?;
    let (i, rect) = parse_rect(i)?;
    let (i, extra) = le_u32(i)?;

    let hitbox = HitBox { kind, rect, extra: Some(extra) };

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

        self.boxes
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
            if hitbox.extra != None {
                rebuilt.write_u32::<LE>(hitbox.extra.unwrap()).unwrap();
            }
        };

        self.boxes.iter().for_each(|boxes| {
            for b in boxes {
                write_hitbox(b);
            }
        });

        rebuilt
    }
}