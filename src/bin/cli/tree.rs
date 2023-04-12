use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use termtree::Tree;

use bfstool::read_archive_file;
use bfstool::Format::Bfs2004a;

use crate::display::display_size;

#[derive(Parser)]
pub struct Arguments {
    /// BFS archive file name
    archive: PathBuf,
    /// Ignore invalid magic/version/hash size
    #[clap(long)]
    force: bool,
}

#[derive(Debug)]
struct TreeDirectory {
    name: String,
    size: u64,
    directory_children: Vec<TreeDirectory>,
    file_children: Vec<TreeFile>,
}

#[derive(Debug)]
struct TreeFile {
    name: String,
    size: u64,
}

fn insert_tree_file(directory: &mut TreeDirectory, to_create: &mut VecDeque<&str>, size: u64) {
    if to_create.len() == 1 {
        directory.file_children.push(TreeFile {
            name: to_create.pop_front().unwrap().to_string(),
            size,
        })
    } else {
        let new_directory_name = to_create.pop_front().unwrap();
        match directory
            .directory_children
            .iter_mut()
            .find(|directory| directory.name == new_directory_name)
        {
            Some(directory) => {
                insert_tree_file(directory, to_create, size);
            }
            None => {
                let mut new_directory = TreeDirectory {
                    name: new_directory_name.to_string(),
                    size: 0,
                    directory_children: vec![],
                    file_children: vec![],
                };
                insert_tree_file(&mut new_directory, to_create, size);
                directory.directory_children.push(new_directory);
            }
        };
    }
}

fn calculate_directory_size(directory: &mut TreeDirectory) {
    if !directory.directory_children.is_empty() {
        directory
            .directory_children
            .iter_mut()
            .for_each(calculate_directory_size);
    }
    let size = directory
        .directory_children
        .iter()
        .fold(0, |acc, directory| acc + directory.size);
    let size = directory
        .file_children
        .iter()
        .fold(size, |acc, file| acc + file.size);
    directory.size = size;
}

fn build_printable_tree(directory: &TreeDirectory) -> Tree<String> {
    let result = directory.directory_children.iter().fold(
        Tree::new(format!(
            "{} [{}]",
            directory.name,
            display_size(&directory.size)
        )),
        |mut root, directory| {
            root.push(build_printable_tree(directory));
            root
        },
    );
    let result = directory
        .file_children
        .iter()
        .fold(result, |mut root, file| {
            root.push(format!("{} [{}]", file.name, display_size(&file.size)));
            root
        });
    result
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let archive = read_archive_file(&arguments.archive, Bfs2004a, arguments.force)?;

    let mut tree = archive
        .file_names()
        .iter()
        .map(|name| (name, archive.file_info(name)))
        .flat_map(|(name, file_info_vec)| {
            file_info_vec
                .into_iter()
                .map(move |file_info| (name, file_info))
        })
        .fold(
            TreeDirectory {
                name: arguments
                    .archive
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                size: 0,
                directory_children: vec![],
                file_children: vec![],
            },
            |mut root, (name, file_info)| {
                let mut path = name.split('/').collect::<VecDeque<&str>>();
                insert_tree_file(&mut root, &mut path, file_info.size);
                root
            },
        );

    calculate_directory_size(&mut tree);

    println!("Listing archive: {}", arguments.archive.to_string_lossy());
    println!(
        "Physical size: {}",
        display_size(&fs::metadata(&arguments.archive).unwrap().len())
    );
    println!("File count: {}", archive.file_count());
    println!();
    println!("{}", build_printable_tree(&tree));

    Ok(())
}
