use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use termtree::Tree;

use bfstool::read_archive_file;

use crate::display::display_size;

use super::Format;

#[derive(Parser)]
pub struct Arguments {
    /// BFS archive file name
    archive: PathBuf,
    /// Ignore invalid magic/version/hash size
    #[clap(long)]
    force: bool,
    /// BFS archive format
    #[clap(short, long)]
    format: Format,
}

#[derive(Debug, Eq, PartialEq)]
struct TreeDirectory {
    name: String,
    size: u64,
    directory_children: Vec<TreeDirectory>,
    file_children: Vec<TreeFile>,
}

#[derive(Debug, Eq, PartialEq)]
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

pub fn run(arguments: Arguments, mut writer: impl std::io::Write) -> Result<(), Box<dyn Error>> {
    let archive = read_archive_file(&arguments.archive, arguments.format.into(), arguments.force)?;

    let mut tree = archive
        .multiple_file_info(archive.file_names())
        .into_iter()
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
    writeln!(writer)?;
    writeln!(writer, "{}", build_printable_tree(&tree))?;

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
            format: Format::Bfs2004a,
        };
        run(arguments, &mut result)?;

        let mut expected_result_file = File::open("test_data/cli/tree.txt")?;
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

    #[test]
    fn tree_creation_test() {
        let mut tree = TreeDirectory {
            name: "root".to_string(),
            size: 0,
            directory_children: vec![],
            file_children: vec![],
        };

        let path = "dir1/file1.txt".to_string();
        let mut path = path.split('/').collect::<VecDeque<&str>>();

        insert_tree_file(&mut tree, &mut path, 100);

        assert_eq!(
            tree,
            TreeDirectory {
                name: "root".to_string(),
                size: 0,
                directory_children: vec![TreeDirectory {
                    name: "dir1".to_string(),
                    size: 0,
                    directory_children: vec![],
                    file_children: vec![TreeFile {
                        name: "file1.txt".to_string(),
                        size: 100,
                    }],
                }],
                file_children: vec![],
            }
        );

        let path = "dir1/file2.txt".to_string();
        let mut path = path.split('/').collect::<VecDeque<&str>>();

        insert_tree_file(&mut tree, &mut path, 200);

        assert_eq!(
            tree,
            TreeDirectory {
                name: "root".to_string(),
                size: 0,
                directory_children: vec![TreeDirectory {
                    name: "dir1".to_string(),
                    size: 0,
                    directory_children: vec![],
                    file_children: vec![
                        TreeFile {
                            name: "file1.txt".to_string(),
                            size: 100,
                        },
                        TreeFile {
                            name: "file2.txt".to_string(),
                            size: 200,
                        }
                    ],
                }],
                file_children: vec![],
            }
        );

        calculate_directory_size(&mut tree);

        assert_eq!(
            tree,
            TreeDirectory {
                name: "root".to_string(),
                size: 300,
                directory_children: vec![TreeDirectory {
                    name: "dir1".to_string(),
                    size: 300,
                    directory_children: vec![],
                    file_children: vec![
                        TreeFile {
                            name: "file1.txt".to_string(),
                            size: 100,
                        },
                        TreeFile {
                            name: "file2.txt".to_string(),
                            size: 200,
                        }
                    ],
                }],
                file_children: vec![],
            }
        );
    }
}
