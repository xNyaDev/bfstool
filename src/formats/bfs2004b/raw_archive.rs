use std::io::SeekFrom;

use binrw::BinRead;

use crate::formats::bfs2004a::ArchiveHeader;
use crate::formats::bfs2004b::{
    EncodedHuffmanData, FileHeader, FileNameLengthTable, FileNameOffsetTable, HashTable,
    MetadataHeader, SerializedHuffmanDict,
};

use super::metadata_helpers;

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

    use crate::formats::bfs2004b::*;

    use super::*;

    #[test]
    fn parsing_test_fo2a() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/fo2a.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x37288,
                file_count: 6349,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x11F50,
                file_count: 7,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x371F8,
                file_count: 6,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0x10014,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x3AA0,
                huffman_dictionary_offset: 0x57E6,
                huffman_data_offset: 0x5888,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 3747);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[3746], 0xA77F);

        assert_eq!(result.file_name_length_table.len(), 3747);
        assert_eq!(result.file_name_length_table[0], 6);
        assert_eq!(result.file_name_length_table[3746], 13);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x24
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x69
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 42892);
        assert_eq!(result.encoded_huffman_data[0], 0xB5);
        assert_eq!(result.encoded_huffman_data[42891], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x623AD335,
                unpacked_size: 0x40000,
                packed_size: 0x12664,
                crc32: 0x487CE316,
                folder_id: 0x4F2,
                file_id: 0xB4A,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[6348],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x2F27CCFA,
                unpacked_size: 0x9187,
                packed_size: 0x2AB8,
                crc32: 0xAC3BC1F0,
                folder_id: 0x44F,
                file_id: 0xD11,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_ps2() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/ps2_flatout.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x37DFC,
                file_count: 6205,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x10450,
                file_count: 6,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x37CA0,
                file_count: 7,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0xE514,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x33D4,
                huffman_dictionary_offset: 0x4DB4,
                huffman_data_offset: 0x4E56,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 3312);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[3311], 0x96AF);

        assert_eq!(result.file_name_length_table.len(), 3312);
        assert_eq!(result.file_name_length_table[0], 20);
        assert_eq!(result.file_name_length_table[3311], 13);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x1E
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x6C
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 38590);
        assert_eq!(result.encoded_huffman_data[0], 0x42);
        assert_eq!(result.encoded_huffman_data[38589], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0x426743E,
                unpacked_size: 0x40000,
                packed_size: 0x12664,
                crc32: 0x487CE316,
                folder_id: 0x3C6,
                file_id: 0x9B4,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[6204],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0xA2F0865,
                unpacked_size: 0x8180,
                packed_size: 0xAEB,
                crc32: 0xAC86DF23,
                folder_id: 0x332,
                file_id: 0x72B,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_psp() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/foho_flatout.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x36210,
                file_count: 5909,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x10498,
                file_count: 8,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x360B0,
                file_count: 7,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0xE55C,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x33DC,
                huffman_dictionary_offset: 0x4DC0,
                huffman_data_offset: 0x4E62,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 3314);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[3313], 0x96F1);

        assert_eq!(result.file_name_length_table.len(), 3314);
        assert_eq!(result.file_name_length_table[0], 20);
        assert_eq!(result.file_name_length_table[3313], 13);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x1C
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x2E
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 38650);
        assert_eq!(result.encoded_huffman_data[0], 0x1E);
        assert_eq!(result.encoded_huffman_data[38649], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x05,
                file_copies: 0,
                data_offset: 0xE5CB4FA,
                unpacked_size: 0xCA0,
                packed_size: 0x771,
                crc32: 0x624B53E4,
                folder_id: 0x3A0,
                file_id: 0xA6B,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[5908],
            FileHeader {
                flags: 0x05,
                file_copies: 1,
                data_offset: 0xDE29682,
                unpacked_size: 0x8480,
                packed_size: 0x4509,
                crc32: 0xF4684B70,
                folder_id: 0x339,
                file_id: 0x415,
                file_copies_offsets: vec![0xDE35C3B],
            }
        );

        Ok(())
    }

    #[test]
    fn parsing_test_xbox() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2004b/xbox_flatout2.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x38734,
                file_count: 6299,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x10058,
                file_count: 8,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0x385D8,
                file_count: 7,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0xE11C,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x3364,
                huffman_dictionary_offset: 0x4D0C,
                huffman_data_offset: 0x4DAE,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 3284);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[3283], 0x9360);

        assert_eq!(result.file_name_length_table.len(), 3284);
        assert_eq!(result.file_name_length_table[0], 20);
        assert_eq!(result.file_name_length_table[3283], 13);

        assert_eq!(result.serialized_huffman_dict.len(), 81);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x2E
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x61
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 37742);
        assert_eq!(result.encoded_huffman_data[0], 0x78);
        assert_eq!(result.encoded_huffman_data[37741], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x8EB9BF7,
                unpacked_size: 0x8A,
                packed_size: 0x63,
                crc32: 0,
                folder_id: 0x3E0,
                file_id: 0x949,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[6298],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x1BD755DD,
                unpacked_size: 0x155F0,
                packed_size: 0xEBD2,
                crc32: 0,
                folder_id: 0x2FB,
                file_id: 0x67C,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }
}
