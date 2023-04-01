use binrw::BinRead;

/// Archive Header for archive of format Bfs2004a
#[derive(Debug, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 66 73 31`, `"bfs1"`
    pub magic: u32,
    /// File version
    ///
    /// `05 05 04 20`, v2004.05.05a
    pub version: u32,
    /// Offset at which the header section ends
    pub header_end: u32,
    /// Number of files in the archive
    pub file_count: u32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from europe.bfs, first 14h bytes
        let test_data = vec![
            0x62, 0x66, 0x73, 0x31, 0x05, 0x05, 0x04, 0x20, 0xDB, 0x0F, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0xAC, 0x0F, 0x00, 0x00,
        ];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = ArchiveHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x31736662,
                version: 0x20040505,
                header_end: 0xFDB,
                file_count: 1,
            }
        );
    }
}
