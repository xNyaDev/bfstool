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
    fn parsing_test() -> io::Result<()> {
        // Test data comes from europe.bfs, the entire header section
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

        // Test data comes from common1.bfs, the entire header section
        let test_file = File::open("test_data/bfs2004a/common1.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader);

        assert!(result.is_ok());

        Ok(())
    }
}
