use nom::multi::count;
use nom::number::complete::{le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use crate::archive_reader::NomParseable;

/// Header for a single file in a Bfs2004a archive
#[derive(Debug, Eq, PartialEq)]
pub struct FileHeader {
    /// Flags for the archived file
    ///
    /// Official flags:
    /// - `0x01` - compressed
    /// - `0x04` - Has crc32
    pub flags: u8,
    /// How many additional copies of this file are archived
    pub file_copies: u8,
    /// Padding
    pub padding: u16,
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
    /// length is 0, the file name is generated using the offset with a .bin extension
    pub file_name_length: u16,
    /// File name
    ///
    /// In official archives, file name length can not be 0. If reading an unofficial archive and
    /// the file name length is 0, the file name is generated using the offset with a .bin extension
    pub file_name: String,
    /// Absolute offsets of all additional file copies
    pub file_copies_offsets: Vec<u32>,
}

impl NomParseable for FileHeader {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (
            input,
            (
                flags,
                file_copies,
                padding,
                data_offset,
                unpacked_size,
                packed_size,
                crc32,
                file_name_length,
            ),
        ) = tuple((le_u8, le_u8, le_u16, le_u32, le_u32, le_u32, le_u32, le_u16))(input)?;
        let (input, file_name) = if file_name_length == 0 {
            (input, format!("{:08x}.bin", data_offset))
        } else {
            let (input, bytes) = count(le_u8, file_name_length as usize)(input)?;
            (input, String::from_utf8_lossy(&bytes).to_string())
        };
        let (input, file_copies_offsets) = count(le_u32, file_copies as usize)(input)?;
        Ok((
            input,
            Self {
                flags,
                file_copies,
                padding,
                data_offset,
                unpacked_size,
                packed_size,
                crc32,
                file_name_length,
                file_name,
                file_copies_offsets,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from europe.bfs, FACh-FDDh
        let test_data = vec![
            0x05, 0x00, 0x00, 0x00, 0xDC, 0x0F, 0x00, 0x00, 0x4F, 0x04, 0x00, 0x00, 0xD7, 0x01,
            0x00, 0x00, 0x6E, 0x0C, 0x26, 0xF6, 0x19, 0x00, 0x64, 0x61, 0x74, 0x61, 0x2F, 0x6C,
            0x61, 0x6E, 0x67, 0x75, 0x61, 0x67, 0x65, 0x2F, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F,
            0x6E, 0x2E, 0x69, 0x6E, 0x69, 0x00, 0x78, 0x9C,
        ];
        assert_eq!(
            FileHeader::parse(&test_data),
            Ok((
                vec![0x00, 0x78, 0x9C].as_slice(),
                FileHeader {
                    flags: 0x05,
                    file_copies: 0,
                    padding: 0,
                    data_offset: 0xFDC,
                    unpacked_size: 0x44F,
                    packed_size: 0x1D7,
                    crc32: 0xF6260C6E,
                    file_name_length: 0x19,
                    file_name: "data/language/version.ini".to_string(),
                    file_copies_offsets: vec![],
                }
            ))
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
        assert_eq!(
            FileHeader::parse(&test_data),
            Ok((
                vec![].as_slice(),
                FileHeader {
                    flags: 0x01,
                    file_copies: 1,
                    padding: 0,
                    data_offset: 0x9FE602,
                    unpacked_size: 0xAB38,
                    packed_size: 0x92D1,
                    crc32: 0,
                    file_name_length: 0x1B,
                    file_name: "data/cars/shared/common.dds".to_string(),
                    file_copies_offsets: vec![0xD4DD3E4],
                }
            ))
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
        assert_eq!(
            FileHeader::parse(&test_data),
            Ok((
                vec![].as_slice(),
                FileHeader {
                    flags: 0x04,
                    file_copies: 0,
                    padding: 0,
                    data_offset: 0x133FB,
                    unpacked_size: 0x2A26E,
                    packed_size: 0x2A26E,
                    crc32: 0x8AF8FAD,
                    file_name_length: 0,
                    file_name: "000133fb.bin".to_string(),
                    file_copies_offsets: vec![],
                }
            ))
        );
    }
}
