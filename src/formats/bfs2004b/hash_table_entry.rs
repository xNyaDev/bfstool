use binrw::BinRead;

/// A single entry in a [`HashTable`](super::HashTable)
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct HashTableEntry {
    /// Offset for file headers of files with this hash
    pub offset: u32,
    /// Number of files for this specific hash
    pub file_count: u32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from fo2a.bfs, 14h-1Bh
        let test_data = include_bytes!("../../../test_data/bfs2004b/fo2a.bin");
        let test_data = &test_data[0x14..=0x1B];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = HashTableEntry::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashTableEntry {
                offset: 0x11F50,
                file_count: 7,
            }
        );
    }
}
