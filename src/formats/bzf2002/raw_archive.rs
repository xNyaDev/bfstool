use binrw::BinRead;

use super::{ArchiveHeader, FileHeader};

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

    use crate::formats::bzf2002::{MAGIC, VERSION};

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bzf2002/demo_Shader.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader);

        assert!(result.is_ok());

        let result = result.unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_size: 0x41D,
                file_count: 26,
            }
        );
        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x01,
                data_offset: 0x420,
                unpacked_size: 0x123C,
                packed_size: 0x3B8,
                crc32: 0,
                file_name_length: 16,
                file_name: "fix_car_body.sha".to_string(),
            }
        );
        assert_eq!(
            result.file_headers[25],
            FileHeader {
                flags: 0x01,
                data_offset: 0x3657,
                unpacked_size: 0x3DD,
                packed_size: 0x10C,
                crc32: 0,
                file_name_length: 17,
                file_name: "shaderlib_pro.ini".to_string(),
            }
        );

        let test_file = File::open("test_data/bzf2002/tt_Language.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            RawArchive {
                archive_header: ArchiveHeader {
                    magic: MAGIC,
                    version: VERSION,
                    header_size: 0x56,
                    file_count: 2,
                },
                file_headers: vec![
                    FileHeader {
                        flags: 0x05,
                        data_offset: 0x58,
                        unpacked_size: 0x709,
                        packed_size: 0x1BD,
                        crc32: 0xF120B349,
                        file_name_length: 12,
                        file_name: "language.ini".to_string(),
                    },
                    FileHeader {
                        flags: 0x05,
                        data_offset: 0x215,
                        unpacked_size: 0x39B0,
                        packed_size: 0xF9F,
                        crc32: 0x2215375C,
                        file_name_length: 20,
                        file_name: "language_english.txt".to_string(),
                    }
                ],
            }
        );
        Ok(())
    }
}
