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
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        // Test data comes from fo2a.bfs, 1F3Ch-1F4Fh
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x1F3C))?;

        let result = MetadataHeader::read(&mut test_reader);

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

        Ok(())
    }
}
