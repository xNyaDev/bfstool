use crate::util::{AsBytes, u32_from_le_bytes};

#[derive(Clone, Default)]
pub struct FileInfoTableEntry {
    /// Offset at which the `FileHeaders` for this hash start
    ///
    /// Offset from `0h`
    pub file_header_offset: u32,

    /// Number of files for this specific hash
    pub file_count: u32,
}

impl AsBytes for FileInfoTableEntry {
    const BYTE_COUNT: usize = 8;

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut bytes = bytes;
        Self {
            file_header_offset: u32_from_le_bytes(&mut bytes),
            file_count: u32_from_le_bytes(&mut bytes),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.file_header_offset.to_le_bytes());
        result.extend_from_slice(&self.file_count.to_le_bytes());
        result
    }
}