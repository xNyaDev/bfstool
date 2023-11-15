use binrw::BinRead;

use crate::ArchivedFileInfo;
use crate::CompressionMethod;

/// Header for a single file in a Bzf2002 archive

#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct FileHeader {
    /// Flags for the archived file
    ///
    /// Official flags:
    /// - `0x01` - compressed
    /// - `0x04` - Has crc32
    pub flags: u8,
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
            copies: 0,
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
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bzf2002/demo_Shader.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x10))?;

        let result = FileHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
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

        let test_file = File::open("test_data/bzf2002/tt_Language.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x10))?;

        let result = FileHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                data_offset: 0x58,
                unpacked_size: 0x709,
                packed_size: 0x1BD,
                crc32: 0xF120B349,
                file_name_length: 12,
                file_name: "language.ini".to_string(),
            }
        );

        Ok(())
    }
}
