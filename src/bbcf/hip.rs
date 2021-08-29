use crate::{helpers, Error};

use nom::{bytes::complete::tag, number::complete::le_u32, IResult};

pub struct BBCFHip {}

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

    let (i, unk) = le_u32(i)?;
    dbg!(unk);

    let (i, file_size) = le_u32(i)?;
    dbg!(file_size);

    let (i, palette_size) = le_u32(i)?;
    dbg!(palette_size);

    if palette_size == 0 {
        let (i, x) = le_u32(i)?;
        dbg!(x);
        
        let (i, y) = le_u32(i)?;
        dbg!(y);
    }

    let (i, unk) = le_u32(i)?;
    dbg!(unk);
    let (i, unk) = le_u32(i)?;
    dbg!(unk);
    let (i, unk) = le_u32(i)?;
    dbg!(unk);
    let (i, unk) = le_u32(i)?;
    dbg!(unk);
    
    let (i, image_width) = le_u32(i)?;
    dbg!(image_width);

    let (i, image_height) = le_u32(i)?;
    dbg!(image_height);

    todo!()
}
