use crate::{helpers::{pad_to_nearest, pad_to_nearest_with_excess, RGBAColor}, Error};

/// Trait that parses a type from some binary data.
/// This is a wrapper over [`BinRead`] which accepts all types that implement `AsRef<[u8]>`
pub trait ParseFromBytes: Sized  {
    /// Parse a type from a slice of bytes
    fn parse<R: AsRef<[u8]>>(bytes: &R) -> Result<Self, Error>;
}

/// Trait implemented by types that can be rebuilt into a vector of bytes
pub trait Rebuild {
    /// Rebuild the type into a [`Vec`] of bytes
    fn to_bytes(&self) -> Vec<u8>;
}

pub(crate) trait Pac {
    const MAGIC_FPAC: &'static [u8; 4] = b"FPAC";
    /// Dictates what the a metadata entry is aligned to.
    /// e.g. `0x4` will add padding so that the entry is aligned to the nearest `0x4`
    const META_ENTRY_ALIGNMENT: usize;
    /// The total size of the PAC header, doesn't include entries
    const HEADER_SIZE: usize;
    /// The size of the entry metadata NOT counting the string size
    const META_ENTRY_FIXED_SIZE: usize;
    /// Dictates what the contents of each file is aligned to.
    /// e.g. `0x4` will pad the file contents so that the entry is aligned to the nearest `0x4`
    const DATA_ALIGNMENT: usize;
    /// Decides if the type needs excess padding on the entries.
    /// this means if data is already aligned, it will add more alignment padding
    const EXCESS_PADDING: bool;
    /// The amount of entries contained in the PAC
    fn entry_count(&self) -> usize;
    /// Needed size for fixed-length entry strings
    fn string_size(&self) -> usize;
    /// Total size of the whole PAC file, header + entry metadata + data section size
    fn total_file_size(&self) -> usize;
    /// Total size of the file entry metadata
    fn entry_section_size(&self) -> usize {
        let entry_size_unaligned = Self::META_ENTRY_FIXED_SIZE + self.string_size();
        let entry_size = if Self::EXCESS_PADDING {
            pad_to_nearest_with_excess(entry_size_unaligned, Self::META_ENTRY_ALIGNMENT)
        } else {
            pad_to_nearest(entry_size_unaligned, Self::META_ENTRY_ALIGNMENT)
        };
        entry_size * self.entry_count()
    }
    /// Gets the offset where the actual data begins
    fn data_start(&self) -> usize {
        Self::HEADER_SIZE + self.entry_section_size()
    }
}

pub(crate) trait JonBin {
    const MAGIC_JONB: &'static [u8; 4] = b"JONB";
    const STRING_SIZE: usize = 0x20;
}

pub trait Palette {
    fn get_palette(&self) -> Vec<RGBAColor>;
    /// returns an `RgbaImage` of the palette.
    /// Can panic if `get_palette` returns a vec with 0 colors
    fn get_palette_bytes(&self) -> Vec<u8> {
        let palette = self.get_palette();

        let raw_palette = palette.iter().fold(Vec::new(), |mut pal, color| {
            pal.extend(color.to_rgba_slice());
            pal
        });

        raw_palette
    }
}
