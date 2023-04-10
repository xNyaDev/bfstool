use crate::{ArchivedFileInfo, CompressionMethod};
use binrw::BinRead;

/// Header for a single file in a Bfs2004a archive
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct FileHeader {
    /// Flags for the archived file
    ///
    /// Official flags:
    /// - `0x01` - compressed
    /// - `0x04` - Has crc32
    pub flags: u8,
    /// How many additional copies of this file are archived
    pub file_copies: u8,
    #[br(pad_before = 0x2)]
    /// Where is the file data stored, absolute offset
    pub data_offset: u32,
    /// File size of the file after unpacking
    pub unpacked_size: u32,
    /// File size of the file in archive
    pub packed_size: u32,
    /// CRC32 of the file
    ///
    /// If flag `0x04` is set, this value contains the CRC-32/JAMCRC value of the compressed data.
    /// Otherwise it's 0.
    pub crc32: u32,
    /// Length of the file name
    ///
    /// In official archives, this can not be 0. If reading an unofficial archive and the file name
    /// length is 0, the file name will be empty and that case needs to be handled in the user's
    /// code
    pub file_name_length: u16,
    /// File name
    ///
    /// In official archives, file name length can not be 0. If reading an unofficial archive and
    /// the file name length is 0, the file name will be empty and that case needs to be handled
    /// in the user's code
    #[br(count = file_name_length, map = |bytes: Vec<u8>| { String::from_utf8_lossy(&bytes).to_string() })]
    pub file_name: String,
    /// Absolute offsets of all additional file copies
    #[br(count = file_copies)]
    pub file_copies_offsets: Vec<u32>,
}

impl From<&FileHeader> for ArchivedFileInfo {
    fn from(file_header: &FileHeader) -> Self {
        Self {
            offset: file_header.data_offset as usize,
            compression_method: if file_header.flags & 0x01 == 0x01 {
                CompressionMethod::Zlib
            } else {
                CompressionMethod::None
            },
            size: file_header.unpacked_size as usize,
            compressed_size: file_header.packed_size as usize,
            copies: file_header.file_copies as usize,
            hash: if file_header.flags & 0x04 == 0x04 {
                Some(file_header.crc32)
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from europe.bfs, FACh-FDAh
        let test_data = include_bytes!("../../../test_data/bfs2004a/europe.bin");
        let test_data = &test_data[0xFAC..=0xFDA];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0xFDC,
                unpacked_size: 0x44F,
                packed_size: 0x1D7,
                crc32: 0xF6260C6E,
                file_name_length: 0x19,
                file_name: "data/language/version.ini".to_string(),
                file_copies_offsets: vec![],
            }
        );

        // Test data comes from common1.bfs, 54E7h-551Bh
        let test_data = include_bytes!("../../../test_data/bfs2004a/common1.bin");
        let test_data = &test_data[0x54E7..=0x551B];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x87FE5A1,
                unpacked_size: 0x5A4,
                packed_size: 0x21F,
                crc32: 0xE91D1F8B,
                file_name_length: 0x1F,
                file_name: "data/shader/fix_lightmapped.sha".to_string(),
                file_copies_offsets: vec![],
            }
        );
    }

    #[test]
    fn parsing_test_file_copies() {
        // Test data comes from flatout.bfs - Xbox, Redump (Europe) (En,Fr,Es,It), A511h-A545h
        let test_data = vec![
            0x01, 0x01, 0x00, 0x00, 0x02, 0xE6, 0x9F, 0x00, 0x38, 0xAB, 0x00, 0x00, 0xD1, 0x92,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1B, 0x00, 0x64, 0x61, 0x74, 0x61, 0x2F, 0x63,
            0x61, 0x72, 0x73, 0x2F, 0x73, 0x68, 0x61, 0x72, 0x65, 0x64, 0x2F, 0x63, 0x6F, 0x6D,
            0x6D, 0x6F, 0x6E, 0x2E, 0x64, 0x64, 0x73, 0xE4, 0xD3, 0x4D, 0x0D,
        ];
        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x01,
                file_copies: 1,
                data_offset: 0x9FE602,
                unpacked_size: 0xAB38,
                packed_size: 0x92D1,
                crc32: 0,
                file_name_length: 0x1B,
                file_name: "data/cars/shared/common.dds".to_string(),
                file_copies_offsets: vec![0xD4DD3E4],
            }
        );
    }
    /// Test for unofficial archives with file name length 0
    #[test]
    fn parsing_test_file_name_length_0() {
        // Test data comes from fov3.bfs, 229Ch-22B1h
        let test_data = vec![
            0x04, 0x00, 0x00, 0x00, 0xFB, 0x33, 0x01, 0x00, 0x6E, 0xA2, 0x02, 0x00, 0x6E, 0xA2,
            0x02, 0x00, 0xAD, 0x8F, 0xAF, 0x08, 0x00, 0x00,
        ];
        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x04,
                file_copies: 0,
                data_offset: 0x133FB,
                unpacked_size: 0x2A26E,
                packed_size: 0x2A26E,
                crc32: 0x8AF8FAD,
                file_name_length: 0,
                file_name: "".to_string(),
                file_copies_offsets: vec![],
            }
        );
    }
}
