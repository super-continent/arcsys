use crate::helpers::pad_to_nearest;

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
    /// The amount of entries contained in the PAC
    fn entry_count(&self) -> usize;
    /// Needed size for fixed-length entry strings
    fn string_size(&self) -> usize;
    /// Total size of the whole PAC file, header + entry metadata + data section size
    fn total_file_size(&self) -> usize;
    /// Total size of the file entry metadata
    fn entry_section_size(&self) -> usize {
        let entry_size_unaligned = Self::META_ENTRY_FIXED_SIZE + self.string_size();
        let entry_size = pad_to_nearest(entry_size_unaligned, Self::META_ENTRY_ALIGNMENT);

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