pub use super::bfs2004b::{HashTable, HashTableEntry, HuffmanDictEntry, HuffmanDictNodeType};
pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use metadata_header::MetadataHeader;

mod archive_header;
mod file_header;
mod metadata_header;

/// Amount of entries in the hash table
pub const HASH_SIZE: u32 = 0x3E5;

/// File magic signature
pub const MAGIC: u32 = u32::from_le_bytes(*b"bfs1");

/// File version
pub const VERSION: u32 = 0x20070310;
