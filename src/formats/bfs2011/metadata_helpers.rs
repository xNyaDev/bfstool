use super::HashTable;

pub use crate::formats::bfs2004b::metadata_helpers::calculate_metadata_count;

/// Calculate where does the metadata section start as an absolute offset
pub fn calculate_metadata_start(hash_table: &HashTable) -> u32 {
    hash_table.entries.len() as u32 * 8 + 24
}
