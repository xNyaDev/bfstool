use binrw::BinRead;

/// Archive Header for archive of format Bfs2011
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 66 73 31`, `"bfs1"`
    pub magic: u32,
    /// File version
    ///
    /// `20 12 11 20`, v2011.12.20
    pub version: u32,
    /// Offset at which the header section ends, with the highest bit set to 1
    pub header_end: u32,
    /// Number of files in the archive
    pub file_count: u32,
    /// Unknown value, always 1
    pub unk: u32,
}

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
        let test_file = File::open("test_data/bfs2011/00__ridge_racer__.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: super::super::MAGIC,
                version: super::super::VERSION,
                header_end: 0x800054E8,
                file_count: 273,
                unk: 1,
            }
        );

        Ok(())
    }
}
