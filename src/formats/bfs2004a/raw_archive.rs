use binrw::BinRead;

use crate::formats::bfs2004a::{ArchiveHeader, FileHeader, FileHeaderOffsetTable, HashTable};

/// Raw archive contents that can be read directly from a .bfs file or written to one
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct RawArchive {
    /// The archive header
    pub archive_header: ArchiveHeader,
    /// Offsets for every file header
    #[br(count = archive_header.file_count)]
    pub file_header_offsets: FileHeaderOffsetTable,
    /// Stores information about the hash size and how many files with specific hash are there
    pub hash_table: HashTable,
    /// All [FileHeader]s
    #[br(count = archive_header.file_count)]
    pub file_headers: Vec<FileHeader>,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use pretty_assertions::assert_eq;

    use crate::formats::bfs2004a::*;

    use super::*;

    #[test]
    fn parsing_test_europe() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004a/europe.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader);

        let mut expected_result_hash_table_entries = Vec::new();

        for _ in 0..HASH_SIZE {
            expected_result_hash_table_entries.push(HashTableEntry::default());
        }
        expected_result_hash_table_entries[275].file_count = 1;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            RawArchive {
                archive_header: ArchiveHeader {
                    magic: MAGIC,
                    version: VERSION,
                    header_end: 0xFDB,
                    file_count: 1,
                },
                file_header_offsets: vec![0xFAC],
                hash_table: HashTable {
                    hash_size: HASH_SIZE,
                    entries: expected_result_hash_table_entries
                },
                file_headers: vec![FileHeader {
                    flags: 0x05,
                    file_copies: 0,
                    data_offset: 0xFDC,
                    unpacked_size: 0x44F,
                    packed_size: 0x1D7,
                    crc32: 0xF6260C6E,
                    file_name_length: 0x19,
                    file_name: "data/language/version.ini".to_string(),
                    file_copies_offsets: vec![],
                }],
            }
        );
        Ok(())
    }

    #[test]
    fn parsing_test_common1() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004a/common1.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x101DD,
                file_count: 1116,
            }
        );

        assert_eq!(result.file_header_offsets[0], 0x54E7);
        assert_eq!(result.file_header_offsets[1115], 0xA43C);

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                starting_index: 0x0,
                file_count: 1,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                starting_index: 0x45B,
                file_count: 1,
            }
        );

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x101E0,
                unpacked_size: 0x918,
                packed_size: 0x1B7,
                crc32: 0x99ED26DC,
                file_name_length: 24,
                file_name: "data/drivers/aiprof1.ini".to_string(),
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[1115],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x1E905F1B,
                unpacked_size: 0xAB38,
                packed_size: 0x1D14,
                crc32: 0x5935B45,
                file_name_length: 28,
                file_name: "data/menu/tracks/winter3.dds".to_string(),
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_ps2() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004a/ps2_flatout.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x322CD,
                file_count: 3003,
            }
        );

        assert_eq!(result.file_header_offsets[0], 0x234C8);
        assert_eq!(result.file_header_offsets[3002], 0x2EAD0);

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                starting_index: 0x0,
                file_count: 3,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                starting_index: 0xBB6,
                file_count: 5,
            }
        );

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x322D0,
                unpacked_size: 0x2ED3,
                packed_size: 0x74D,
                crc32: 0xB0A39016,
                file_name_length: 18,
                file_name: "data/sound/sfx.ini".to_string(),
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[3002],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x1385B89F,
                unpacked_size: 0x56500,
                packed_size: 0x4F685,
                crc32: 0xA1D69229,
                file_name_length: 54,
                file_name: "data/tracks/winter/winter2/c/lighting/lightmap1_w2.tm2".to_string(),
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_xbox() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004a/xbox_flatout.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x3486E,
                file_count: 3142,
            }
        );

        assert_eq!(result.file_header_offsets[0], 0x831F);
        assert_eq!(result.file_header_offsets[3141], 0x30EB6);

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                starting_index: 0x0,
                file_count: 2,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                starting_index: 0xC44,
                file_count: 2,
            }
        );

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x34870,
                unpacked_size: 0x2ED3,
                packed_size: 0x74D,
                crc32: 0,
                file_name_length: 18,
                file_name: "data/sound/sfx.ini".to_string(),
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[3141],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x3D152136,
                unpacked_size: 0x5555F0,
                packed_size: 0x325C05,
                crc32: 0,
                file_name_length: 54,
                file_name: "data/tracks/winter/winter2/c/lighting/lightmap1_w2.dds".to_string(),
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }
}
