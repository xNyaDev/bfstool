use binrw::BinRead;

/// Archive Header for archive of formats: Bfs2004a, Bfs2004b
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 66 73 31`, `"bfs1"`
    pub magic: u32,
    /// File version
    ///
    /// `05 05 04 20`, v2004.05.05a and v2004.05.05b
    ///
    /// `10 03 07 20`, v2007.03.10
    pub version: u32,
    /// Offset at which the header section ends
    pub header_end: u32,
    /// Number of files in the archive
    pub file_count: u32,
}

/// Bfs2004a-specific tests
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004a/europe.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

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

        // Test data comes from common1.bfs, first 10h bytes
        let test_file = File::open("test_data/bfs2004a/common1.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x31736662,
                version: 0x20040505,
                header_end: 0x101DD,
                file_count: 1116,
            }
        );

        Ok(())
    }
}
