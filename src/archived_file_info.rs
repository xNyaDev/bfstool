use crate::CompressionMethod;

/// Provides information about an archived file, without the name
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ArchivedFileInfo {
    /// Offset of this file in the archive
    pub offset: usize,
    /// Compression method used by this file
    pub compression_method: CompressionMethod,
    /// Uncompressed size of the file
    pub size: usize,
    /// Compressed size of the file
    pub compressed_size: usize,
    /// Number of copies of this file
    pub copies: usize,
    /// File hash
    pub hash: Option<u32>,
}
