use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use pretty_assertions::assert_eq;

use bfstool::{ArchivedFileInfo, CompressionMethod};

#[test]
fn test_bfs2004a() -> Result<(), Box<dyn Error>> {
    let file = File::open("test_data/bfs2004a.bin")?;
    let mut file_reader = BufReader::new(file);

    let archive = bfstool::read_archive(&mut file_reader, bfstool::Format::Bfs2004a, false)?;

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
