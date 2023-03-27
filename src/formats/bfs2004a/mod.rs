pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;

/// Amount of entries in the hash table
pub const HASH_SIZE: usize = 0x3E5;
