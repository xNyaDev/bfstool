use std::io::{BufRead, Seek, SeekFrom};

use binrw::BinRead;

pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;

use crate::archive_reader::ReadError::{InvalidHashSize, InvalidMagic, InvalidVersion};
use crate::archive_reader::{ArchiveReader, ReadError};
use crate::ArchivedFileInfo;

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;

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
}

/// Raw archive contents that can be read directly from a .bfs file or written to one
#[derive(Debug, Default, Eq, PartialEq, BinRead)]
#[brw(little)]
pub struct RawArchive {
    /// The archive header
    pub archive_header: ArchiveHeader,
    /// Offsets for every file header
    #[br(count = archive_header.file_count)]
    pub file_header_offsets: Vec<u32>,
    /// Stores information about the hash size and how many files with specific hash are there
    pub hash_table: HashTable,
    /// All [FileHeader]s
    #[br(count = archive_header.file_count)]
    pub file_headers: Vec<FileHeader>,
}

impl<R: BufRead + Seek> ArchiveReader<R> for ReadArchive<R> {
    fn file_count(&self) -> u64 {
        self.raw_archive.archive_header.file_count as u64
    }

    fn file_names(&self) -> Vec<String> {
        self.raw_archive
            .file_headers
            .iter()
            .map(|file_header| file_header.file_name.clone())
            .collect()
    }

    fn file_info(&self, file_name: &str) -> Vec<ArchivedFileInfo> {
        self.raw_archive
            .file_headers
            .iter()
            .filter_map(|file_header| {
                if file_name == file_header.file_name {
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
                if file_names.contains(&file_header.file_name) {
                    Some((
                        file_header.file_name.clone(),
                        ArchivedFileInfo::from(file_header),
                    ))
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

/// Checks the magic, version and hash size of the archive to ensure it's a valid Bfs2004a archive
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
    archive.seek(SeekFrom::Start(0x10 + archive_header.file_count as u64 * 4))?;
    let hash_size = u32::read_le(archive)?;
    if hash_size != HASH_SIZE {
        return Err(InvalidHashSize {
            expected: HASH_SIZE,
            got: hash_size,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parsing_test() {
        // Test data comes from europe.bfs, the entire header section
        let test_data = include_bytes!("../../../test_data/bfs2004a/europe.bin");

        let mut test_data_cursor = Cursor::new(test_data);

        let result = RawArchive::read(&mut test_data_cursor);

        let mut expected_result_hash_table_entries = Vec::new();

        for _ in 0..HASH_SIZE {
            expected_result_hash_table_entries.push(HashTableEntry::default());
        }
        expected_result_hash_table_entries[275].file_count = 1;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            RawArchive {
                archive_header: ArchiveHeader {
                    magic: MAGIC,
                    version: VERSION,
                    header_end: 0xFDB,
                    file_count: 1,
                },
                file_header_offsets: vec![0xFAC],
                hash_table: HashTable {
                    hash_size: HASH_SIZE,
                    entries: expected_result_hash_table_entries
                },
                file_headers: vec![FileHeader {
                    flags: 0x05,
                    file_copies: 0,
                    data_offset: 0xFDC,
                    unpacked_size: 0x44F,
                    packed_size: 0x1D7,
                    crc32: 0xF6260C6E,
                    file_name_length: 0x19,
                    file_name: "data/language/version.ini".to_string(),
                    file_copies_offsets: vec![],
                }],
            }
        );

        // Test data comes from common1.bfs, the entire header section
        let test_data = include_bytes!("../../../test_data/bfs2004a/common1.bin");

        let mut test_data_cursor = Cursor::new(test_data);

        let result = RawArchive::read(&mut test_data_cursor);

        assert!(result.is_ok());
    }
}
