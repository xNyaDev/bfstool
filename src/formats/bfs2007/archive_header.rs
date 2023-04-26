pub use super::super::bfs2004b::ArchiveHeader;

/// Bfs2007-specific tests
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
        let test_file = File::open("test_data/bfs2007/fouc_data.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: super::super::MAGIC,
                version: super::super::VERSION,
                header_end: 0x50268,
                file_count: 9567,
            }
        );

        Ok(())
    }
}
