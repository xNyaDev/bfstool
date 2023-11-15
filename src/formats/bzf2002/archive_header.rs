use binrw::BinRead;

/// Archive Header for archive of format Bzf2002
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 7A 66 32`, `"bzf2"`
    pub magic: u32,
    /// File version
    ///
    /// `11 10 02 20`, v2002.10.11
    pub version: u32,
    /// Size of the entire header section.
    ///
    /// Needs to be aligned to the next 4 bytes, as the encryption works on `u32`s
    pub header_size: u32,
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
        let test_file = File::open("test_data/bzf2002/demo_Shader.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x32667A62,
                version: 0x20021011,
                header_size: 0x41D,
                file_count: 26,
            }
        );

        let test_file = File::open("test_data/bzf2002/tt_Language.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = ArchiveHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ArchiveHeader {
                magic: 0x32667A62,
                version: 0x20021011,
                header_size: 0x56,
                file_count: 2,
            }
        );
        Ok(())
    }
}
