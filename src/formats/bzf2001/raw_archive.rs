use binrw::BinRead;

use crate::formats::bzf2001::{ArchiveHeader, FileHeader};

/// Raw archive contents that can be read directly from a .bzf file or written to one
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct RawArchive {
    /// The archive header
    pub archive_header: ArchiveHeader,
    /// Offsets for every file header
    #[br(count = archive_header.file_count)]
    pub file_headers: Vec<FileHeader>,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use pretty_assertions::assert_eq;

    use crate::formats::bzf2001::{MAGIC, VERSION};

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bzf2001/language.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            RawArchive {
                archive_header: ArchiveHeader {
                    magic: MAGIC,
                    version: VERSION,
                    file_count: 4,
                },
                file_headers: vec![
                    FileHeader {
                        flags: 0x01,
                        data_offset: 0xE0,
                        unpacked_size: 0xF5F,
                        packed_size: 0x78D,
                        file_name: "credits.txt".to_string(),
                    },
                    FileHeader {
                        flags: 0x01,
                        data_offset: 0x86D,
                        unpacked_size: 0x705,
                        packed_size: 0x1E0,
                        file_name: "Language.ini".to_string(),
                    },
                    FileHeader {
                        flags: 0x01,
                        data_offset: 0xA4D,
                        unpacked_size: 0x212A,
                        packed_size: 0xE67,
                        file_name: "language_deutsch.txt".to_string(),
                    },
                    FileHeader {
                        flags: 0x01,
                        data_offset: 0x18B4,
                        unpacked_size: 0x1D1B,
                        packed_size: 0xD26,
                        file_name: "language_english.TXT".to_string(),
                    }
                ],
            }
        );
        Ok(())
    }
}
