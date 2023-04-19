use std::error::Error;

use clap::{Parser, Subcommand};

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

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List(arguments) => list::run(arguments, &mut std::io::stdout()),
        Commands::Tree(arguments) => tree::run(arguments, &mut std::io::stdout()),
        Commands::Extract(arguments) => extract::run(arguments),
    }
}
