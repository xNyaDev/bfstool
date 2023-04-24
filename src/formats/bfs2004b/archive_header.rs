pub use super::super::bfs2004a::ArchiveHeader;

/// Bfs2004b-specific tests
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use binrw::BinRead;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

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

        Ok(())
    }
}
