use crate::util::{AsBytes, u32_from_le_bytes};

#[derive(Default)]
pub struct FileNameTableHeader {
    /// Offset at which `FileHeaders` start
    ///
    /// Offset from `FileNameTableHeader` start
    pub file_headers_offset: u32,

    /// A pointer to a `Vec<u32>` with offsets for every file name
    ///
    /// Offset from `FileNameTableHeader` start
    ///
    /// The offsets in the vec are offset from `FileNameTableHeader.huffman_data_offset`
    pub file_offset_table_offset: u32,

    /// A pointer to a `Vec<u16>` with sizes of encoded strings for every name
    ///
    /// Offset from `FileNameTableHeader` start
    pub file_name_size_table_offset: u32,

    /// A pointer to `Vec<HuffmanTreeEntry>`, the huffman tree for filenames
    ///
    /// Offset from `FileNameTableHeader` start
    pub huffman_tree_offset: u32,

    /// Offset at which the actual encoded data for the huffman tree starts
    ///
    /// Offset from `FileNameTableHeader` start
    pub huffman_data_offset: u32,
}

impl AsBytes for FileNameTableHeader {
    const BYTE_COUNT: usize = 20;

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut bytes = bytes;
        Self {
            file_headers_offset: u32_from_le_bytes(&mut bytes),
            file_offset_table_offset: u32_from_le_bytes(&mut bytes),
            file_name_size_table_offset: u32_from_le_bytes(&mut bytes),
            huffman_tree_offset: u32_from_le_bytes(&mut bytes),
            huffman_data_offset: u32_from_le_bytes(&mut bytes),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.file_headers_offset.to_le_bytes());
        result.extend_from_slice(&self.file_offset_table_offset.to_le_bytes());
        result.extend_from_slice(&self.file_name_size_table_offset.to_le_bytes());
        result.extend_from_slice(&self.huffman_tree_offset.to_le_bytes());
        result.extend_from_slice(&self.huffman_data_offset.to_le_bytes());
        result
    }
}