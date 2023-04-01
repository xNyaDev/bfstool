use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, Seek};

use binrw::BinRead;

use crate::display::{ascii_value, spaced_hex};
use crate::formats::*;
use crate::ArchivedFileInfo;

/// An archive type must implement ArchiveReader to be readable
pub trait ArchiveReader {
    /// Returns file count of the archive
    fn file_count(&self) -> u64;
    /// Returns file names of all files in the archive
    fn file_names(&self) -> Vec<String>;
    /// Returns ArchivedFileInfo for the given file name, if any
    ///
    /// If there are multiple files with the same name, all of them are returned
    fn file_info(&self, file_name: &str) -> Vec<ArchivedFileInfo>;
}

/// Read an archive with the provided format, returning an ArchiveReader impl
///
/// If `force` is true then Magic / Version / Hash size check are skipped
pub fn read_archive<R: BufRead + Seek>(
    archive: &mut R,
    archive_format: Format,
    force: bool,
) -> Result<Box<dyn ArchiveReader>, ReadError> {
    match archive_format {
        Format::Bfs2004a => match bfs2004a::RawArchive::read(archive) {
            Ok(archive) => {
                if archive.archive_header.magic != bfs2004a::MAGIC && !force {
                    Err(ReadError::InvalidMagic {
                        expected: bfs2004a::MAGIC,
                        got: archive.archive_header.magic,
                    })
                } else if archive.archive_header.version != bfs2004a::VERSION && !force {
                    Err(ReadError::InvalidVersion {
                        expected: bfs2004a::VERSION,
                        got: archive.archive_header.version,
                    })
                } else if archive.hash_table.hash_size != bfs2004a::HASH_SIZE as u32 && !force {
                    Err(ReadError::InvalidHashSize {
                        expected: bfs2004a::HASH_SIZE as u32,
                        got: archive.hash_table.hash_size,
                    })
                } else {
                    Ok(Box::new(archive))
                }
            }
            Err(error) => match error {
                binrw::Error::Io(io_error) => Err(ReadError::IoError(io_error)),
                error => Err(ReadError::ParsingError(error.to_string())),
            },
        },
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
