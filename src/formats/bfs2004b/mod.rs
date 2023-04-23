use std::io::SeekFrom;

use binrw::BinRead;

pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;
pub use huffman_dict_entry::{HuffmanDictEntry, HuffmanDictNodeType};
pub use metadata_header::MetadataHeader;

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;
mod huffman_dict_entry;
mod metadata_header;
mod metadata_helpers;

/// Amount of entries in the hash table
pub const HASH_SIZE: u32 = 0x3E5;

/// File magic signature
pub const MAGIC: u32 = u32::from_le_bytes(*b"bfs1");

/// File version
pub const VERSION: u32 = 0x20040505;

/// Contains offsets of specific file names in the Huffman data
pub type FileNameOffsetTable = Vec<u32>;

/// Contains lengths of specific file names in the Huffman data
pub type FileNameLengthTable = Vec<u16>;

/// Contains the encoded Huffman dictionary
pub type HuffmanDict = Vec<HuffmanDictEntry>;

/// Contains the encoded Huffman data
pub type HuffmanData = Vec<u8>;

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
    /// Encoded Huffman dictionary
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
    pub huffman_dict: HuffmanDict,
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
    pub huffman_data: HuffmanData,
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

    use super::*;

    #[test]
    fn parsing_test() -> io::Result<()> {
        // Test data comes from europe.bfs, the entire header section
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

        assert_eq!(result.huffman_dict.len(), 81);
        assert_eq!(
            result.huffman_dict[0],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Branch,
                value: 0x24
            }
        );
        assert_eq!(
            result.huffman_dict[80],
            HuffmanDictEntry {
                node_type: HuffmanDictNodeType::Leaf,
                value: 0x69
            }
        );

        assert_eq!(result.huffman_data.len(), 42892);
        assert_eq!(result.huffman_data[0], 0xB5);
        assert_eq!(result.huffman_data[42891], 0x0);

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
}
