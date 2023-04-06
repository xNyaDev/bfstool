use std::error::Error;
use std::path::PathBuf;

use pretty_assertions::assert_eq;

use bfstool::{ArchivedFileInfo, CompressionMethod};

#[test]
fn test_bfs2004a() -> Result<(), Box<dyn Error>> {
    let archive = bfstool::read_archive_file(
        &PathBuf::from("test_data/bfs2004a.bin"),
        bfstool::Format::Bfs2004a,
        false,
    )?;

    assert_eq!(archive.file_count(), 1);
    assert_eq!(archive.file_names(), vec!["data/language/version.ini"]);
    assert_eq!(
        archive.file_info("data/language/version.ini"),
        vec![ArchivedFileInfo {
            offset: 0xFDC,
            compression_method: CompressionMethod::Zlib,
            size: 0x44F,
            compressed_size: 0x1D7,
            copies: 0,
            hash: Some(0xF6260C6E),
        }]
    );
    assert_eq!(archive.file_info("non_existing_file"), vec![]);

    Ok(())
}
