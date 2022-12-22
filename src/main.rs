use std::{fs, io};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use tabled::{Alignment, Modify, Style, Table, Tabled};
use tabled::object::{Columns, Segment};

use crate::archived_data::{raw_extract, zlib_extract};
use crate::bfs::{BfsFile, BfsFileTrait};
use crate::crypt::{create_key, decrypt_headers_block, read_and_decrypt_block};
use crate::Endianness::{Be, Le};
use crate::filter::{apply_copy_filters, apply_filters, apply_single_filter, load_copy_filters, load_filters};
use crate::identify::{identify, identify_format};
use crate::key_parser::KeyValueParser;
use crate::version_parser::VersionValueParser;
use crate::util::{list_files_recursively, string_lines_to_vec, u32_from_be_bytes, u32_from_le_bytes, write_data_to_file_endian};

mod bfs;
mod util;
mod archived_data;
mod filter;
mod v1;
mod v2;
mod identify;
mod v3;
mod crypt;
mod key_parser;
mod version_parser;

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
        bfs_name: String,
        /// File format, if omitted bfstool will try to identify the file using bfs_file_dat.md
        #[clap(short, long, value_enum)]
        format: Option<Format>,
        /// List only filenames
        #[clap(long)]
        raw: bool,
        /// Order in which to list the files
        #[clap(short, long, value_enum, default_value = "name-asc")]
        order: FileListOrder,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Extract files from the archive
    #[clap(visible_alias = "e", visible_alias = "x")]
    Extract {
        /// BFS archive file name
        bfs_name: String,
        /// Folder to extract to
        output_folder: String,
        /// Filter which files to extract - if omitted, all files are extracted
        filter: Option<String>,
        /// File format, if omitted bfstool will try to identify the file using bfs_file_dat.md
        #[clap(short, long, value_enum)]
        format: Option<Format>,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Archive all files in a folder
    #[clap(visible_alias = "a")]
    Archive {
        /// BFS archive file name
        bfs_name: String,
        /// Folder to archive
        input_folder: String,
        /// Compression scheme. Non-zlib supported only for FO2 w/ Reloaded ModLoader add-in.
        #[clap(long, value_enum, default_value_t = Compression::Zlib)]
        compression: Compression,
        /// Compression level [0-9] for Zlib. [0-12] for LZ4, [0-22] for Zlib.
        #[clap(value_parser = clap::value_parser ! (u32).range(0..=9), short, long)]
        level: Option<u32>,
        /// Filter for compression - You can either supply the filter name or a filter file
        #[clap(long, value_enum, required_unless_present_any = ["filter_file", "version", "help"])]
        filter: Option<Filter>,
        /// Filter file for compression
        #[clap(long, conflicts_with = "filter", required_unless_present_any = ["filter", "version", "help"])]
        filter_file: Option<String>,
        /// Copy filter for multiple file copies - You can either supply the filter name or a filter file
        #[clap(long, value_enum, required_unless_present_any = ["copy_filter_file", "version", "help"])]
        copy_filter: Option<CopyFilter>,
        /// Copy filter file for multiple file copies
        #[clap(long, conflicts_with = "copy_filter", required_unless_present_any = ["copy_filter", "version", "help"])]
        copy_filter_file: Option<String>,
        /// File format
        #[clap(short, long, value_enum)]
        format: Format,
        /// BFS archive version
        #[clap(long, value_parser = VersionValueParser::new())]
        file_version: [u8; 4],
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Stores all files with matching hash only once
        #[clap(long)]
        deduplicate: bool,
    },
    /// Identify an unknown BFS file using file hashes from bfs_file_dat.md
    #[clap(visible_alias = "i", visible_alias = "id", visible_alias = "info")]
    Identify {
        /// BFS archive file name
        bfs_name: String,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Test if the filters in the archive match the given one
    #[clap(visible_alias = "tf")]
    TestFilters {
        /// BFS archive file name
        bfs_name: String,
        /// Filter for compression - You can either supply the filter name or a filter file
        #[clap(long, value_enum)]
        filter: Option<Filter>,
        /// Filter file for compression
        #[clap(long, conflicts_with = "filter")]
        filter_file: Option<String>,
        /// File format, if omitted bfstool will try to identify the file using bfs_file_dat.md
        #[clap(short, long, value_enum)]
        format: Option<Format>,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Test if the copy filters in the archive match the given one
    #[clap(visible_alias = "tcf")]
    TestCopyFilters {
        /// BFS archive file name
        bfs_name: String,
        /// Filter for file copies - You can either supply the filter name or a filter file
        #[clap(long, value_enum)]
        copy_filter: Option<CopyFilter>,
        /// Filter file for file copies
        #[clap(long, conflicts_with = "copy_filter")]
        copy_filter_file: Option<String>,
        /// File format, if omitted bfstool will try to identify the file using bfs_file_dat.md
        #[clap(short, long, value_enum)]
        format: Option<Format>,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Decrypt an archive
    Decrypt {
        /// The input file to decrypt
        input: String,
        /// The decrypted file
        output: String,
        /// Key for the BFS archive
        #[clap(long, value_parser = KeyValueParser::new())]
        key: [u8; 16],
        /// Key for the archive header
        #[clap(long, value_parser = KeyValueParser::new())]
        header_key: [u8; 16],
        #[clap(long, value_enum)]
        /// Data endianness, if omitted bfstool will try to guess
        data_mode: Option<Endianness>,
        #[clap(long, value_enum)]
        /// Key endianness, if omitted bfstool will try to guess
        key_mode: Option<Endianness>,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
    },
    /// Dump file and generate rebuild info
    #[clap(visible_alias = "d")]
    Dump {
        /// BFS archive file name
        bfs_name: String,
        /// Folder to dump to
        output_folder: String,
        /// File format, if omitted bfstool will try to identify the file using bfs_file_dat.md
        #[clap(short, long, value_enum)]
        format: Option<Format>,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
        /// Treat the file name as CRC32 instead of calculating
        #[clap(long)]
        fast_identify: bool,
    },
    /// Rebuild file from given info
    #[clap(visible_alias = "r")]
    Rebuild {
        /// Rebuild info JSON file
        rebuild_info: String,
        /// BFS archive file name
        bfs_name: String,
        /// Print more info
        #[clap(short, long)]
        verbose: bool,
        /// Suppress progress bar
        #[clap(short = 'q', long)]
        no_progress: bool,
    },
}

#[derive(ValueEnum, Clone, Eq, PartialEq)]
pub enum Format {
    V1,
    V1a,
    V2,
    V2a,
    V3,
}

#[derive(ValueEnum, Clone, Eq, PartialEq, Copy)]
pub enum Compression {
    Zlib,
    ZStd,
    Lz4
}

#[derive(ValueEnum, Clone, Eq, PartialEq)]
pub enum Filter {
    All,
    None,
    Fo1,
    Fo2,
    Fo2FxPatch,
    Fo2Demo,
    Fo2Ps2Beta,
    Fo2XboxBeta,
    Fouc,
    FoucX360,
    Foho,
    Srr,
    Rru,
    Fo2PcModLoader,
}

#[derive(ValueEnum, Clone, Eq, PartialEq)]
pub enum CopyFilter {
    None,
    Fo1Pc,
    Fo1Ps2,
    Fo1Ps2Jp,
    Fo1Ps2Usa,
    Fo1Xbox,
    Fo2Pc,
    Fo2Ps2,
    Fo2Ps2Beta,
    Fo2Ps2GermanPack,
    Fo2Ps2Usa,
    Fo2Xbox,
    Fo2XboxBeta,
    FoucPc,
    FoucPcLangpack,
    FoucX360,
    FoucX360De,
    FoucX360Jp,
    Foho,
    Srr,
    Rru,
    RruPcUpdate
}

#[derive(ValueEnum, Clone, Eq, PartialEq)]
pub enum FileListOrder {
    MethodAsc,
    MethodDesc,
    SizeAsc,
    SizeDesc,
    CompressedAsc,
    CompressedDesc,
    CopiesAsc,
    CopiesDesc,
    OffsetAsc,
    OffsetDesc,
    NameAsc,
    NameDesc,
}

#[derive(ValueEnum, Clone, Eq, PartialEq, Copy)]
pub enum Endianness {
    Le,
    Be,
}

fn main() {
    // CLion does not detect variable type here
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List {
            bfs_name,
            format,
            raw,
            order,
            no_progress,
            fast_identify
        } => {
            let format = identify_format(&bfs_name, no_progress, fast_identify, format);
            let bfs_file = BfsFile::read_bfs_from_file(
                bfs_name.clone(),
                format,
            ).expect("Failed to open BFS file");

            fn display_offset(offset: &u32) -> String {
                format!("{:08x}", offset)
            }
            fn display_copies(copies: &(u8, u16)) -> String {
                format!("{}+{}", copies.0, copies.1)
            }

            #[derive(Tabled, Eq, PartialEq)]
            pub struct FileToList {
                #[tabled(rename = "Method")]
                pub method: FileMethod,

                #[tabled(rename = "Size")]
                pub size: u32,

                #[tabled(rename = "Compressed")]
                pub compressed: u32,

                #[tabled(rename = "Copies", display_with = "display_copies")]
                pub copies: (u8, u16),

                #[tabled(rename = "Offset", display_with = "display_offset")]
                pub offset: u32,

                #[tabled(rename = "File Name")]
                pub file_name: String,
            }

            #[derive(Eq, PartialEq, Ord, PartialOrd)]
            pub enum FileMethod {
                Store,
                Zlib,
                Unknown(u8),
            }

            impl Display for FileMethod {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    match self {
                        FileMethod::Store => {
                            write!(f, "store")
                        }
                        FileMethod::Zlib => {
                            write!(f, "zlib")
                        }
                        FileMethod::Unknown(num) => {
                            write!(f, "{}", num)
                        }
                    }
                }
            }

            let file_headers = bfs_file.get_file_headers();

            let files = bfs_file.get_file_name_to_header_map().iter().map(
                |(file_name, file_header_index)| {
                    let file_header = file_headers.get(file_header_index.clone()).unwrap();
                    FileToList {
                        method: match file_header.get_method() {
                            5 | 1 => FileMethod::Zlib,
                            4 | 0 => FileMethod::Store,
                            unknown => FileMethod::Unknown(unknown),
                        },
                        size: file_header.get_unpacked_size(),
                        compressed: file_header.get_packed_size(),
                        copies: file_header.get_file_copies_num(),
                        offset: file_header.get_data_offset(),
                        file_name: file_name.clone(),
                    }
                }
            ).collect::<Vec<FileToList>>();

            if !raw {
                println!("Listing archive: {}", bfs_name);
                println!("Physical size: {}", fs::metadata(&bfs_name).unwrap().len());
                println!("Headers size: {}", bfs_file.get_data_offset() - 1);
                println!("File count: {}", files.len());
                let version = bfs_file.get_file_version().to_le_bytes();
                println!(
                    "File version: {:02x}{:02x}{:02x}{:02x}",
                    version[0],
                    version[1],
                    version[2],
                    version[3]
                );
            }

            let mut files = files;
            files.sort_unstable_by(
                match order {
                    FileListOrder::MethodAsc => {
                        |x: &FileToList, y: &FileToList| {
                            x.method.cmp(&y.method)
                        }
                    }
                    FileListOrder::MethodDesc => {
                        |x: &FileToList, y: &FileToList| {
                            y.method.cmp(&x.method)
                        }
                    }
                    FileListOrder::SizeAsc => {
                        |x: &FileToList, y: &FileToList| {
                            x.size.cmp(&y.size)
                        }
                    }
                    FileListOrder::SizeDesc => {
                        |x: &FileToList, y: &FileToList| {
                            y.size.cmp(&x.size)
                        }
                    }
                    FileListOrder::CompressedAsc => {
                        |x: &FileToList, y: &FileToList| {
                            x.compressed.cmp(&y.compressed)
                        }
                    }
                    FileListOrder::CompressedDesc => {
                        |x: &FileToList, y: &FileToList| {
                            y.compressed.cmp(&x.compressed)
                        }
                    }
                    FileListOrder::CopiesAsc => {
                        |x: &FileToList, y: &FileToList| {
                            (x.copies.0 as u16 + x.copies.1).cmp(&(y.copies.0 as u16 + y.copies.1))
                        }
                    }
                    FileListOrder::CopiesDesc => {
                        |x: &FileToList, y: &FileToList| {
                            (y.copies.0 as u16 + y.copies.1).cmp(&(x.copies.0 as u16 + x.copies.1))
                        }
                    }
                    FileListOrder::OffsetAsc => {
                        |x: &FileToList, y: &FileToList| {
                            x.offset.cmp(&y.offset)
                        }
                    }
                    FileListOrder::OffsetDesc => {
                        |x: &FileToList, y: &FileToList| {
                            y.offset.cmp(&x.offset)
                        }
                    }
                    FileListOrder::NameAsc => {
                        |x: &FileToList, y: &FileToList| {
                            x.file_name.cmp(&y.file_name)
                        }
                    }
                    FileListOrder::NameDesc => {
                        |x: &FileToList, y: &FileToList| {
                            y.file_name.cmp(&x.file_name)
                        }
                    }
                }
            );

            if raw {
                files.into_iter().for_each(
                    |file| {
                        println!("{}", file.file_name);
                    }
                );
            } else {
                println!(
                    "{}",
                    Table::new(files)
                        .with(Style::markdown())
                        .with(
                            Modify::new(Segment::all())
                                .with(Alignment::right())
                        )
                        .with(
                            Modify::new(Columns::single(4))
                                .with(Alignment::center())
                        )
                        .with(
                            Modify::new(Columns::last())
                                .with(Alignment::left())
                        )
                );
            }
        }
        Commands::Extract {
            bfs_name,
            output_folder,
            filter,
            format,
            verbose,
            no_progress,
            fast_identify
        } => {
            let format = identify_format(&bfs_name, no_progress, fast_identify, format);
            let bfs_file = BfsFile::read_bfs_from_file(
                bfs_name.clone(),
                format,
            ).expect("Failed to open BFS file");

            let file_list = bfs_file.get_file_name_to_header_map().keys().cloned().into_iter().collect();
            let filter = filter.unwrap_or("**/*".to_string());
            let filtered_file_list = apply_single_filter(file_list, filter);

            if filtered_file_list.len() != 0 {
                // Create all required directories
                let folders = filtered_file_list.iter().map(
                    |file| {
                        let file_path = Path::new(file);
                        let folder_path = file_path.parent().unwrap();
                        folder_path.to_path_buf()
                    }
                ).collect::<HashSet<PathBuf>>();
                folders.into_iter().for_each(
                    |folder| {
                        let full_path = Path::new(&output_folder).join(folder);
                        fs::create_dir_all(full_path).expect("Failed to create a directory");
                    }
                );

                let bar = if no_progress {
                    ProgressBar::hidden()
                } else {
                    ProgressBar::new(filtered_file_list.len() as u64)
                };
                bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{pos}/{len}]").unwrap().progress_chars("##-"));

                let file = File::open(bfs_name).expect("Failed to open BFS file");
                let mut reader = BufReader::new(file);

                let file_name_to_header_map = bfs_file.get_file_name_to_header_map();
                let file_headers = bfs_file.get_file_headers();

                for file in filtered_file_list {
                    let file_header_index = file_name_to_header_map.get(&file).unwrap().clone();
                    let file_header = file_headers.get(file_header_index).unwrap();

                    let full_file_path = Path::new(&output_folder).join(&file);

                    let mut output_file = File::create(full_file_path).expect("Failed to create extracted");
                    let mut status;
                    if file_header.get_method() == 5 || file_header.get_method() == 1 { // zlib
                        let size = zlib_extract(&mut reader, &mut output_file, file_header.get_data_offset(), file_header.get_packed_size()).expect("Failed to write to extracted file");
                        status = format!("{} -> {} bytes", file_header.get_packed_size(), size);
                        if size != file_header.get_unpacked_size() as usize {
                            status += &format!(", {} expected. File may be corrupt.", file_header.get_unpacked_size());
                        }
                    } else { // store
                        let size = raw_extract(&mut reader, &mut output_file, file_header.get_data_offset(), file_header.get_unpacked_size()).expect("Failed to write to extracted file");
                        status = format!("{} bytes", size);
                    }

                    if verbose {
                        bar.println(format!("{file:?} {status}"));
                    }
                    bar.inc(1);
                }

                bar.finish_and_clear();

                if !no_progress {
                    let file_count = bar.length().unwrap_or(1);
                    if file_count == 1 {
                        println!("Extracted 1 file.");
                    } else {
                        println!("Extracted {file_count} files.");
                    }
                }
            } else {
                println!("No files to extract.");
            }
        }
        Commands::Archive {
            bfs_name,
            input_folder,
            level,
            filter,
            filter_file,
            copy_filter,
            copy_filter_file,
            format,
            file_version: version,
            verbose,
            no_progress,
            deduplicate,
            compression,
        } => {
            let input_files = list_files_recursively(input_folder.clone());

            if input_files.len() != 0 {
                let bar = if no_progress {
                    ProgressBar::hidden()
                } else {
                    ProgressBar::new(input_files.len() as u64)
                };
                bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{pos}/{len}]").unwrap().progress_chars("##-"));

                let filters = load_filters(filter, filter_file);
                let copy_filters = load_copy_filters(copy_filter, copy_filter_file);

                BfsFile::archive(
                    format,
                    bfs_name,
                    input_folder,
                    input_files,
                    verbose,
                    filters,
                    copy_filters,
                    level,
                    &bar,
                    version,
                    deduplicate,
                    compression
                ).expect("Failed to archive BFS file");

                bar.finish_and_clear();
                if !no_progress {
                    let file_count = bar.length().unwrap_or(1);
                    if file_count == 1 {
                        println!("Archived 1 file.");
                    } else {
                        println!("Archived {file_count} files.");
                    }
                }
            } else {
                println!("No files to archive.");
            }
        }
        Commands::Identify {
            bfs_name,
            no_progress,
            fast_identify
        } => {
            if let Some(file_info) = identify(&bfs_name, no_progress, fast_identify) {
                println!("File name: {}", file_info.file_name);
                println!("Game: {}", file_info.game);
                println!("Platform: {}", file_info.platform);
                println!("Format: {}", file_info.format);
                println!("Filter: {}", file_info.filter);
                println!("Copy filter: {}", file_info.copy_filter);
                println!("Source: ");
                string_lines_to_vec(file_info.source.clone()).into_iter().for_each(|line| {
                    println!("- {}", line.trim())
                });
                println!("CRC32: {}", file_info.crc32);
                println!("MD5: {}", file_info.md5);
                println!("SHA1: {}", file_info.sha1);
            } else {
                println!("File not found in the BFS file database.");
                println!("Perhaps it's a modded file or not yet supported by bfstool.");
                if fast_identify {
                    println!("Try removing --fast-identify and running again.");
                }
            }
        }
        Commands::TestFilters {
            bfs_name,
            filter,
            filter_file,
            format,
            verbose,
            no_progress,
            fast_identify
        } => {
            let (format, filter, filter_file) = if
            format.is_some() && (filter.is_some() || filter_file.is_some()) {
                (format.unwrap(), filter, filter_file)
            } else {
                let format = identify_format(&bfs_name, no_progress, fast_identify, format);
                let file_info = identify(
                    &bfs_name,
                    no_progress,
                    fast_identify,
                );
                let filter = if filter.is_some() {
                    filter
                } else if filter.is_none() && filter_file.is_none() {
                    if let Some(file_info) = file_info {
                        Some(Filter::from_str(&file_info.filter, false).unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                };
                (format, filter, filter_file)
            };

            let bfs_file = BfsFile::read_bfs_from_file(
                bfs_name.clone(),
                format,
            ).expect("Failed to open BFS file");

            println!("Testing filters for archive: {}", bfs_name);

            let all_files = bfs_file.get_file_name_to_header_map().keys().cloned().into_iter().collect();
            let file_headers = bfs_file.get_file_headers();
            let mut compressed_files = bfs_file.get_file_name_to_header_map().iter().filter_map(
                |(file_name, header_index)| {
                    if let Some(header) = file_headers.get(header_index.clone()) {
                        if header.get_method() == 1 || header.get_method() == 5 {
                            Some(file_name)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            ).cloned().collect::<Vec<String>>();
            compressed_files.sort_unstable();

            let filters = load_filters(filter, filter_file);
            let mut filtered_files = apply_filters(all_files, filters);
            filtered_files.sort_unstable();

            if filtered_files == compressed_files {
                println!("All files are matching the compression filter");
            } else {
                println!("Some files are not matching the compression filter");
                if verbose {
                    let compressed_files = compressed_files.into_iter().collect::<HashSet<String>>();
                    let filtered_files = filtered_files.into_iter().collect::<HashSet<String>>();

                    let mut not_in_compressed = filtered_files.difference(&compressed_files).cloned().collect::<Vec<String>>();
                    not_in_compressed.sort_unstable();

                    let mut not_in_filtered = compressed_files.difference(&filtered_files).cloned().collect::<Vec<String>>();
                    not_in_filtered.sort_unstable();

                    if !not_in_compressed.is_empty() {
                        println!("Files that are not compressed but should be:");
                        for file in not_in_compressed {
                            println!("+ {}", file);
                        }
                        if !not_in_filtered.is_empty() {
                            println!()
                        }
                    }

                    if !not_in_filtered.is_empty() {
                        println!("Files that are compressed but should not be:");
                        for file in not_in_filtered {
                            println!("- {}", file);
                        }
                    }
                } else {
                    println!("To see which ones, please add the --verbose flag");
                }
            }
        }
        Commands::TestCopyFilters {
            bfs_name,
            copy_filter,
            copy_filter_file,
            format,
            verbose,
            no_progress,
            fast_identify
        } => {
            let (format, copy_filter, copy_filter_file) = if
            format.is_some() && (copy_filter.is_some() || copy_filter_file.is_some()) {
                (format.unwrap(), copy_filter, copy_filter_file)
            } else {
                let format = identify_format(&bfs_name, no_progress, fast_identify, format);
                let file_info = identify(
                    &bfs_name,
                    no_progress,
                    fast_identify,
                );
                let copy_filter = if copy_filter.is_some() {
                    copy_filter
                } else if copy_filter.is_none() && copy_filter_file.is_none() {
                    if let Some(file_info) = file_info {
                        Some(CopyFilter::from_str(&file_info.copy_filter, false).unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                };
                (format, copy_filter, copy_filter_file)
            };

            let bfs_file = BfsFile::read_bfs_from_file(
                bfs_name.clone(),
                format,
            ).expect("Failed to open BFS file");

            println!("Testing copy filters for archive: {}", bfs_name);

            let mut file_names = bfs_file.get_file_name_to_header_map().keys().cloned().into_iter().collect::<Vec<String>>();
            file_names.sort_unstable();
            let file_headers = bfs_file.get_file_headers();

            let file_copy_map = bfs_file.get_file_name_to_header_map().iter().map(|(file_name, header_index)| {
                let file_header = file_headers.get(header_index.clone()).unwrap();
                (file_name.clone(), file_header.get_file_copies_num())
            }).collect::<HashMap<String, (u8, u16)>>();

            let filters = load_copy_filters(copy_filter, copy_filter_file);

            let filtered_file_copy_map = apply_copy_filters(file_names.clone(), filters);

            if file_copy_map == filtered_file_copy_map {
                println!("All files are matching the copy filter");
            } else {
                println!("Some files are not matching the copy filter");
                if verbose {
                    for file_name in file_names {
                        let actual = file_copy_map.get(&file_name).unwrap();
                        let expected = filtered_file_copy_map.get(&file_name).unwrap();
                        if actual != expected {
                            println!(
                                "{} is {}+{}, should be {}+{}",
                                file_name,
                                actual.0,
                                actual.1,
                                expected.0,
                                expected.1
                            );
                        }
                    }
                } else {
                    println!("To see which ones, please add the --verbose flag");
                }
            }
        }
        Commands::Decrypt {
            input,
            output,
            key,
            header_key,
            data_mode,
            key_mode,
            verbose,
            no_progress
        } => {
            let input_file = File::open(&input).expect("Failed to open input file");
            let mut input_file_reader = BufReader::new(input_file);

            if verbose {
                println!("Checking data key");
            }
            let combinations = match (data_mode, key_mode) {
                (Some(data_mode), Some(key_mode)) => {
                    vec![(data_mode, key_mode)]
                }
                (Some(data_mode), None) => {
                    vec![
                        (data_mode, Le),
                        (data_mode, Be),
                    ]
                }
                (None, Some(key_mode)) => {
                    vec![
                        (Le, key_mode),
                        (Be, key_mode),
                    ]
                }
                (None, None) => {
                    vec![
                        (Le, Le),
                        (Be, Be),
                        (Be, Le),
                        (Le, Be),
                    ]
                }
            };

            let mut combination = None;
            let mut data_offset = 0;
            let mut decrypted_data = Vec::new();

            for (data_mode, key_mode) in combinations {
                if verbose {
                    println!(
                        "Checking {} data with {} key",
                        match data_mode {
                            Le => "little endian",
                            Be => "big endian"
                        },
                        match key_mode {
                            Le => "little endian",
                            Be => "big endian"
                        }
                    );
                }

                let key = create_key(key, key_mode);

                input_file_reader.seek(SeekFrom::Start(0)).expect("Failed to read input file");

                let mut block_vec = read_and_decrypt_block(&mut input_file_reader, key, data_mode).expect("Failed to read input file");
                if block_vec.get(0) == Some(&0x31736662) || block_vec.get(0) == Some(&0x62667331) { // "bfs1" header
                    combination = Some((data_mode, key_mode));
                    data_offset = match data_mode {
                        Le => { block_vec.get(2).unwrap().clone() }
                        Be => { block_vec.get(2).unwrap().swap_bytes() }
                    } & 0x7FFFFFFF;
                    decrypted_data.append(&mut block_vec);
                    if verbose {
                        println!("Matched");
                    }
                    break;
                } else {
                    if verbose {
                        println!("Not matched");
                    }
                }
            }

            if let Some((data_mode, key_mode)) = combination {
                let key = create_key(key, key_mode);

                let mut decrypted_index = 0x8000;
                if verbose {
                    println!("Checking headers key");
                }

                while decrypted_index < data_offset {
                    decrypted_index += 0x8000;
                    let mut block_vec = read_and_decrypt_block(&mut input_file_reader, key, data_mode).expect("Failed to read input file");
                    decrypted_data.append(&mut block_vec);
                }

                let mut header_data: Vec<u32> = decrypted_data.drain(5..(data_offset as usize / size_of::<u32>())).collect();
                decrypt_headers_block(&mut header_data, create_key(header_key, key_mode));

                if header_data.get(0) == Some(&0x3E5) || header_data.get(0) == Some(&0xE5030000) {
                    if verbose {
                        println!("Headers decrypted, decrypting the entire file");
                    }
                    let file_size = fs::metadata(&input).unwrap().len();
                    let bar = if no_progress {
                        ProgressBar::hidden()
                    } else {
                        ProgressBar::new(file_size)
                    };
                    bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{bytes}/{total_bytes}]").unwrap().progress_chars("##-"));

                    let output_file = File::create(&output).expect("Failed to create output file");
                    let mut output_file_writer = BufWriter::new(output_file);

                    write_data_to_file_endian(
                        &mut output_file_writer,
                        decrypted_data.drain(0..5).collect(),
                        data_mode,
                    ).expect("Failed to write to output file");

                    write_data_to_file_endian(
                        &mut output_file_writer,
                        header_data,
                        data_mode,
                    ).expect("Failed to write to output file");

                    write_data_to_file_endian(
                        &mut output_file_writer,
                        decrypted_data,
                        data_mode,
                    ).expect("Failed to write to output file");

                    bar.inc(decrypted_index as u64);

                    for _ in ((decrypted_index as u64)..file_size).step_by(0x8000) {
                        let block_vec = read_and_decrypt_block(&mut input_file_reader, key, data_mode).expect("Failed to read input file");
                        write_data_to_file_endian(
                            &mut output_file_writer,
                            block_vec,
                            data_mode,
                        ).expect("Failed to write to output file");
                        bar.inc(0x8000);
                    }
                } else {
                    println!("Incorrect headers key");
                }
            } else {
                println!("Incorrect key");
            }
        }
        Commands::Dump {
            bfs_name,
            output_folder,
            format,
            verbose,
            no_progress,
            fast_identify
        } => {
            let format = identify_format(&bfs_name, no_progress, fast_identify, format);
            let bfs_file = BfsFile::read_bfs_from_file(
                bfs_name.clone(),
                format,
            ).expect("Failed to open BFS file");

            let bar = if no_progress {
                ProgressBar::hidden()
            } else {
                ProgressBar::new(bfs_file.get_file_count() as u64 + 1)
            };
            bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{pos}/{len}]").unwrap().progress_chars("##-"));

            fs::create_dir_all(&output_folder).expect("Failed to create output directory");

            println!("Dumping archive: {}", bfs_name);

            let file_headers = bfs_file.get_file_headers();

            let file = File::open(&bfs_name).expect("Failed to open BFS file");
            let mut reader = BufReader::new(file);

            let mut rebuild_info = HashMap::new();

            let mut file = File::create(format!("{}/00000000.dat", output_folder)).expect("Failed to create dump file");
            raw_extract(&mut reader, &mut file, 0, bfs_file.get_data_offset()).expect("Failed to write dump file");

            rebuild_info.insert(0, "00000000.dat".to_string());

            bar.inc(1);
            if verbose {
                bar.println(format!("\"00000000.dat\" {} bytes", bfs_file.get_data_offset()));
            }

            for file_header in file_headers {
                let mut file = File::create(format!("{}/{:08x}.dat", output_folder, file_header.get_data_offset())).expect("Failed to create dump file");
                raw_extract(&mut reader, &mut file, file_header.get_data_offset(), file_header.get_packed_size()).expect("Failed to write dump file");

                for offset in [file_header.get_data_offset()].iter().chain(&file_header.get_file_copies_offsets()) {
                    rebuild_info.insert(*offset, format!("{:08x}.dat", file_header.get_data_offset()));
                }

                bar.inc(1);
                if verbose {
                    bar.println(format!("\"{:08x}.dat\" {} bytes", file_header.get_data_offset(), file_header.get_packed_size()));
                }
            }

            let file_stem = Path::new(&bfs_name).file_stem().unwrap().to_string_lossy().to_string();
            let file_name = format!("{}/{}.json", output_folder, file_stem);

            let file = File::create(file_name).expect("Failed to dump BFS file");

            serde_json::to_writer(file, &rebuild_info).expect("Failed to dump BFS file");

            bar.finish_and_clear();

            if !no_progress {
                let file_count = bar.length().unwrap_or(1);
                println!("Dumped {} files.", file_count);
            }
        }
        Commands::Rebuild {
            rebuild_info,
            bfs_name,
            verbose,
            no_progress
        } => {
            let file = File::open(&rebuild_info).expect("Failed to open rebuild info");
            let reader = BufReader::new(file);

            let rebuild_info = serde_json::from_reader::<BufReader<File>, HashMap<u32, String>>(reader).expect("Failed to open rebuild info");
            let mut rebuild_info_offset_vec = rebuild_info.keys().cloned().into_iter().collect::<Vec<u32>>();
            rebuild_info_offset_vec.sort_unstable();

            let bfs_file = File::create(&bfs_name).expect("Failed to create BFS file");
            let mut writer = BufWriter::new(bfs_file);

            let bar = if no_progress {
                ProgressBar::hidden()
            } else {
                ProgressBar::new(rebuild_info_offset_vec.len() as u64)
            };
            bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{pos}/{len}]").unwrap().progress_chars("##-"));

            let dump_directory = Path::new(&bfs_name).parent().unwrap().to_path_buf();

            println!("Rebuilding archive: {}", bfs_name);

            for rebuild_info_offset in rebuild_info_offset_vec {
                let rebuild_info_file = rebuild_info.get(&rebuild_info_offset).unwrap();

                let mut rebuild_info_file_path = dump_directory.clone();
                rebuild_info_file_path.push(rebuild_info_file);

                let file = File::open(rebuild_info_file_path).expect("Failed to open dump file");
                let mut reader = BufReader::new(file);

                let current_offset = writer.stream_position().unwrap();

                if (current_offset as u32) < rebuild_info_offset {
                    writer.write_all(&vec![0u8; (rebuild_info_offset - current_offset as u32) as usize]).expect("Failed to write to BFS file");
                }

                let size = io::copy(&mut reader, &mut writer).expect("Failed to write to BFS file");

                bar.inc(1);
                if verbose {
                    bar.println(format!("\"{}\" {} bytes", rebuild_info_file, size));
                }
            }

            bar.finish_and_clear();
            if !no_progress {
                let file_count = bar.length().unwrap_or(1);
                println!("Rebuilt BFS file from {} files.", file_count);
            }
        }
    }
}