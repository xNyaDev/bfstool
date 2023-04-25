use std::collections::HashMap;

use bitvec::prelude::*;

use crate::formats::bfs2004b::{
    EncodedHuffmanData, FileNameLengthTable, FileNameOffsetTable, HuffmanDictNodeType,
    SerializedHuffmanDict,
};

/// Contains the deserialized Huffman dictionary
type HuffmanDict = HashMap<u32, u8>;

/// Decode all Huffman-encoded names
pub fn decode_all_names(
    file_name_offset_table: &FileNameOffsetTable,
    file_name_length_table: &FileNameLengthTable,
    serialized_huffman_dict: &SerializedHuffmanDict,
    encoded_huffman_data: &EncodedHuffmanData,
) -> Vec<String> {
    let dict = deserialize_huffman_dict(serialized_huffman_dict);

    let mut next_offset_iter = file_name_offset_table.iter();
    next_offset_iter.next();

    file_name_offset_table
        .iter()
        .zip(file_name_length_table.iter())
        .map(|(offset, length)| {
            let encoded_data = match next_offset_iter.next() {
                None => &encoded_huffman_data[(*offset as usize)..],
                Some(next_offset) => {
                    &encoded_huffman_data[(*offset as usize)..(*next_offset as usize)]
                }
            };
            let decoded_data = decode_huffman_data(encoded_data, &dict, *length);
            String::from_utf8_lossy(&decoded_data).to_string()
        })
        .collect()
}

/// Deserialize a Huffman dictionary
fn deserialize_huffman_dict(serialized: &SerializedHuffmanDict) -> HuffmanDict {
    let mut result = HuffmanDict::new();
    let mut deserialize_queue = Vec::new();
    let mut deserialize_single =
        |(key, position): (u32, u8), deserialize_queue: &mut Vec<(u32, u8)>| {
            if let Some(entry) = serialized.get(position as usize) {
                match entry.node_type {
                    HuffmanDictNodeType::Branch => {
                        deserialize_queue.push(((key << 1) | 1, position + 1));
                        deserialize_queue.push((key << 1, entry.value));
                    }
                    HuffmanDictNodeType::Leaf => {
                        result.insert(key, entry.value);
                    }
                }
            }
        };
    deserialize_single((1, 0), &mut deserialize_queue);
    while let Some(queued_item) = deserialize_queue.pop() {
        deserialize_single(queued_item, &mut deserialize_queue);
    }
    result
}

/// Decode some Huffman data with the given length
fn decode_huffman_data(encoded_data: &[u8], dict: &HuffmanDict, data_length: u16) -> Vec<u8> {
    let mut pattern = 1;
    let bits = encoded_data.view_bits::<Lsb0>();

    bits.iter()
        .filter_map(|bit| {
            pattern = (pattern << 1) | *bit as u32;
            dict.get(&pattern).map(|&decoded| {
                pattern = 1;
                decoded
            })
        })
        .take(data_length as usize)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::{BufRead, BufReader, Read};

    use binrw::BinRead;
    use pretty_assertions::assert_eq;

    use crate::formats::bfs2004b::RawArchive;

    use super::*;

    #[test]
    fn decode_all_names_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let archive = RawArchive::read(&mut test_reader).unwrap();

        let result = decode_all_names(
            &archive.file_name_offset_table,
            &archive.file_name_length_table,
            &archive.serialized_huffman_dict,
            &archive.encoded_huffman_data,
        );

        let expected_result_file = File::open("test_data/bfs2004b/fo2a_decoded_names.txt")?;
        let expected_result_reader = BufReader::new(expected_result_file);
        let expected_result = expected_result_reader
            .lines()
            .filter_map(|line| {
                let line = line.unwrap();
                if line.trim().is_empty() {
                    None
                } else {
                    Some(line)
                }
            })
            .collect::<Vec<String>>();

        assert_eq!(result, expected_result);

        Ok(())
    }

    #[test]
    fn deserialize_huffman_dict_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let archive = RawArchive::read(&mut test_reader).unwrap();
        let result = deserialize_huffman_dict(&archive.serialized_huffman_dict);

        assert_eq!(
            result,
            HuffmanDict::from([
                (0x0E, b'd'),
                (0x0F, b'a'),
                (0x13, b's'),
                (0x15, b'_'),
                (0x17, b'e'),
                (0x18, b'r'),
                (0x19, b't'),
                (0x1B, b'.'),
                (0x20, b'i'),
                (0x23, b'c'),
                (0x24, b'l'),
                (0x25, b'o'),
                (0x28, b'n'),
                (0x2D, b'g'),
                (0x42, b'b'),
                (0x44, b'm'),
                (0x45, b'u'),
                (0x52, b'w'),
                (0x58, b'h'),
                (0x68, b'p'),
                (0x69, b'/'),
                (0x6A, b'f'),
                (0x86, b'y'),
                (0x87, b'k'),
                (0xA6, b'v'),
                (0xA7, b'1'),
                (0xB3, b'2'),
                (0x165, b'3'),
                (0x1AD, b'0'),
                (0x1AF, b'4'),
                (0x2C9, b'x'),
                (0x358, b'6'),
                (0x359, b'5'),
                (0x35D, b'7'),
                (0x590, b'8'),
                (0x591, b'9'),
                (0x6B8, b'j'),
                (0xD72, b'-'),
                (0x1AE7, b'z'),
                (0x35CC, b'q'),
                (0x35CD, b' '),
            ])
        );

        Ok(())
    }

    #[test]
    fn decode_huffman_data_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let archive = RawArchive::read(&mut test_reader).unwrap();
        let dict = deserialize_huffman_dict(&archive.serialized_huffman_dict);

        let mut data = Vec::new();

        let mut data_source = archive
            .encoded_huffman_data
            .take((archive.file_name_offset_table[1] - archive.file_name_offset_table[0]) as u64);

        data_source.read_to_end(&mut data)?;

        let result = decode_huffman_data(data.as_slice(), &dict, archive.file_name_length_table[0]);

        assert_eq!(result, b"01.ogg".to_vec());

        Ok(())
    }
}
