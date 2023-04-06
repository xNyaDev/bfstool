use binrw::BinRead;

/// A single entry in a [`HashTable`](super::HashTable)
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct HashTableEntry {
    /// The starting file header index with this hash
    pub starting_index: u16,
    /// Number of files for this specific hash
    pub file_count: u16,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from europe.bfs, 464h-467h
        let test_data = include_bytes!("../../../test_data/bfs2004a/europe.bin");
        let test_data = &test_data[0x464..=0x467];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = HashTableEntry::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashTableEntry {
                starting_index: 0,
                file_count: 1,
            }
        );
    }
}
