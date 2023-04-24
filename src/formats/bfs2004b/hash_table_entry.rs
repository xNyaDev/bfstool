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
    use std::fs::File;
    use std::io;
    use std::io::{BufReader, Seek, SeekFrom};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);
        test_reader.seek(SeekFrom::Start(0x14))?;

        let result = HashTableEntry::read(&mut test_reader);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HashTableEntry {
                offset: 0x11F50,
                file_count: 7,
            }
        );

        Ok(())
    }
}
