pub use super::super::bfs2004a::ArchiveHeader;

/// Bfs2004b-specific tests
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinRead;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from fo2a.bfs, first 10h bytes
        let test_data = include_bytes!("../../../test_data/bfs2004b/fo2a.bin");
        let test_data = &test_data[..=0x10];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = ArchiveHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x31736662,
                version: 0x20040505,
                header_end: 0x37288,
                file_count: 6349,
            }
        );
    }
}
