use crate::util::{AsBytes, u16_from_le_bytes};

#[derive(Default)]
pub struct FileInfoTableEntry {
    /// The starting file index with this hash
    pub starting_file: u16,

    /// Number of files for this specific hash
    pub file_count: u16,
}

impl AsBytes for FileInfoTableEntry {
    const BYTE_COUNT: usize = 4;

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut bytes = bytes;
        Self {
            starting_file: u16_from_le_bytes(&mut bytes),
            file_count: u16_from_le_bytes(&mut bytes),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.starting_file.to_le_bytes());
        result.extend_from_slice(&self.file_count.to_le_bytes());
        result
    }
}