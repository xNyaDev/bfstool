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
