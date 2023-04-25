use std::io::{BufRead, Seek, SeekFrom};

use binrw::BinRead;

pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;
pub use huffman_dict_entry::{HuffmanDictEntry, HuffmanDictNodeType};
pub use metadata_header::MetadataHeader;
pub use raw_archive::RawArchive;

use crate::archive_reader::ReadError;
use crate::archive_reader::ReadError::{InvalidHashSize, InvalidMagic, InvalidVersion};

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

/// Checks the magic, version and hash size of the archive to ensure it's a valid Bfs2004b archive
pub fn check_archive<R: BufRead + Seek>(archive: &mut R) -> Result<(), ReadError> {
    archive.seek(SeekFrom::Start(0))?;
    let archive_header = ArchiveHeader::read(archive)?;
    if archive_header.magic != MAGIC {
        return Err(InvalidMagic {
            expected: MAGIC,
            got: archive_header.magic,
        });
    }
    if archive_header.version != VERSION {
        return Err(InvalidVersion {
            expected: VERSION,
            got: archive_header.version,
        });
    }
    let hash_size = u32::read_le(archive)?;
    if hash_size != HASH_SIZE {
        return Err(InvalidHashSize {
            expected: HASH_SIZE,
            got: hash_size,
        });
    }
    Ok(())
}
