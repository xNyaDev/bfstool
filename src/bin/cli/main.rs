use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use number_prefix::NumberPrefix;
use tabled::object::{Columns, Segment};
use tabled::{Alignment, Modify, Style, Table, Tabled};

use bfstool::Format::Bfs2004a;
use bfstool::{read_archive, CompressionMethod};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all files in the archive
    #[clap(visible_alias = "l", visible_alias = "ls")]
    List {
        /// BFS archive file name
        archive: PathBuf,
        /// Ignore invalid magic/version/hash size
        #[clap(long)]
        force: bool,
    },
}

#[derive(Tabled, Eq, PartialEq)]
struct TableFileInfo {
    #[tabled(rename = "Method")]
    pub method: CompressionMethod,

    #[tabled(rename = "Size", display_with = "display_size")]
    pub size: usize,

    #[tabled(rename = "Compressed", display_with = "display_size")]
    pub compressed: usize,

    #[tabled(rename = "Copies")]
    pub copies: usize,

    #[tabled(rename = "Offset", display_with = "display_offset")]
    pub offset: usize,

    #[tabled(rename = "File Name")]
    pub file_name: String,
}

fn display_offset(offset: &usize) -> String {
    format!("{:08x}", offset)
}

fn display_size(size: &usize) -> String {
    match NumberPrefix::binary(*size as f64) {
        NumberPrefix::Standalone(bytes) => {
            format!("{} B", bytes)
        }
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{:.1} {}B", n, prefix)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List { archive, force } => {
            let file = File::open(&archive)?;
            let archive_name = archive;

            let mut file_reader = BufReader::new(file);
            let archive = read_archive(&mut file_reader, Bfs2004a, force)?;

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

            println!("Listing archive: {}", archive_name.to_string_lossy());
            println!(
                "Physical size: {}",
                fs::metadata(&archive_name).unwrap().len()
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
            )
        }
    }
    Ok(())
}
