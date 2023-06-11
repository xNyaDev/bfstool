use binrw::BinRead;

/// Archive Header for archive of format Bzf2001
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 62 7A 66`, `"bbzf"`
    pub magic: u32,
    /// File version
    ///
    /// `01 20 06 06`, v2001.06.06
    pub version: u32,
    /// Number of files in the archive
    pub file_count: u32,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bzf2001/language.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x667A6262,
                version: 0x06062001,
                file_count: 4,
            }
        );
        Ok(())
    }
}
