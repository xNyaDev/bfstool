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
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        // Test data comes from europe.bfs, 464h-467h
        let test_file = File::open("test_data/bfs2004a/europe.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x464))?;

        let result = HashTableEntry::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashTableEntry {
                starting_index: 0,
                file_count: 1,
            }
        );

        Ok(())
    }
}
