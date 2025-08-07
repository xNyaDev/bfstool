use std::io::SeekFrom;

use binrw::BinRead;

use super::{
    ArchiveHeader, EncodedHuffmanData, FileHeader, FileNameLengthTable, FileNameOffsetTable,
    HashTable, MetadataHeader, SerializedHuffmanDict, metadata_helpers,
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
            archive_header.header_end & 0x7FFFFFFF,
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
            archive_header.header_end & 0x7FFFFFFF,
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
            archive_header.header_end & 0x7FFFFFFF,
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
            archive_header.header_end & 0x7FFFFFFF,
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

    use crate::formats::bfs2011::*;

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        let test_file = File::open("test_data/bfs2011/00__ridge_racer__.bin")?;
        let mut test_reader = BufReader::new(test_file);

        let result = RawArchive::read(&mut test_reader).unwrap();

        assert_eq!(
            result.archive_header,
            ArchiveHeader {
                magic: MAGIC,
                version: VERSION,
                header_end: 0x800054E8,
                file_count: 273,
                unk: 1,
            }
        );

        assert_eq!(result.hash_table.hash_size, HASH_SIZE);
        assert_eq!(
            result.hash_table.entries[0],
            HashTableEntry {
                offset: 0x3B50,
                file_count: 1,
            }
        );
        assert_eq!(
            result.hash_table.entries[HASH_SIZE as usize - 1],
            HashTableEntry {
                offset: 0,
                file_count: 0,
            }
        );

        assert_eq!(
            result.metadata_header,
            MetadataHeader {
                file_headers_offset: 0x1C10,
                file_name_offset_table_offset: 0x14,
                file_name_length_table_offset: 0x45C,
                huffman_dictionary_offset: 0x680,
                huffman_data_offset: 0x716,
            }
        );

        assert_eq!(result.file_name_offset_table.len(), 274);
        assert_eq!(result.file_name_offset_table[0], 0x0);
        assert_eq!(result.file_name_offset_table[273], 0x14ED);

        assert_eq!(result.file_name_length_table.len(), 274);
        assert_eq!(result.file_name_length_table[0], 27);
        assert_eq!(result.file_name_length_table[273], 12);

        assert_eq!(result.serialized_huffman_dict.len(), 75);
        assert_eq!(
            result.serialized_huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x26
            }
        );
        assert_eq!(
            result.serialized_huffman_dict[74],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x64
            }
        );

        assert_eq!(result.encoded_huffman_data.len(), 5370);
        assert_eq!(result.encoded_huffman_data[0], 0xE0);
        assert_eq!(result.encoded_huffman_data[5369], 0x0);

        assert_eq!(
            result.file_headers[0],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x2105E,
                unpacked_size: 0xE7AC,
                packed_size: 0xBCC,
                crc32: 0,
                folder_id: 0x2C,
                file_id: 0x111,
                file_copies_offsets: vec![],
            }
        );
        assert_eq!(
            result.file_headers[272],
            FileHeader {
                flags: 0x01,
                file_copies: 0,
                data_offset: 0x5FB95,
                unpacked_size: 0xBEE,
                packed_size: 0x275,
                crc32: 0,
                folder_id: 0xA9,
                file_id: 0x111,
                file_copies_offsets: vec![],
            }
        );

        Ok(())
    }
}
