use nom::multi::count;
use nom::number::complete::le_u32;
use nom::IResult;

use crate::archive_reader::NomParseable;
use crate::formats::bfs2004a::hash_table_entry::HashTableEntry;

/// Stores information about the hash size and how many files with specific hash are there
#[derive(Debug, Eq, PartialEq)]
pub struct HashTable {
    /// Hash size, should be equal to [`HASH_SIZE`](super::HASH_SIZE)
    pub hash_size: u32,
    /// A list of entries in the table. Vec length is `hash_size`.
    pub entries: Vec<HashTableEntry>,
}

impl NomParseable for HashTable {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, hash_size) = le_u32(input)?;
        let (input, entries) = count(HashTableEntry::parse, hash_size as usize)(input)?;
        Ok((input, Self { hash_size, entries }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsing_test() {
        use super::*;

        // Test data is made up to have one entry.
        //
        // Should not fail if hash_size is not super::HASH_SIZE, that check should be done while
        // reading the archive.
        let test_data = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];

        assert_eq!(
            HashTable::parse(&test_data),
            Ok((
                vec![].as_slice(),
                HashTable {
                    hash_size: 1,
                    entries: vec![HashTableEntry {
                        starting_index: 0,
                        file_count: 1,
                    }],
                }
            ))
        );
    }
}
