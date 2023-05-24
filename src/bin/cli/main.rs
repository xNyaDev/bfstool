use std::error::Error;

use clap::{Parser, Subcommand, ValueEnum};

mod display;
mod extract;
mod list;
mod tree;

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
    List(list::Arguments),
    /// Display all files in the archive in a tree-like fashion
    Tree(tree::Arguments),
    /// Extract all files from the archive
    #[clap(visible_alias = "e", visible_alias = "x")]
    Extract(extract::Arguments),
}

#[derive(ValueEnum, Clone, Eq, PartialEq)]
enum Format {
    Bfs2004a,
    Bfs2004b,
    Bfs2007,
}

impl From<Format> for bfstool::Format {
    fn from(value: Format) -> Self {
        match value {
            Format::Bfs2004a => bfstool::Format::Bfs2004a,
            Format::Bfs2004b => bfstool::Format::Bfs2004b,
            Format::Bfs2007 => bfstool::Format::Bfs2007,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List(arguments) => list::run(arguments, &mut std::io::stdout()),
        Commands::Tree(arguments) => tree::run(arguments, &mut std::io::stdout()),
        Commands::Extract(arguments) => extract::run(arguments),
    }
}
