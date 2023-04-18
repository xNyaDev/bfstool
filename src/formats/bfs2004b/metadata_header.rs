use binrw::BinRead;

/// Header for the metadata section in a Bfs2004b file
///
/// All offsets here are treating the start of MetadataHeader as 0h.
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct MetadataHeader {
    /// Offset where file headers start
    pub file_headers_offset: u32,
    /// Offset where the file name offset table starts
    pub file_name_offset_table_offset: u32,
    /// Offset where the file name length table starts
    pub file_name_length_table_offset: u32,
    /// Offset where the Huffman dictionary starts
    pub huffman_dictionary_offset: u32,
    /// Offset where the Huffman data starts
    pub huffman_data_offset: u32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from fo2a.bfs, 1F3Ch-1F4Fh
        let test_data = include_bytes!("../../../test_data/bfs2004b/fo2a.bin");
        let test_data = &test_data[0x1F3C..=0x1F4F];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = MetadataHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            MetadataHeader {
                file_headers_offset: 0x10014,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x3AA0,
                huffman_dictionary_offset: 0x57E6,
                huffman_data_offset: 0x5888,
            }
        );
    }
}
