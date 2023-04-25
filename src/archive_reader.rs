use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{fs, io};

use binrw::BinRead;

use crate::compression::extract_data;
use crate::display::{ascii_value, spaced_hex};
use crate::formats::*;
use crate::ArchivedFileInfo;

/// An archive type must implement ArchiveReader to be readable
pub trait ArchiveReader<R: BufRead + Seek> {
    /// Returns file count of the archive
    fn file_count(&self) -> u64;
    /// Returns file names of all files in the archive
    fn file_names(&self) -> Vec<String>;
    /// Returns ArchivedFileInfo for the given file name, if any
    ///
    /// If there are multiple files with the same name, all of them are returned
    fn file_info(&self, file_name: &str) -> Vec<ArchivedFileInfo>;
    /// Returns ArchivedFileInfo for the given file names as a tuple of (name, info), if present
    ///
    /// If there are multiple files with the same name, all of them are returned
    fn multiple_file_info(&self, file_names: Vec<String>) -> Vec<(String, ArchivedFileInfo)>;
    /// Returns a mutable reference to the internal reader
    fn reader(&mut self) -> &mut R;
    /// Extracts listed files from the archive to the given folder
    fn extract_files<'a>(
        &mut self,
        file_names: Vec<String>,
        folder_name: &Path,
        callback: Box<dyn Fn(&str, ArchivedFileInfo) + 'a>,
    ) -> io::Result<()> {
        let file_info = self.multiple_file_info(file_names);
        let reader = self.reader();
        file_info
            .into_iter()
            .try_for_each(|(file_name, archived_file_info)| {
                let file_path = PathBuf::from(&file_name);
                fs::create_dir_all(folder_name.join(file_path.parent().unwrap_or(Path::new(""))))?;
                let mut output_file = File::create(folder_name.join(file_path))?;

                reader.seek(SeekFrom::Start(archived_file_info.offset))?;
                extract_data(
                    reader,
                    &mut output_file,
                    archived_file_info.compressed_size,
                    archived_file_info.compression_method,
                )?;
                callback(file_name.as_ref(), archived_file_info);

                Ok(())
            })
    }
}

/// Read an archive file with the provided format, returning an ArchiveReader impl
///
/// If `force` is true then Magic / Version / Hash size check are skipped
///
/// Utility function that opens a file then calls [read_archive] on it
pub fn read_archive_file(
    archive: &PathBuf,
    archive_format: Format,
    force: bool,
) -> Result<Box<dyn ArchiveReader<BufReader<File>>>, ReadError> {
    let file = File::open(archive)?;
    let file_reader = BufReader::new(file);
    read_archive(file_reader, archive_format, force)
}

/// Read an archive with the provided format, returning an ArchiveReader impl
///
/// If `force` is true then Magic / Version / Hash size check are skipped
pub fn read_archive<R: BufRead + Seek + 'static>(
    mut archive: R,
    archive_format: Format,
    force: bool,
) -> Result<Box<dyn ArchiveReader<R>>, ReadError> {
    match archive_format {
        Format::Bfs2004a => {
            if !force {
                bfs2004a::check_archive(&mut archive)?;
            }
            archive.seek(SeekFrom::Start(0))?;
            let raw_archive = bfs2004a::RawArchive::read(&mut archive)?;
            Ok(Box::new(bfs2004a::ReadArchive {
                reader: archive,
                raw_archive,
            }))
        }
        Format::Bfs2004b => {
            if !force {
                bfs2004b::check_archive(&mut archive)?;
            }
            archive.seek(SeekFrom::Start(0))?;
            let raw_archive = bfs2004b::RawArchive::read(&mut archive)?;
            let decoded_names = bfs2004b::decode_all_names(
                &raw_archive.file_name_offset_table,
                &raw_archive.file_name_length_table,
                &raw_archive.serialized_huffman_dict,
                &raw_archive.encoded_huffman_data,
            );
            Ok(Box::new(bfs2004b::ReadArchive {
                reader: archive,
                raw_archive,
                decoded_names,
            }))
        }
        _ => todo!(),
    }
}

/// Errors that can occur while reading the archive
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadError {
    /// Archive magic does not match expected value
    InvalidMagic {
        /// Expected magic
        expected: u32,
        /// Actual magic
        got: u32,
    },
    /// Archive version does not match expected value
    InvalidVersion {
        /// Expected version
        expected: u32,
        /// Actual version
        got: u32,
    },
    /// Archive hash size does not match expected value
    InvalidHashSize {
        /// Expected hash size
        expected: u32,
        /// Actual hash size
        got: u32,
    },
    /// An IO error occurred
    IoError(io::Error),
    /// Error while parsing with binrw
    ParsingError(String),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ReadError::InvalidMagic { expected, got } => {
                let expected_bytes = expected.to_be_bytes();
                let got_bytes = got.to_be_bytes();
                write!(
                    f,
                    "Archive magic does not match - expected: {}{}, got: {}{}",
                    spaced_hex(&expected_bytes),
                    if let Some(string) = ascii_value(&expected_bytes) {
                        format!(" ({})", string)
                    } else {
                        Default::default()
                    },
                    spaced_hex(&got_bytes),
                    if let Some(string) = ascii_value(&got_bytes) {
                        format!(" ({})", string)
                    } else {
                        Default::default()
                    },
                )
            }
            ReadError::InvalidVersion { expected, got } => {
                let expected_bytes = expected.to_be_bytes();
                let got_bytes = got.to_be_bytes();
                write!(
                    f,
                    "Archive version does not match - expected: {}, got: {}",
                    spaced_hex(&expected_bytes),
                    spaced_hex(&got_bytes),
                )
            }
            ReadError::InvalidHashSize { expected, got } => {
                write!(
                    f,
                    "Archive hash size does not match - expected: {}, got: {}",
                    expected, got,
                )
            }
            ReadError::IoError(error) => {
                write!(f, "An IO error occurred: {}", error)
            }
            ReadError::ParsingError(error) => {
                write!(f, "A parsing error occurred: {}", error)
            }
        }
    }
}

impl Error for ReadError {}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        ReadError::IoError(error)
    }
}

impl From<binrw::Error> for ReadError {
    fn from(error: binrw::Error) -> Self {
        match error {
            binrw::Error::Io(io_error) => ReadError::IoError(io_error),
            error => ReadError::ParsingError(error.to_string()),
        }
    }
}
