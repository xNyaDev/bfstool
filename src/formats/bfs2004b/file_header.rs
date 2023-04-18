use binrw::BinRead;

use crate::ArchivedFileInfo;
use crate::CompressionMethod;

/// Header for a single file in a Bfs2004b archive
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
    /// ID of the folder the file resides in
    pub folder_id: u16,
    /// ID of the filename
    pub file_id: u16,
    /// Absolute offsets of all additional file copies
    #[br(count = file_copies)]
    pub file_copies_offsets: Vec<u32>,
}

impl From<&FileHeader> for ArchivedFileInfo {
    fn from(file_header: &FileHeader) -> Self {
        Self {
            offset: file_header.data_offset as u64,
            compression_method: if file_header.flags & 0x01 == 0x01 {
                CompressionMethod::Zlib
            } else {
                CompressionMethod::None
            },
            size: file_header.unpacked_size as u64,
            compressed_size: file_header.packed_size as u64,
            copies: file_header.file_copies as u64,
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
        // Test data comes from fo2a.bfs, 11F50h-11F67h
        let test_data = include_bytes!("../../../test_data/bfs2004b/fo2a.bin");
        let test_data = &test_data[0x11F50..=0x11F67];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x623AD335,
                unpacked_size: 0x40000,
                packed_size: 0x12664,
                crc32: 0x487CE316,
                folder_id: 0x4F2,
                file_id: 0xB4A,
                file_copies_offsets: vec![],
            }
        );
    }

    #[test]
    fn parsing_test_file_copies() {
        // Test data comes from flatout2.bfs - Xbox, Redump (USA, Europe) (En,Fr,De,Es,It), 1B9A0h-1B9BFh
        let test_data = vec![
            0x01, 0x02, 0x00, 0x00, 0x2D, 0x90, 0x3D, 0x00, 0x50, 0x2B, 0x00, 0x00, 0x0C, 0x17,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1C, 0x03, 0xF9, 0x0A, 0xD2, 0x4D, 0x8A, 0x20,
            0x9C, 0x2F, 0xB2, 0x20,
        ];
        let mut test_data_cursor = Cursor::new(test_data);

        let result = FileHeader::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x01,
                file_copies: 2,
                data_offset: 0x3D902D,
                unpacked_size: 0x2B50,
                packed_size: 0x170C,
                crc32: 0,
                folder_id: 0x31C,
                file_id: 0xAF9,
                file_copies_offsets: vec![0x208A4DD2, 0x20B22F9C],
            }
        );
    }
}
