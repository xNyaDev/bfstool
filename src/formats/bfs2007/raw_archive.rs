use std::io::SeekFrom;

use binrw::BinRead;

use super::{
    metadata_helpers, ArchiveHeader, EncodedHuffmanData, FileHeader, FileNameLengthTable,
    FileNameOffsetTable, HashTable, MetadataHeader, SerializedHuffmanDict,
};

/// Raw archive contents that can be read directly from a .bfs file or written to one
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct RawArchive {
    /// The archive header
    pub archive_header: ArchiveHeader,
    /// Stores information about the hash size and how many files with specific hash are there
    pub hash_table: HashTable,
    /// Header for the metadata section
    pub metadata_header: MetadataHeader,
    /// Offsets of specific file names in the Huffman data
    #[br(
        seek_before(
            SeekFrom::Start(
                metadata_helpers::calculate_metadata_start(&hash_table) as u64 +
                metadata_header.file_name_offset_table_offset as u64
            )
        ),
        count = metadata_helpers::calculate_metadata_count(
            metadata_header.file_name_offset_table_offset,
            &metadata_header,
            archive_header.header_end,
            metadata_helpers::calculate_metadata_start(&hash_table)
        )
    )]
    pub file_name_offset_table: FileNameOffsetTable,
    /// Lengths of specific file names in the Huffman data
    #[br(
        seek_before(
            SeekFrom::Start(
                metadata_helpers::calculate_metadata_start(&hash_table) as u64 +
                metadata_header.file_name_length_table_offset as u64
            )
        ),
        count = metadata_helpers::calculate_metadata_count(
            metadata_header.file_name_length_table_offset,
            &metadata_header,
            archive_header.header_end,
            metadata_helpers::calculate_metadata_start(&hash_table)
        )
    )]
    pub file_name_length_table: FileNameLengthTable,
    /// Serialized Huffman dictionary
    #[br(
        seek_before(
            SeekFrom::Start(
                metadata_helpers::calculate_metadata_start(&hash_table) as u64 +
                metadata_header.huffman_dictionary_offset as u64
            )
        ),
        count = metadata_helpers::calculate_metadata_count(
            metadata_header.huffman_dictionary_offset,
            &metadata_header,
            archive_header.header_end,
            metadata_helpers::calculate_metadata_start(&hash_table)
        )
    )]
    pub serialized_huffman_dict: SerializedHuffmanDict,
    /// Encoded Huffman data
    #[br(
        seek_before(
            SeekFrom::Start(
                metadata_helpers::calculate_metadata_start(&hash_table) as u64 +
                metadata_header.huffman_data_offset as u64
            )
        ),
        count = metadata_helpers::calculate_metadata_count(
            metadata_header.huffman_data_offset,
            &metadata_header,
            archive_header.header_end,
            metadata_helpers::calculate_metadata_start(&hash_table)
        )
    )]
    pub encoded_huffman_data: EncodedHuffmanData,
    /// All [FileHeader]s
    #[br(
        seek_before(
            SeekFrom::Start(
                metadata_helpers::calculate_metadata_start(&hash_table) as u64 +
                metadata_header.file_headers_offset as u64
            )
        ),
        count = archive_header.file_count
    )]
    pub file_headers: Vec<FileHeader>,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    use pretty_assertions::assert_eq;

    use crate::formats::bfs2007::*;

    use super::*;

    #[test]
    fn parsing_test_fouc() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/fouc_data.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x50268,
                file_count: 9567,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x16588,
                file_count: 9,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x501D8,
                file_count: 6,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0x1464C,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x4800,
                huffman_dictionary_offset: 0x6BF6,
                huffman_data_offset: 0x6C98,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 4603);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[4602], 0xD9A9);

        assert_eq!(result.file_name_length_table.len(), 4603);
        assert_eq!(result.file_name_length_table[0], 18);
        assert_eq!(result.file_name_length_table[4602], 20);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x2C
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x6C
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 55732);
        assert_eq!(result.encoded_huffman_data[0], 0xD1);
        assert_eq!(result.encoded_huffman_data[55731], 0x19);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x86B1065A,
                unpacked_size: 0xAB38,
                packed_size: 0x8749,
                crc32: 0x22434A64,
                folder_id: 0x5B8,
                file_id: 0xB83,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[9566],
            FileHeader {
                flags: 0x04,
                file_copies: 0,
                data_offset: 0xCA08A800,
                unpacked_size: 0x155F0,
                packed_size: 0x155F0,
                crc32: 0xFBE9D4BB,
                folder_id: 0x4ED,
                file_id: 0x8F5,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_x360() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/fouc_x360_data.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x4C154,
                file_count: 9156,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x14AC0,
                file_count: 9,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x4C094,
                file_count: 8,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0x12B84,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x4174,
                huffman_dictionary_offset: 0x6224,
                huffman_data_offset: 0x62C6,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 4184);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[4183], 0xC8AD);

        assert_eq!(result.file_name_length_table.len(), 4184);
        assert_eq!(result.file_name_length_table[0], 18);
        assert_eq!(result.file_name_length_table[4183], 20);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x18
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x61
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 51390);
        assert_eq!(result.encoded_huffman_data[0], 0x9C);
        assert_eq!(result.encoded_huffman_data[51389], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x37B6E244,
                unpacked_size: 0x87,
                packed_size: 0x5D,
                crc32: 0xD9C92F2F,
                folder_id: 0x482,
                file_id: 0xB97,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[9155],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0xABF10853,
                unpacked_size: 0x20034,
                packed_size: 0x48D1,
                crc32: 0x74253087,
                folder_id: 0x3D3,
                file_id: 0x789,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_srr() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2007/srr_data.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x2BD10,
                file_count: 4375,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0xFF88,
                file_count: 5,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x2BC58,
                file_count: 2,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0xE04C,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x2FE4,
                huffman_dictionary_offset: 0x47CC,
                huffman_data_offset: 0x4866,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 3060);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[3059], 0x97D1);

        assert_eq!(result.file_name_length_table.len(), 3060);
        assert_eq!(result.file_name_length_table[0], 19);
        assert_eq!(result.file_name_length_table[3059], 24);

        assert_eq!(result.serialized_huffman_dict.len(), 77);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x1A
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[76],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x68
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 38886);
        assert_eq!(result.encoded_huffman_data[0], 0x54);
        assert_eq!(result.encoded_huffman_data[38885], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x2278040,
                unpacked_size: 0x20480,
                packed_size: 0x17A14,
                crc32: 0x9C01D262,
                folder_id: 0x36F,
                file_id: 0x2A4,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[4374],
            FileHeader {
                flags: 0x05,
                file_copies: 34,
                data_offset: 0x1B30511,
                unpacked_size: 0x1CD5,
                packed_size: 0x46D,
                crc32: 0x2E925204,
                folder_id: 0x330,
                file_id: 0xB17,
                file_copies_offsets: vec![
                    0x01B400B2, 0x01B4FDC0, 0x01B5FE57, 0x01B6F9AD, 0x01B80362, 0x01B8FC97,
                    0x01B9F6E3, 0x01BAE49C, 0x01BBE495, 0x01BCD2B1, 0x01BDD467, 0x01BED231,
                    0x01BFC277, 0x01C0BF52, 0x01C1B528, 0x01C2AF96, 0x01C3AA96, 0x01C4A432,
                    0x01C59B2A, 0x01C69CD6, 0x01C7981D, 0x01C88F76, 0x01C98A07, 0x01CA8A8E,
                    0x01CB8407, 0x01CC80B1, 0x01CD7B88, 0x01CE27E2, 0x01CF20B7, 0x01D015E4,
                    0x01D10FB7, 0x01D206BC, 0x01D3033E, 0x01D3FE09,
                ],
            }
        );

        Ok(())
    }
}
