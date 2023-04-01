use binrw::BinRead;

use crate::formats::bfs2004a::hash_table_entry::HashTableEntry;

/// Stores information about the hash size and how many files with specific hash are there
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct HashTable {
    /// Hash size, should be equal to [`HASH_SIZE`](super::HASH_SIZE)
    pub hash_size: u32,
    /// A list of entries in the table. Vec length is `hash_size`.
    #[br(count = hash_size)]
    pub entries: Vec<HashTableEntry>,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data is made up to have one entry.
        //
        // Should not fail if hash_size is not super::HASH_SIZE, that check should be done while
        // reading the archive.
        let test_data = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = HashTable::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashTable {
                hash_size: 1,
                entries: vec![HashTableEntry {
                    starting_index: 0,
                    file_count: 1,
                }],
            }
        );
    }
}
