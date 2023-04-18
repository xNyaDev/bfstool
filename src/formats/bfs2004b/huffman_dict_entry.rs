use binrw::BinRead;

/// Huffman dictionary node type
///
/// A branch node contains index of the right child node
/// A leaf node contains a value at the given key
#[derive(Debug, Eq, PartialEq, BinRead)]
#[brw(little, repr = u8)]
pub enum HuffmanDictNodeType {
    /// A branch node contains index of the right child node
    Branch = 0x00,
    /// A leaf node contains a value at the given key
    Leaf = 0x80,
}

/// Serialized Huffman dictionary entry
#[derive(Debug, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct HuffmanDictEntry {
    /// Dict node type
    ///
    /// - `0x80` - Leaf node
    /// - `0x00` - Branch node
    pub node_type: HuffmanDictNodeType,

    /// Node value, depending on node type
    ///
    /// - Leaf node: Value located at the key
    /// - Branch node: Index of the right child node in the dict
    pub value: u8,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data is made up to test both node types
        let test_data = vec![0x00, 0x01];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = HuffmanDictEntry::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x01,
            }
        );

        let test_data = vec![0x80, 0x01];

        let mut test_data_cursor = Cursor::new(test_data);

        let result = HuffmanDictEntry::read(&mut test_data_cursor);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x01,
            }
        );
    }
}
