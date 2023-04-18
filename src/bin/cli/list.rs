use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use tabled::settings::object::{Columns, Segment};
use tabled::settings::{Alignment, Modify, Style};
use tabled::{Table, Tabled};

use bfstool::read_archive_file;
use bfstool::CompressionMethod;
use bfstool::Format::Bfs2004a;

use crate::display::{display_offset, display_size};

#[derive(Parser)]
pub struct Arguments {
    /// BFS archive file name
    archive: PathBuf,
    /// Ignore invalid magic/version/hash size
    #[clap(long)]
    force: bool,
}

#[derive(Tabled, Eq, PartialEq)]
pub struct TableFileInfo {
    #[tabled(rename = "Method")]
    pub method: CompressionMethod,

    #[tabled(rename = "Size", display_with = "display_size")]
    pub size: u64,

    #[tabled(rename = "Compressed", display_with = "display_size")]
    pub compressed: u64,

    #[tabled(rename = "Copies")]
    pub copies: u64,

    #[tabled(rename = "Offset", display_with = "display_offset")]
    pub offset: u64,

    #[tabled(rename = "File Name")]
    pub file_name: String,
}

pub fn run(arguments: Arguments, mut writer: impl std::io::Write) -> Result<(), Box<dyn Error>> {
    let archive = read_archive_file(&arguments.archive, Bfs2004a, arguments.force)?;

    let table_contents = archive
        .multiple_file_info(archive.file_names())
        .into_iter()
        .map(|(name, file_info)| TableFileInfo {
            method: file_info.compression_method,
            size: file_info.size,
            compressed: file_info.compressed_size,
            copies: file_info.copies,
            offset: file_info.offset,
            file_name: name,
        })
        .collect::<Vec<TableFileInfo>>();

    writeln!(
        writer,
        "Listing archive: {}",
        arguments.archive.to_string_lossy()
    )?;
    writeln!(
        writer,
        "Physical size: {}",
        display_size(&fs::metadata(&arguments.archive).unwrap().len())
    )?;
    writeln!(writer, "File count: {}", archive.file_count())?;
    writeln!(
        writer,
        "{}",
        Table::new(table_contents)
            .with(Style::markdown())
            .with(Modify::new(Segment::all()).with(Alignment::right()))
            .with(Modify::new(Columns::single(4)).with(Alignment::center()))
            .with(Modify::new(Columns::last()).with(Alignment::left()))
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn listing_test() -> Result<(), Box<dyn Error>> {
        let mut result = Vec::new();
        let arguments = Arguments {
            archive: PathBuf::from("test_data/bfs2004a/europe.bin"),
            force: false,
        };
        run(arguments, &mut result)?;

        let mut expected_result_file = File::open("test_data/cli/list.txt")?;
        let mut expected_result = Vec::new();
        expected_result_file.read_to_end(&mut expected_result)?;

        // Compare results as strings for pretty diff when mismatching
        //
        // Ignore mismatching line breaks when comparing (assume \r\n and \n are equal) by
        // removing all occurrences of \r
        assert_eq!(
            String::from_utf8_lossy(&result)
                .to_string()
                .replace('\r', ""),
            String::from_utf8_lossy(&expected_result)
                .to_string()
                .replace('\r', "")
        );

        Ok(())
    }
}
