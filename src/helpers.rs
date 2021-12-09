use nom::{
    bytes::complete::take,
    bytes::complete::take_until,
    combinator::{map_res, verify},
    multi,
    number::complete::le_u8,
    IResult,
};

use crate::{traits::Palette, Error};

pub fn take_str_of_size(i: &[u8], size: u32) -> IResult<&[u8], &str> {
    let (i, bytes) = take(size)(i)?;
    let (_, parsed_string) = map_res(take_until("\0"), std::str::from_utf8)(bytes)?;

    Ok((i, parsed_string))
}

/// Takes padding and verifies that all bytes taken are null/0x00
pub fn take_null(i: &[u8], amount: usize) -> IResult<&[u8], ()> {
    let verify_null_byte = verify(le_u8, |i| *i == 0);

    let (i, _) = multi::count(verify_null_byte, amount)(i)?;

    Ok((i, ()))
}

/// Turns a string into a fixed-length array of bytes, adding null bytes to meet the required length
/// Will truncate the input to the desired size if the string is too long
pub fn string_to_fixed_bytes<T: Into<String>>(string: T, size: usize) -> Vec<u8> {
    let mut bytes = string.into().bytes().collect::<Vec<u8>>();

    bytes.truncate(size);

    if bytes.len() < size {
        let needed_nulls = size - bytes.len();
        let mut nulls: Vec<u8> = vec![0x00; needed_nulls];

        bytes.append(&mut nulls);
    }

    bytes
}

#[inline]
pub fn pad_to_nearest(size: usize, step: usize) -> usize {
    let rem = size % step;

    // remove excess padding if data is already aligned
    if rem == 0 {
        size
    } else {
        size + (step - rem)
    }
}

#[inline]
pub fn pad_to_nearest_with_excess(size: usize, step: usize) -> usize {
    let rem = size % step;
    size + (step - rem)
}

#[inline]
pub fn needed_to_align(size: usize, step: usize) -> usize {
    let rem = size % step;

    // remove excess padding if data is already aligned
    if rem == 0 {
        0
    } else {
        step - rem
    }
}

#[inline]
pub fn needed_to_align_with_excess(size: usize, step: usize) -> usize {
    let rem = size % step;
    step - rem
}

#[inline]
pub fn slice_consumed(slice: &[u8]) -> Result<(), Error> {
    // if slice.len() != 0 {
    //     dbg!(slice);
    // }

    match slice.len() {
        0 => Ok(()),
        _ => Err(Error::Parser("Full slice not consumed".into())),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct RGBAColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl RGBAColor {
    pub fn to_rgba_slice(&self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
    pub fn to_argb_slice(&self) -> [u8; 4] {
        [self.alpha, self.red, self.green, self.blue]
    }
}

#[derive(Clone)]
pub struct IndexedImage {
    pub palette: Vec<RGBAColor>,
    pub image: Vec<u8>,
}

impl Palette for IndexedImage {
    fn get_palette(&self) -> Vec<RGBAColor> {
        self.palette.clone()
    }
}
