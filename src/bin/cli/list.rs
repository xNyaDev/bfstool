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

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let archive = read_archive_file(&arguments.archive, Bfs2004a, arguments.force)?;

    let table_contents = archive
        .file_names()
        .iter()
        .map(|name| (name, archive.file_info(name)))
        .flat_map(|(name, file_info_vec)| {
            file_info_vec
                .into_iter()
                .map(move |file_info| (name, file_info))
        })
        .map(|(name, file_info)| TableFileInfo {
            method: file_info.compression_method,
            size: file_info.size,
            compressed: file_info.compressed_size,
            copies: file_info.copies,
            offset: file_info.offset,
            file_name: name.to_string(),
        })
        .collect::<Vec<TableFileInfo>>();

    println!("Listing archive: {}", arguments.archive.to_string_lossy());
    println!(
        "Physical size: {}",
        display_size(&fs::metadata(&arguments.archive).unwrap().len())
    );
    println!("File count: {}", archive.file_count());
    println!(
        "{}",
        Table::new(table_contents)
            .with(Style::markdown())
            .with(Style::markdown())
            .with(Modify::new(Segment::all()).with(Alignment::right()))
            .with(Modify::new(Columns::single(4)).with(Alignment::center()))
            .with(Modify::new(Columns::last()).with(Alignment::left()))
    );
    Ok(())
}
