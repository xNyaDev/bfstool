use std::collections::HashMap;
use std::io;

use indicatif::ProgressBar;

use crate::Format;
use crate::util::FileHeaderTrait;
use crate::v1::V1BfsFile;
use crate::v2::V2BfsFile;
use crate::v3::V3BfsFile;

pub enum BfsFile {
    V1BfsFile(V1BfsFile),
    V2BfsFile(V2BfsFile),
    V3BfsFile(V3BfsFile),
}

impl BfsFileTrait for BfsFile {
    fn read_bfs_from_file(path: String, format: Format) -> io::Result<Self> {
        Ok(match format {
            Format::V1 | Format::V1a => {
                BfsFile::V1BfsFile(V1BfsFile::read_bfs_from_file(path, format)?)
            }
            Format::V2 | Format::V2a => {
                BfsFile::V2BfsFile(V2BfsFile::read_bfs_from_file(path, format)?)
            }
            Format::V3 => {
                BfsFile::V3BfsFile(V3BfsFile::read_bfs_from_file(path, format)?)
            }
        })
    }

    fn archive(format: Format, bfs_path: String, input_folder_path: String, input_files: Vec<String>, verbose: bool, filters: Vec<String>, copy_filters: Vec<String>, level: Option<u32>, bar: &ProgressBar, file_version: [u8; 4]) -> io::Result<()> {
        match format {
            Format::V1 | Format::V1a => {
                V1BfsFile::archive(format, bfs_path, input_folder_path, input_files, verbose, filters, copy_filters, level, bar, file_version)
            }
            Format::V2 | Format::V2a => {
                V2BfsFile::archive(format, bfs_path, input_folder_path, input_files, verbose, filters, copy_filters, level, bar, file_version)
            }
            Format::V3 => {
                V3BfsFile::archive(format, bfs_path, input_folder_path, input_files, verbose, filters, copy_filters, level, bar, file_version)
            }
        }
    }

    fn get_file_count(&self) -> u32 {
        match self {
            BfsFile::V1BfsFile(bfs_file) => bfs_file.get_file_count(),
            BfsFile::V2BfsFile(bfs_file) => bfs_file.get_file_count(),
            BfsFile::V3BfsFile(bfs_file) => bfs_file.get_file_count(),
        }
    }

    fn get_data_offset(&self) -> u32 {
        match self {
            BfsFile::V1BfsFile(bfs_file) => bfs_file.get_data_offset(),
            BfsFile::V2BfsFile(bfs_file) => bfs_file.get_data_offset(),
            BfsFile::V3BfsFile(bfs_file) => bfs_file.get_data_offset(),
        }
    }

    fn get_file_headers(&self) -> Vec<Box<dyn FileHeaderTrait>> {
        match self {
            BfsFile::V1BfsFile(bfs_file) => bfs_file.get_file_headers(),
            BfsFile::V2BfsFile(bfs_file) => bfs_file.get_file_headers(),
            BfsFile::V3BfsFile(bfs_file) => bfs_file.get_file_headers(),
        }
    }

    fn get_file_name_to_header_map(&self) -> &HashMap<String, usize> {
        match self {
            BfsFile::V1BfsFile(bfs_file) => bfs_file.get_file_name_to_header_map(),
            BfsFile::V2BfsFile(bfs_file) => bfs_file.get_file_name_to_header_map(),
            BfsFile::V3BfsFile(bfs_file) => bfs_file.get_file_name_to_header_map(),
        }
    }
}

pub trait BfsFileTrait: Sized {
    fn read_bfs_from_file(path: String, format: Format) -> io::Result<Self>;
    fn archive(format: Format, bfs_path: String, input_folder_path: String, input_files: Vec<String>, verbose: bool, filters: Vec<String>, copy_filters: Vec<String>, level: Option<u32>, bar: &ProgressBar, file_version: [u8; 4]) -> io::Result<()>;
    fn get_file_count(&self) -> u32;
    fn get_data_offset(&self) -> u32;
    fn get_file_headers(&self) -> Vec<Box<dyn FileHeaderTrait>>;
    fn get_file_name_to_header_map(&self) -> &HashMap<String, usize>;
}