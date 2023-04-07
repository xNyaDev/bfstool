use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

use bfstool::read_archive_file;
use bfstool::Format::Bfs2004a;

#[derive(Parser)]
pub struct Arguments {
    /// BFS archive file name
    archive: PathBuf,
    /// Ignore invalid magic/version/hash size
    #[clap(long)]
    force: bool,
    /// Output directory
    output: PathBuf,
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let mut archive = read_archive_file(&arguments.archive, Bfs2004a, arguments.force)?;

    archive.extract_files(archive.file_names(), &arguments.output)?;

    Ok(())
}
