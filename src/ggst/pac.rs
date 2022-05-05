use std::io::Write;

use crate::{helpers, traits::Pac, Error};

use nom::{bytes::complete::take, combinator, number::complete::le_u32, IResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GGSTPac {
    pub unknown: u32,
    pub files: Vec<GGSTPacEntry>,
}

impl Pac for GGSTPac {
    const DATA_ALIGNMENT: usize = 0x4;
    const META_ENTRY_ALIGNMENT: usize = 0x10;
    const META_ENTRY_FIXED_SIZE: usize = 0x10;
    const HEADER_SIZE: usize = 0x20;
    const EXCESS_PADDING: bool = false;

    fn entry_count(&self) -> usize {
        self.files.len()
    }

    fn string_size(&self) -> usize {
        let largest_name = self
            .files
            .iter()
            .map(|x| x.name.len())
            .max()
            .expect("no entries");

        helpers::pad_to_nearest_with_excess(largest_name, 0x20)
    }

    fn total_file_size(&self) -> usize {
        self.data_start()
            + self.files.iter().fold(0, |acc, entry| {
                let len = entry.contents.len();

                acc + helpers::pad_to_nearest(len, GGSTPac::DATA_ALIGNMENT)
            })
    }
}

#[derive(Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct GGSTPacEntry {
    pub unknown: u32,
    pub id: u32,
    pub name: String,
    #[serde(skip)]
    pub contents: Vec<u8>,
}

impl GGSTPac {
    pub fn parse(input: &[u8]) -> Result<GGSTPac, Error> {
        let res = self::parse_pac_impl(input);

        match res {
            Ok((i, pac)) => {
                helpers::slice_consumed(i)?;
                Ok(pac)
            }
            Err(e) => Err(Error::Parser(e.to_string())),
        }
    }
}

fn parse_pac_impl(i: &[u8]) -> IResult<&[u8], GGSTPac> {
    let original_input = <&[u8]>::clone(&i);
    let (i, _) = nom::bytes::complete::tag(b"FPAC")(i)?;

    let (i, data_start) = le_u32(i)?;
    let (i, _total_size) = le_u32(i)?;
    let (i, file_count) = combinator::verify(le_u32, |x| *x > 0)(i)?;
    let (i, unknown) = le_u32(i)?;
    let (i, string_size) = le_u32(i)?;
    //println!("meta done, string size: {}", string_size);

    // padding
    let (i, _) = take(8u8)(i)?;

    let (_, entries): (_, Vec<ParsedEntryMeta>) =
        nom::multi::count(|i| parse_entry(i, string_size), file_count as usize)(i)?;
    let entry_count = entries.len();
    let mut entry_iter = entries.into_iter();

    let i = &original_input[data_start as usize..];
    let (i, result_entries) = nom::multi::count(
        |i| parse_entry_contents(i, entry_iter.next().unwrap()),
        entry_count,
    )(i)?;

    Ok((
        i,
        GGSTPac {
            unknown,
            files: result_entries,
        },
    ))
}

fn parse_entry(i: &[u8], string_size: u32) -> IResult<&[u8], ParsedEntryMeta> {
    let (i, file_name) = helpers::take_str_of_size(i, string_size)?;
    //println!("parsing entry: {}", file_name.clone());
    let (i, id) = le_u32(i)?;
    let (i, _offset) = le_u32(i)?;
    let (i, size) = le_u32(i)?;
    let (i, unknown) = le_u32(i)?;

    //println!("{:#X}", _unknown);

    let needed_padding = helpers::needed_to_align((string_size + 0x10) as usize, 0x10);

    let (i, _) = helpers::take_null(i, needed_padding)?;

    let file_entry = ParsedEntryMeta {
        unknown,
        name: file_name.to_string(),
        id,
        size,
    };
    Ok((i, file_entry))
}

struct ParsedEntryMeta {
    unknown: u32,
    name: String,
    id: u32,
    size: u32,
}

fn parse_entry_contents(i: &[u8], entry: ParsedEntryMeta) -> IResult<&[u8], GGSTPacEntry> {
    //println!("Parsing contents for entry {}", entry.name);

    let (i, file_contents) = take(entry.size)(i)?;

    let padding_len = helpers::needed_to_align(entry.size as usize, 0x4);

    let (i, _) = take(padding_len)(i)?;

    Ok((
        i,
        GGSTPacEntry {
            unknown: entry.unknown,
            id: entry.id,
            name: entry.name.to_string(),
            contents: file_contents.to_vec(),
        },
    ))
}

impl GGSTPac {
    pub fn to_bytes(&self) -> Vec<u8> {
        rebuild_pac_impl(self)
    }
}

fn rebuild_pac_impl(pac: &GGSTPac) -> Vec<u8> {
    use byteorder::{WriteBytesExt, LE};

    let mut pac_file: Vec<u8> = Vec::new();

    // Write the headers to the fpac
    // contents:
    // 00 magic b"FPAC"
    // 04 data start offset
    // 08 total size
    // 0C files contained total
    // 10 unknown
    // 14 string size
    // 18..20 null padding
    // 20...N file entries

    pac_file.write_all(GGSTPac::MAGIC_FPAC).unwrap();
    pac_file.write_u32::<LE>(pac.data_start() as u32).unwrap();
    pac_file
        .write_u32::<LE>(pac.total_file_size() as u32)
        .unwrap();
    pac_file.write_u32::<LE>(pac.entry_count() as u32).unwrap();
    pac_file.write_u32::<LE>(pac.unknown).unwrap();
    pac_file.write_u32::<LE>(pac.string_size() as u32).unwrap();
    pac_file.write_u64::<LE>(0x00).unwrap();

    let string_size = pac.string_size();
    // Write file entries while also accumulating the contents of the files
    // to be added on after entries are written
    let data_section = pac
        .files
        .iter()
        .fold(Vec::<u8>::new(), |mut data_section, e| {
            let file_name = helpers::string_to_fixed_bytes(e.name.as_str(), string_size);
            let id = e.id;
            let offset = data_section.len();
            let size = e.contents.len();

            // File entry structure:
            // 00..N File name
            // N file ID
            // N+4 file data offset
            // N+8 file size
            // N+C unknown
            let mut file_entry = Vec::new();
            file_entry.write_all(&file_name).unwrap();
            file_entry.write_u32::<LE>(id).unwrap();
            file_entry.write_u32::<LE>(offset as u32).unwrap();
            file_entry.write_u32::<LE>(size as u32).unwrap();
            file_entry.write_u32::<LE>(e.unknown).unwrap();

            // Resize to include correct padding
            file_entry.resize(
                helpers::pad_to_nearest(file_entry.len(), GGSTPac::META_ENTRY_ALIGNMENT),
                0x00,
            );

            // Write entry to PAC file
            pac_file.write_all(&file_entry).unwrap();

            // Write contents to data section
            data_section.write_all(&e.contents).unwrap();

            // padding to align data
            let needed_padding =
                helpers::needed_to_align(e.contents.len(), GGSTPac::DATA_ALIGNMENT);
            (0..needed_padding).for_each(|_| data_section.write_u8(0x00).unwrap());

            data_section
        });

    pac_file.write_all(&data_section).unwrap();

    pac_file
}
