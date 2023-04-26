use binrw::BinRead;

use crate::ArchivedFileInfo;
use crate::CompressionMethod;

/// Header for a single file in a Bfs2007 archive
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct FileHeader {
    /// Flags for the archived file
    ///
    /// Official flags:
    /// - `0x01` - compressed
    /// - `0x04` - Has crc32
    pub flags: u8,
    #[br(pad_before = 0x1)]
    /// How many additional copies of this file are archived
    pub file_copies: u16,
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
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/fouc_data.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x16588))?;

        let result = FileHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x86B1065A,
                unpacked_size: 0xAB38,
                packed_size: 0x8749,
                crc32: 0x22434A64,
                folder_id: 0x5B8,
                file_id: 0xB83,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_file_copies() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/fouc_data.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x167A0))?;

        let result = FileHeader::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            FileHeader {
                flags: 0x05,
                file_copies: 1,
                data_offset: 0x11942C4,
                unpacked_size: 0x80080,
                packed_size: 0x33377,
                crc32: 0x4E724E71,
                folder_id: 0x513,
                file_id: 0x102C,
                file_copies_offsets: vec![0x6D58F4B],
            }
        );

        Ok(())
    }
}
