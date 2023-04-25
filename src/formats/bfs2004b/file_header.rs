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
    ///
    /// Unofficial flags:
    /// - `0x08` - compression method is Zstandard (zstd) - [Sewer56's FlatOut 2 Mod Loader](https://github.com/Sewer56/FlatOut2.Utils.ModLoader/blob/main/FlatOut2.Utils.ModLoader/Patches/Compression/SupportCustomCompressionPatch.cs)
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
                if file_header.flags & 0x08 == 0x08 {
                    CompressionMethod::Zstd
                } else {
                    CompressionMethod::Zlib
                }
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
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x11F50))?;

        let result = FileHeader::read(&mut test_reader);

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

        Ok(())
    }

    #[test]
    fn parsing_test_file_copies() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/xbox_flatout2.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x1B9A0))?;

        let result = FileHeader::read(&mut test_reader);

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

        Ok(())
    }
}
