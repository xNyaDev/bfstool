use nom::number::complete::le_u16;
use nom::sequence::tuple;
use nom::IResult;

use crate::archive_reader::NomParseable;

/// A single entry in a [`HashTable`](super::HashTable)
#[derive(Debug, Eq, PartialEq)]
pub struct HashTableEntry {
    /// The starting file header index with this hash
    pub starting_index: u16,
    /// Number of files for this specific hash
    pub file_count: u16,
}

impl NomParseable for HashTableEntry {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (starting_index, file_count)) = tuple((le_u16, le_u16))(input)?;
        Ok((
            input,
            Self {
                starting_index,
                file_count,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsing_test() {
        use super::*;

        // Test data comes from europe.bfs, 464h-467h
        let test_data = vec![0x00, 0x00, 0x01, 0x00];

        assert_eq!(
            HashTableEntry::parse(&test_data),
            Ok((
                vec![].as_slice(),
                HashTableEntry {
                    starting_index: 0,
                    file_count: 1,
                }
            ))
        );
    }
}
