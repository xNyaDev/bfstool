pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;
pub use huffman_dict_entry::{HuffmanDictEntry, HuffmanDictNodeType};
pub use metadata_header::MetadataHeader;
pub use raw_archive::RawArchive;

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;
mod huffman_dict_entry;
mod metadata_header;
mod metadata_helpers;
mod raw_archive;

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
