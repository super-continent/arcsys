use std::io::Write;

use crate::{
    helpers::{self, IndexedImage, RGBAColor},
    traits::Palette,
    Error,
};

use byteorder::{WriteBytesExt, LE};
use nom::{
    bytes::complete::tag,
    number::complete::{le_u32, le_u8},
    IResult,
};
use serde::{Deserialize, Serialize};

/// A contained buffer of pixel (and possibly palette) data
/// stored within a [`BBCFHip`]
#[derive(Clone, Serialize, Deserialize)]
pub enum BBCFHipImage {
    Indexed(IndexedImage),
    /// A raw RGBA image
    Raw(Vec<RGBAColor>),
}

/// The sprite format used in Blazblue Centralfiction
#[derive(Clone, Serialize, Deserialize)]
pub struct BBCFHip {
    pub version: u32,
    pub unk_width: u32,
    pub unk_height: u32,
    pub unknown: u32,
    pub unknown_2: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub image_data: BBCFHipImage,
}

impl BBCFHip {
    pub fn parse(input: &[u8]) -> Result<BBCFHip, Error> {
        let res = self::parse_hip_impl(input);

        match res {
            Ok((i, pac)) => {
                helpers::slice_consumed(i)?;
                Ok(pac)
            }
            Err(e) => Err(Error::Parser(e.to_string())),
        }
    }
}

fn parse_hip_impl(i: &[u8]) -> IResult<&[u8], BBCFHip> {
    let (i, _) = tag("HIP\0")(i)?;

    // cant tell if this is the version but it seems consistent across all HIPs
    let (i, maybe_version) = le_u32(i)?;
    //dbg!(maybe_version);

    let (i, _file_size) = le_u32(i)?;
    //dbg!(_file_size);

    let (i, palette_size) = le_u32(i)?;
    //dbg!(palette_size);

    let (i, unk_w) = le_u32(i)?;
    let (i, unk_h) = le_u32(i)?;

    //dbg!(unk_w, unk_h);

    let (i, unk) = le_u32(i)?;
    let (i, unk2) = le_u32(i)?;

    let (i, width) = le_u32(i)?;
    let (i, height) = le_u32(i)?;

    //dbg!(crop_w, crop_h);

    let (i, x_offset) = le_u32(i)?;
    let (i, y_offset) = le_u32(i)?;

    let (i, _padding) = nom::bytes::complete::take(0x10usize)(i)?;

    let (i, image_data) = if palette_size > 0 {
        let (i, img) = parse_indexed_image(i, palette_size, width, height)?;

        // hack to deal with a couple image files having 2 null bytes at the end
        // not sure why they do that but otherwise it will throw an error from the slice not fully consumed
        let i = if i == &[0, 0] {
            nom::bytes::complete::take(2usize)(i)?.0
        } else {
            i
        };

        (i, img)
    } else {
        parse_raw_image(i, width, height)?
    };

    let image = BBCFHip {
        version: maybe_version,
        unk_width: unk_w,
        unk_height: unk_h,
        unknown: unk,
        unknown_2: unk2,
        image_width: width,
        image_height: height,
        x_offset,
        y_offset,
        image_data,
    };

    Ok((i, image))
}

fn parse_indexed_image(
    i: &[u8],
    palette_length: u32,
    width: u32,
    height: u32,
) -> IResult<&[u8], BBCFHipImage> {
    let (i, palette) = parse_palette(i, palette_length)?;
    let (i, image) = parse_index_runs(i, width, height)?;

    let indexed_image = BBCFHipImage::Indexed(IndexedImage { palette, image });

    Ok((i, indexed_image))
}

fn parse_raw_image(mut i: &[u8], width: u32, height: u32) -> IResult<&[u8], BBCFHipImage> {
    let len = width * height;
    let mut image_content = Vec::new();

    while image_content.len() != len as usize {
        let (scoped_i, color) = parse_argb(i)?;
        let (scoped_i, run) = le_u8(scoped_i)?;

        let it = (0..run).map(|_| color);
        image_content.extend(it);

        i = scoped_i;
    }

    let image = BBCFHipImage::Raw(image_content);

    Ok((i, image))
}

fn parse_palette(i: &[u8], palette_length: u32) -> IResult<&[u8], Vec<RGBAColor>> {
    let mut palette = Vec::new();

    let i = (0..palette_length).try_fold(i, |i, _| {
        let (i, palette_entry) = parse_rgba(i)?;
        palette.push(palette_entry);
        Ok(i)
    })?;

    Ok((i, palette))
}

fn parse_rgba(i: &[u8]) -> IResult<&[u8], RGBAColor> {
    let (i, red) = le_u8(i)?;
    let (i, green) = le_u8(i)?;
    let (i, blue) = le_u8(i)?;
    let (i, alpha) = le_u8(i)?;

    return Ok((
        i,
        RGBAColor {
            red,
            green,
            blue,
            alpha,
        },
    ));
}

