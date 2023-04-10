use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use bfstool::Format::Bfs2004a;
use bfstool::{read_archive_file, CompressionMethod};

use crate::display::display_size;

#[derive(Parser)]
pub struct Arguments {
    /// BFS archive file name
    archive: PathBuf,
    /// Ignore invalid magic/version/hash size
    #[clap(long)]
    force: bool,
    /// Output directory
    output: PathBuf,
    /// Print names of extracted files
    #[clap(short, long)]
    verbose: bool,
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let mut archive = read_archive_file(&arguments.archive, Bfs2004a, arguments.force)?;

    let file_names = archive.file_names();

    let bar = ProgressBar::new(file_names.len() as u64);

    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] {wide_bar} [{pos}/{len}]")
            .unwrap()
            .progress_chars("##-"),
    );

    archive.extract_files(
        file_names,
        &arguments.output,
        Box::new(|file_name, file_info| {
            if arguments.verbose {
                if file_info.compression_method == CompressionMethod::None {
                    bar.println(format!("{} [{}]", file_name, display_size(&file_info.size)));
                } else {
                    bar.println(format!(
                        "{} [{} -> {}]",
                        file_name,
                        display_size(&file_info.compressed_size),
                        display_size(&file_info.size)
                    ));
                }
            }
            bar.inc(1);
        }),
    )?;

    bar.finish_and_clear();

    println!(
        "Extracted {}.",
        if bar.length() == Some(1) {
            "1 file".to_string()
        } else {
            format!("{} files", bar.length().unwrap_or_default())
        }
    );

    Ok(())
}
