use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, Seek};

use nom::IResult;

use crate::display::{ascii_value, spaced_hex};
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
                    "Archive magic does not match - expected: {}, got: {}",
                    spaced_hex(&expected_bytes),
                    spaced_hex(&got_bytes),
                )
            }
            ReadError::InvalidHashSize { expected, got } => {
                write!(
                    f,
                    "Archive magic does not match - expected: {}, got: {}",
                    expected, got,
                )
            }
            ReadError::IoError(error) => {
                write!(f, "An IO error occurred: {}", error)
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

/// Structs with this trait can be parsed using Nom
pub trait NomParseable: Sized {
    /// Parse the struct from a slice
    fn parse(input: &[u8]) -> IResult<&[u8], Self>;
}
