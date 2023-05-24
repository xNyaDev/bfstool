use std::io::{BufRead, Seek, SeekFrom};

use binrw::BinRead;

pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;
pub use huffman_dict_entry::{HuffmanDictEntry, HuffmanDictNodeType};
pub use huffman_helpers::decode_all_names;
pub use metadata_header::MetadataHeader;
pub use raw_archive::RawArchive;

use crate::archive_reader::ReadError::{InvalidHashSize, InvalidMagic, InvalidVersion};
use crate::archive_reader::{ArchiveReader, ReadError};
use crate::ArchivedFileInfo;

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;
mod huffman_dict_entry;
mod huffman_helpers;
mod metadata_header;
/// Utilities to help deserialize metadata
pub mod metadata_helpers;
mod raw_archive;

/// Amount of entries in the hash table
pub const HASH_SIZE: u32 = 0x3E5;

/// File magic signature
pub const MAGIC: u32 = u32::from_le_bytes(*b"bfs1");

/// File version
pub const VERSION: u32 = 0x20040505;

/// Archive that has been read from a .bfs file
pub struct ReadArchive<R: BufRead + Seek> {
    /// Seekable reader the archive has been read from
    pub reader: R,
    /// Raw archive contents
    pub raw_archive: RawArchive,
    /// Decoded filenames
    pub decoded_names: Vec<String>,
}

/// Contains offsets of specific file names in the Huffman data
pub type FileNameOffsetTable = Vec<u32>;

/// Contains lengths of specific file names in the Huffman data
pub type FileNameLengthTable = Vec<u16>;

/// Contains the serialized Huffman dictionary
pub type SerializedHuffmanDict = Vec<HuffmanDictEntry>;

/// Contains the encoded Huffman data
pub type EncodedHuffmanData = Vec<u8>;

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

impl<R: BufRead + Seek> ReadArchive<R> {
    /// Grab the correct file name for a given [FileHeader]
    fn file_header_to_name(&self, file_header: &FileHeader) -> String {
        format!(
            "{}/{}",
            self.decoded_names[file_header.folder_id as usize],
            self.decoded_names[file_header.file_id as usize],
        )
    }
}

impl<R: BufRead + Seek> ArchiveReader<R> for ReadArchive<R> {
    fn file_count(&self) -> u64 {
        self.raw_archive.archive_header.file_count as u64
    }

    fn file_names(&self) -> Vec<String> {
        self.raw_archive
            .file_headers
            .iter()
            .map(|file_header| self.file_header_to_name(file_header))
            .collect()
    }

    fn file_info(&self, file_name: &str) -> Vec<ArchivedFileInfo> {
        self.raw_archive
            .file_headers
            .iter()
            .filter_map(|file_header| {
                if file_name == self.file_header_to_name(file_header) {
                    Some(ArchivedFileInfo::from(file_header))
                } else {
                    None
                }
            })
            .collect()
    }

    fn multiple_file_info(&self, file_names: Vec<String>) -> Vec<(String, ArchivedFileInfo)> {
        self.raw_archive
            .file_headers
            .iter()
            .filter_map(|file_header| {
                let file_name = self.file_header_to_name(file_header);
                if file_names.contains(&file_name) {
                    Some((file_name, ArchivedFileInfo::from(file_header)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn reader(&mut self) -> &mut R {
        &mut self.reader
    }
}
