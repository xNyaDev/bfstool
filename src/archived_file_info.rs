use crate::CompressionMethod;

/// Provides information about an archived file, without the name
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ArchivedFileInfo {
    /// Offset of this file in the archive
    pub offset: u64,
    /// Compression method used by this file
    pub compression_method: CompressionMethod,
    /// Uncompressed size of the file
    pub size: u64,
    /// Compressed size of the file
    pub compressed_size: u64,
    /// Number of copies of this file
    pub copies: u64,
    /// File hash
    pub hash: Option<u32>,
}
