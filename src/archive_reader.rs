use std::io;
use std::io::{BufRead, Seek};

use crate::formats::Format;

/// An archive type must implement ArchiveReader to be readable
pub trait ArchiveReader {
    /// Returns file count of the archive
    fn file_count(&self) -> u64;
}

/// Read an archive with the provided format, returning an ArchiveReader impl
pub fn read_archive<R: BufRead + Seek>(
    _archive: R,
    _archive_format: Format,
) -> Box<dyn ArchiveReader> {
    todo!()
}

/// Errors that can occur while reading the archive
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
}
