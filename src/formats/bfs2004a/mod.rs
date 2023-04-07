use std::fs::File;
use std::io::{BufRead, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{fs, io};

use binrw::BinRead;

pub use archive_header::ArchiveHeader;
pub use file_header::FileHeader;
pub use hash_table::HashTable;
pub use hash_table_entry::HashTableEntry;

use crate::archive_reader::ArchiveReader;
use crate::compression::extract_data;
use crate::{ArchivedFileInfo, CompressionMethod};

mod archive_header;
mod file_header;
mod hash_table;
mod hash_table_entry;

/// Amount of entries in the hash table
pub const HASH_SIZE: usize = 0x3E5;

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

impl<R: BufRead + Seek> ArchiveReader for ReadArchive<R> {
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
                    Some(ArchivedFileInfo {
                        offset: file_header.data_offset as usize,
                        compression_method: if file_header.flags & 0x01 == 0x01 {
                            CompressionMethod::Zlib
                        } else {
                            CompressionMethod::None
                        },
                        size: file_header.unpacked_size as usize,
                        compressed_size: file_header.packed_size as usize,
                        copies: file_header.file_copies as usize,
                        hash: if file_header.flags & 0x04 == 0x04 {
                            Some(file_header.crc32)
                        } else {
                            None
                        },
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_files(&mut self, file_names: Vec<String>, folder_name: &Path) -> io::Result<()> {
        fs::create_dir_all(folder_name)?;

        self.raw_archive
            .file_headers
            .iter_mut()
            .try_for_each(|file_header| {
                if file_names.contains(&file_header.file_name) {
                    let file_path = PathBuf::from(&file_header.file_name);
                    fs::create_dir_all(
                        folder_name.join(file_path.parent().unwrap_or(Path::new(""))),
                    )?;
                    self.reader
                        .seek(SeekFrom::Start(file_header.data_offset as u64))?;
                    let method = if file_header.flags & 0x01 == 0x01 {
                        CompressionMethod::Zlib
                    } else {
                        CompressionMethod::None
                    };
                    let mut output_file = File::create(folder_name.join(file_path))?;
                    extract_data(
                        &mut self.reader,
                        &mut output_file,
                        file_header.packed_size as u64,
                        method,
                    )?;
                }
                Ok::<(), io::Error>(())
            })?;
        Ok(())
    }
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
                    hash_size: HASH_SIZE as u32,
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
