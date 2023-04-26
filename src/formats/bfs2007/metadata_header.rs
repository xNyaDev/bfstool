pub use super::super::bfs2004b::MetadataHeader;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use binrw::BinRead;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/fouc_data.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x1F3C))?;

        let result = MetadataHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            MetadataHeader {
                file_headers_offset: 0x1464C,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x4800,
                huffman_dictionary_offset: 0x6BF6,
                huffman_data_offset: 0x6C98,
            }
        );

        Ok(())
    }
}
