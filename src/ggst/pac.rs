use crate::{helpers, traits::{Pac, Rebuild}};

use binrw::{binread, io::SeekFrom, NullString};

#[binread]
#[br(little, magic = b"FPAC")]
#[derive(Clone, Debug)]
pub struct GGSTPac {
    #[br(temp)]
    data_start: u32,
    #[br(temp)]
    _total_size: u32,
    #[br(temp)]
    files_contained: u32,
    // seems to change the way filename hashes are used?
    // needs more research
    pac_style: u32,
    #[br(temp, align_after = 0x10)]
    string_size: u32,
    #[br(count = files_contained, args { inner: (string_size, data_start) })]
    pub files: Vec<GGSTPacEntry>,
}

#[binread]
#[br(little)]
#[br(import(string_size: u32, data_start: u32))]
#[derive(Clone, Debug)]
pub struct GGSTPacEntry {
    #[br(pad_size_to = string_size)]
    name: NullString,
    id: u32,
    #[br(temp)]
    file_offset: u32,
    #[br(temp)]
    file_size: u32,
    #[br(align_after = 0x10)]
    name_hash: u32,
    #[br(count = file_size, restore_position, seek_before = SeekFrom::Start((data_start + file_offset) as u64))]
    pub contents: Vec<u8>,
}

impl Pac for GGSTPac {
    const META_ENTRY_ALIGNMENT: usize = 0x10;
    const HEADER_SIZE: usize = 0x20;
    const META_ENTRY_FIXED_SIZE: usize = 0x10;
    const DATA_ALIGNMENT: usize = 0x4;
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

        helpers::pad_to_nearest_with_excess(largest_name, 0x10)
    }

    fn total_file_size(&self) -> usize {
        self.data_start()
            + self.files.iter().fold(0, |acc, entry| {
                let len = entry.contents.len();

                acc + helpers::pad_to_nearest(len, GGSTPac::DATA_ALIGNMENT)
            })
    }
}

impl Rebuild for GGSTPac {
    fn to_bytes(&self) -> Vec<u8> {
        rebuild_pac_impl(self)
    }
}

fn rebuild_pac_impl(pac: &GGSTPac) -> Vec<u8> {
    use std::io::Write;
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
    pac_file.write_u32::<LE>(pac.pac_style).unwrap();
    pac_file.write_u32::<LE>(pac.string_size() as u32).unwrap();
    pac_file.write_u64::<LE>(0x00).unwrap();

    let string_size = pac.string_size();
    // Write file entries while also accumulating the contents of the files
    // to be added on after entries are written
    let data_section = pac
        .files
        .iter()
        .fold(Vec::<u8>::new(), |mut data_section, e| {
            let file_name = helpers::string_to_fixed_bytes(e.name.to_string(), string_size);
            let id = e.id;
            let offset = data_section.len();
            let size = e.contents.len();

            // File entry structure:
            // 00..N File name
            // N file ID
            // N+4 file data offset
            // N+8 file size
            // N+C filename hash
            let mut file_entry = Vec::new();
            file_entry.write_all(&file_name).unwrap();
            file_entry.write_u32::<LE>(id).unwrap();
            file_entry.write_u32::<LE>(offset as u32).unwrap();
            file_entry.write_u32::<LE>(size as u32).unwrap();
            file_entry.write_u32::<LE>(e.name_hash).unwrap();

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
