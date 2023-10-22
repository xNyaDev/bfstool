use std::io;

use thiserror::Error;

/// Support for the Bzf2001 encryption format
pub mod bzf2001;

/// Errors that can occur while encryption/decryption
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CryptError {
    /// An IO error occurred
    #[error("An IO error occurred: {0}")]
    IoError(#[from] io::Error),
    /// Error while parsing with binrw
    #[error("A parsing error occurred: {0}")]
    ParsingError(String),
}

impl From<binrw::Error> for CryptError {
    fn from(error: binrw::Error) -> Self {
        match error {
            binrw::Error::Io(io_error) => CryptError::IoError(io_error),
            error => CryptError::ParsingError(error.to_string()),
        }
    }
}
