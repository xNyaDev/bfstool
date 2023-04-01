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

/// Raw archive contents that can be read directly from a .bfs file or written to one
pub struct RawArchive {
    /// The archive header
    pub archive_header: ArchiveHeader,
    /// Offsets for every file header
    pub file_header_offsets: Vec<u32>,
    /// Stores information about the hash size and how many files with specific hash are there
    pub hash_table: HashTable,
    /// All [FileHeader]s
    pub file_headers: Vec<FileHeader>,
}
