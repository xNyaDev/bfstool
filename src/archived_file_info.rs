use std::fmt::{Display, Formatter};

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

/// Available compression methods
#[derive(Debug, Default, Eq, PartialEq)]
pub enum CompressionMethod {
    /// No compression
    #[default]
    None,
    /// zlib compression
    Zlib,
}

impl Display for CompressionMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompressionMethod::None => {
                    "none"
                }
                CompressionMethod::Zlib => {
                    "zlib"
                }
            }
        )
    }
}
