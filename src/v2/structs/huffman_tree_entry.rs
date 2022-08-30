use crate::util::AsBytes;

#[derive(Clone, Default)]
pub struct HuffmanTreeEntry {
    /// `0x80` - Leaf node
    ///
    /// `0x00` - Internal node
    ///
    /// Key is generated from traversing through internal nodes
    ///
    /// If the node is an internal node, on the left side we have node with key equal to `(current_key << 1) | 1`
    ///
    /// On the right we have node with key equal to `current_key << 1`
    ///
    /// Both of those nodes can also be leaf or internal
    pub node_type: u8,

    /// Leaf node:
    ///
    /// Value located at the key
    ///
    /// Internal node:
    ///
    /// Unique node ID
    pub value: u8,
}

impl AsBytes for HuffmanTreeEntry {
    const BYTE_COUNT: usize = 2;

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            node_type: bytes.get(1).unwrap().clone(),
            value: bytes.get(0).unwrap().clone(),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        vec![
            self.value,
            self.node_type,
        ]
    }
}