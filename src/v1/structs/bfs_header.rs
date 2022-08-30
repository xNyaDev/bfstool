use crate::util::{AsBytes, u32_from_le_bytes};

#[derive(Default)]
pub struct BfsHeader {
    /// File identification magic
    ///
    /// `62 66 73 31`, `"bfs1"`
    pub magic: u32,

    /// Library used to write the file
    ///
    /// `78 4E 79 61`, `"xNya"` for this tool
    pub writing_library: u32,

    /// Offset at which file headers + names end, there may be some null bytes after this before
    /// the actual data begins
    ///
    /// Offset from `0h`
    pub data_offset: u32,

    /// Number of files total in the archive
    pub file_count: u32,
}

impl AsBytes for BfsHeader {
    const BYTE_COUNT: usize = 16;

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut bytes = bytes;
        Self {
            magic: u32_from_le_bytes(&mut bytes),
            writing_library: u32_from_le_bytes(&mut bytes),
            data_offset: u32_from_le_bytes(&mut bytes),
            file_count: u32_from_le_bytes(&mut bytes),
        }
    }
    fn to_bytes(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.magic.to_le_bytes());
        result.extend_from_slice(&self.writing_library.to_le_bytes());
        result.extend_from_slice(&self.data_offset.to_le_bytes());
        result.extend_from_slice(&self.file_count.to_le_bytes());
        result
    }
}