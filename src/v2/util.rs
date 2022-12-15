use std::collections::{HashMap, VecDeque};

use bitreader::BitReader;

use crate::util::{c_char_count, u8_from_bits};
use crate::v2::HuffmanTreeEntry;

/// A function which takes the filenames huffman tree and converts it to a map
///
/// Function empties the tree and fills the passed in map, works recursively
pub fn huffman_tree_to_map(tree: &mut VecDeque<HuffmanTreeEntry>, key: u32, map: &mut HashMap<u32, u8>) {
    if let Some(entry) = tree.pop_front() {
        if entry.node_type == 0x80 {
            // If node is a leaf node, it contains the value at given key
            map.insert(key, entry.value);
        } else {
            // If node is an internal node, we need to go one level lower
            huffman_tree_to_map(tree, (key << 1) | 1, map);
            huffman_tree_to_map(tree, key << 1, map);
        }
    }
}

/// A function which decodes the given data using Huffman coding
pub fn huffman_decode(data: Vec<u8>, map: &HashMap<u32, u8>, decoded_size: u16) -> Vec<u8> {
    let mut pattern = 1;
    let mut result = Vec::new();
    for value in data.clone() {
        for bit in 0..8 {
            // Iterate over every bit in every u8 and try matching it to the decoding map
            let single_bit = (value & (1 << bit) != 0) as u8;
            pattern <<= 1;
            pattern |= single_bit as u32;
            if let Some(decoded) = map.get(&pattern) {
                // If pattern matched add the relevant value to the result and start a new pattern
                result.push(decoded.clone());
                pattern = 1;
            }
        }
    }
    // Cut the result to the decoded_size to prevent garbage at the end
    result.truncate(decoded_size as usize);
    result
}

/// A function which encodes the given data using Huffman coding
pub fn huffman_encode(data: Vec<u8>, map: &HashMap<u8, u32>) -> Vec<u8> {
    let mut result = Vec::new();
    // Get all encoded values for the given data and iterate over them
    let values = data.iter().filter_map(
        |byte| {
            map.get(byte).cloned()
        }
    ).collect::<Vec<u32>>();
    let mut bits = VecDeque::new();
    for value in values {
        let value_bytes = Vec::from(value.to_be_bytes());
        let mut reader = BitReader::new(value_bytes.as_slice());
        // Ignore all the zeroes at the beginning and the first 1
        reader.skip(value.leading_zeros() as u64 + 1).unwrap();
        let mut value_bits = VecDeque::new();
        // Iterate over every bit in the encoded value and add it to the bits VecDeque
        while let Ok(bit) = reader.read_bool() {
            value_bits.push_back(bit);
        }
        bits.append(&mut value_bits);
    }
    // Converts the bits VecDeque to a Vec<u8>
    while !bits.is_empty() {
        result.push(u8_from_bits(&mut bits).reverse_bits());
    }
    result
}

pub fn create_huffman_tree(names: Vec<String>) -> Vec<u8> {
    let character_count = c_char_count(names);
    let mut trees = Vec::new();
    for (character, count) in character_count {
        trees.push(
            Tree {
                huffman_tree_entry: HuffmanTreeEntry {
                    node_type: 0x80,
                    value: character,
                },
                frequency: count,
                left_node: None,
                right_node: None,
            }
        );
    }
    let mut internal_node_id = 0;
    while trees.len() != 1 {
        trees.sort_unstable_by(|x, y| {
            y.frequency.cmp(&x.frequency)
        });
        let left_tree = trees.pop().unwrap();
        let right_tree = trees.pop().unwrap();
        trees.push(
            Tree {
                huffman_tree_entry: HuffmanTreeEntry {
                    node_type: 0x00,
                    value: internal_node_id,
                },
                frequency: left_tree.frequency + right_tree.frequency,
                left_node: Some(Box::from(left_tree)),
                right_node: Some(Box::from(right_tree)),
            }
        );
        internal_node_id += 1;
    }
    let mut tree = trees.get(0).unwrap().clone();
    let serialized = serialize(&tree);
    let mut index_map = HashMap::new();
    for i in (0..serialized.len()).step_by(2) {
        index_map.insert((serialized[i], serialized[i + 1]), (i / 2) as u8);
    }
    set_internal_node_values(&mut tree, &index_map);
    serialize(&tree)
}

#[derive(Clone)]
struct Tree {
    pub huffman_tree_entry: HuffmanTreeEntry,
    pub frequency: u32,
    pub left_node: Option<Box<Tree>>,
    pub right_node: Option<Box<Tree>>,
}

fn serialize(tree: &Tree) -> Vec<u8> {
    let mut serialized = Vec::new();
    serialized.push(tree.huffman_tree_entry.value);
    serialized.push(tree.huffman_tree_entry.node_type);
    if tree.huffman_tree_entry.node_type == 0 {
        let mut left_vec = serialize(&tree.left_node.as_ref().unwrap());
        let mut right_vec = serialize(&tree.right_node.as_ref().unwrap());
        serialized.append(&mut left_vec);
        serialized.append(&mut right_vec);
    }
    serialized
}

fn set_internal_node_values(tree: &mut Tree, index_map: &HashMap<(u8, u8), u8>) {
    if tree.huffman_tree_entry.node_type == 0 {
        let right_node = tree.right_node.as_ref().unwrap();
        tree.huffman_tree_entry.value = index_map.get(
            &(
                right_node.huffman_tree_entry.value,
                right_node.huffman_tree_entry.node_type
            )
        ).unwrap().clone();
        set_internal_node_values(tree.left_node.as_mut().unwrap(), &index_map);
        set_internal_node_values(tree.right_node.as_mut().unwrap(), &index_map);
    }
}