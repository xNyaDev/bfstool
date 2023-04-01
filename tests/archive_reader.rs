use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use pretty_assertions::assert_eq;

#[test]
fn test_bfs2004a() -> Result<(), Box<dyn Error>> {
    let file = File::open("test_data/bfs2004a.bin")?;
    let mut file_reader = BufReader::new(file);

    let archive = bfstool::read_archive(&mut file_reader, bfstool::Format::Bfs2004a, false)?;

    assert_eq!(archive.file_count(), 1);
    assert_eq!(archive.file_names(), vec!["data/language/version.ini"]);

    Ok(())
}