fn parse_argb(i: &[u8]) -> IResult<&[u8], RGBAColor> {
    let (i, alpha) = le_u8(i)?;
    let (i, red) = le_u8(i)?;
    let (i, green) = le_u8(i)?;
    let (i, blue) = le_u8(i)?;

    return Ok((
        i,
        RGBAColor {
            red,
            green,
            blue,
            alpha,
        },
    ));
}

fn parse_index_runs(mut i: &[u8], width: u32, height: u32) -> IResult<&[u8], Vec<u8>> {
    let len = width * height;
    let mut contents = Vec::new();

    while contents.len() != len as usize {
        let (new_i, mut image_content) = parse_index_run(i)?;
        contents.append(&mut image_content);

        i = new_i;
    }

    Ok((i, contents))
}

fn parse_index_run(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i, index) = le_u8(i)?;
    let (i, len) = le_u8(i)?;

    let run = (0..len).map(|_| index).collect::<Vec<u8>>();

    Ok((i, run))
}

// rebuilding
const HEADER_SIZE: u32 = 0x40;

impl BBCFHip {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut image_bytes = match self.image_data.clone() {
            BBCFHipImage::Indexed(indexed) => indexed_to_run_encoded(indexed),
            BBCFHipImage::Raw(raw) => raw_to_run_encoded(raw),
        };

        let mut final_bytes = Vec::new();

        // HIP file structure
        //
        // 00: "HIP\0" magic string
        // 04: version?
        // 08: file size
        // 0C: palette size
        // 10: unknown X
        // 14: unknown Y
        // 18: unknown
        // 1C: unknown 2
        // 20: image width
        // 24: image height
        // 28: X offset?
        // 2C: Y offset?
        // 30: 16-byte padding
        // 40..N: image data

        // magic
        final_bytes.write(b"HIP\0").unwrap();

        // version
        final_bytes.write_u32::<LE>(self.version).unwrap();

        // file size
        final_bytes
            .write_u32::<LE>(HEADER_SIZE + image_bytes.len() as u32)
            .unwrap();

        // palette size
        let palette_size = if let BBCFHipImage::Indexed(i) = &self.image_data {
            i.palette.len()
        } else {
            0
        };
        final_bytes.write_u32::<LE>(palette_size as u32).unwrap();

        // unknown X and Y
        final_bytes.write_u32::<LE>(self.unk_width).unwrap();
        final_bytes.write_u32::<LE>(self.unk_height).unwrap();

        // unknown
        final_bytes.write_u32::<LE>(self.unknown).unwrap();
        final_bytes.write_u32::<LE>(self.unknown_2).unwrap();

        // image width and height
        final_bytes.write_u32::<LE>(self.image_width).unwrap();
        final_bytes.write_u32::<LE>(self.image_height).unwrap();

        // offsets
        final_bytes.write_u32::<LE>(self.x_offset).unwrap();
        final_bytes.write_u32::<LE>(self.y_offset).unwrap();

        // 0x10 padding
        final_bytes.write_u128::<LE>(0).unwrap();

        final_bytes.append(&mut image_bytes);

        final_bytes
    }
}

fn indexed_to_run_encoded(indexed: IndexedImage) -> Vec<u8> {
    // we can just grab the bytes for a raw rgba image here as the palette
    let mut palette = indexed.get_palette_bytes();

    //dbg!(&palette);

    let mut final_image = Vec::new();

    final_image.append(&mut palette);

    let mut run_length = 0;
    let mut indexes = indexed.image.into_iter().peekable();
    while let Some(i) = indexes.next() {
        run_length += 1;

        if run_length == u8::MAX {
            final_image.push(i);
            final_image.push(run_length);

            run_length = 0;
            continue;
        }

        if let Some(next) = indexes.peek() {
            if i == *next {
                continue;
            } else {
                final_image.push(i);
                final_image.push(run_length);

                run_length = 0;
                continue;
            }
        } else {
            final_image.push(i);
            final_image.push(run_length);
        }
    }

    final_image
}

fn raw_to_run_encoded(raw: Vec<RGBAColor>) -> Vec<u8> {
    let mut final_image = Vec::new();

    let mut run_length = 0;
    let mut indexes = raw.into_iter().peekable();
    while let Some(i) = indexes.next() {
        run_length += 1;

        if run_length == u8::MAX {
            final_image.extend(i.to_argb_slice());
            final_image.push(run_length);

            run_length = 0;
            continue;
        }

        if let Some(next) = indexes.peek() {
            if i == *next {
                continue;
            } else {
                final_image.extend(i.to_argb_slice());
                final_image.push(run_length);

                run_length = 0;
                continue;
            }
        } else {
            final_image.extend(i.to_argb_slice());
            final_image.push(run_length);
        }
    }

    final_image
}
