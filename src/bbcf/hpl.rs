use crate::helpers;
use crate::Error;
use crate::RGBAColor;

use nom::bytes::complete::tag;
use nom::number::complete::le_u16;
use nom::number::complete::le_u32;
use nom::IResult;
use serde::{Deserialize, Serialize};

static MAGIC_HPL: &[u8] = b"HPAL";

#[derive(Serialize, Deserialize)]
pub struct BBCFHpl {
    pub version: u32,
    pub unknown_data: (u32, u32, u16, u16),
    pub palette: Vec<RGBAColor>,
}

impl BBCFHpl {
    pub fn parse(input: &[u8]) -> Result<BBCFHpl, Error> {
        let res = parse_hpl_impl(input);

        match res {
            Ok((i, hpl)) => {
                helpers::slice_consumed(i)?;
                Ok(hpl)
            }
            Err(e) => Err(Error::Parser(e.map_input(|_| &[0; 0]).to_string())),
        }
    }
}

fn parse_hpl_impl(i: &[u8]) -> IResult<&[u8], BBCFHpl> {
    let (i, _) = tag(MAGIC_HPL)(i)?;

    let (i, version) = le_u32(i)?;
    let (i, _file_size) = le_u32(i)?;
    let (i, palette_size) = le_u32(i)?;

    let (i, unknown_1) = le_u32(i)?;
    let (i, unknown_2) = le_u32(i)?;
    let (i, unknown_3) = le_u16(i)?;
    let (i, unknown_4) = le_u16(i)?;

    let unknown = (unknown_1, unknown_2, unknown_3, unknown_4);

    let (i, _padding) = le_u32(i)?;

    let (i, palette) = parse_palette(i, palette_size)?;

    let hpl = BBCFHpl {
        version,
        unknown_data: unknown,
        palette,
    };

    Ok((i, hpl))
}

fn parse_palette(i: &[u8], size: u32) -> IResult<&[u8], Vec<RGBAColor>> {
    nom::multi::count(helpers::parse_bgra, size as usize)(i)
}

impl BBCFHpl {
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{WriteBytesExt, LE};
        use std::io::Write;

        // BBCF palette file structure
        // 00: magic "HPAL"
        // 04: version?
        // 08: total file size
        // 0C: palette size
        // 10: unknown u32, u32, u16, u16
        // 1C: 4-byte padding
        // 20..N: palette of BGRA8 colors

        const HEADER_SIZE: usize = 0x20;

        let mut final_bytes = Vec::new();

        final_bytes.write_all(MAGIC_HPL).unwrap();
        final_bytes.write_u32::<LE>(self.version).unwrap();
        final_bytes
            .write_u32::<LE>((HEADER_SIZE + (self.palette.len() * 0x4)) as u32)
            .unwrap();
        final_bytes
            .write_u32::<LE>(self.palette.len() as u32)
            .unwrap();

        // unknown data
        final_bytes.write_u32::<LE>(self.unknown_data.0).unwrap();
        final_bytes.write_u32::<LE>(self.unknown_data.1).unwrap();
        final_bytes.write_u16::<LE>(self.unknown_data.2).unwrap();
        final_bytes.write_u16::<LE>(self.unknown_data.3).unwrap();

        // padding
        final_bytes.write_u32::<LE>(0x0).unwrap();

        // write palette
        self.palette
            .iter()
            .for_each(|p| final_bytes.extend(p.to_bgra_slice()));

        final_bytes
    }
}
