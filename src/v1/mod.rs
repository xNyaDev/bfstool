use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use crc::{Crc, CRC_32_JAMCRC};
use indicatif::ProgressBar;
use xxhash_rust::xxh64::xxh64;

pub use structs::*;

use crate::{apply_copy_filters, Compression, Format};
use crate::archived_data::zlib_compress;
use crate::bfs::BfsFileTrait;
use crate::filter::apply_filters;
use crate::util::{AsBytes, FileHeaderTrait, lua_hash, sanitize_file_list, u32_from_le_bytes};

mod structs;

/// A v1 BFS file is structured this way, starting at `0h`:
///
/// - `BfsHeader` (16 bytes)
///
/// - `Vec<u32>` - File header offsets, of size `BfsHeader.file_count * (4 bytes)`. Every offset is
/// from `0h`
///
/// - `u32` - `file_info_table_entry_count` (4 bytes)
///
/// - `Vec<FileInfoTableEntry>` of size `file_info_table_entry_count * (4 bytes)`
///
/// - `Vec<(FileHeader, Vec<u8>)>` up until `BfsHeader.file_header_end_offset` - one header,
/// one filename
///
/// - File data, to file end
///
/// FOV3 mod bfs contains invalid filenames - size 0 and illegal ASCII characters
///
/// Those will be extracted as `{offset}.dat`
#[derive(Default)]
pub struct V1BfsFile {
    /// Path to the BFS file
    bfs_file_path: String,
    /// All file names with their headers positions in file_headers
    file_name_to_header_map: HashMap<String, usize>,
    // Actual BFS file starts here
    // common1.bfs:
    // bfs_header: 0h-Fh
    // file_header_offsets: 10h-117Fh
    // file_info_table_entry_count: 1180h-1183h
    // file_info_table: 1184h-2117h
    // file_headers: 2118h-101DCh
    // There are also 3 null bytes after file headers, so file data starts at 101E0h
    bfs_header: BfsHeader,
    file_header_offset_table: Vec<u32>,
    file_info_table_entry_count: u32,
    file_info_table: Vec<FileInfoTableEntry>,
    file_headers: Vec<FileHeader>,
    raw_file_names: Vec<Vec<u8>>,
    // Metadata ends here, after this there's only stored file data
}

impl BfsFileTrait for V1BfsFile {
    fn read_bfs_from_file(path: String, format: Format) -> io::Result<Self> {
        let mut result = Self::default();

        // Read the BFS file to respective fields
        let file = File::open(&path)?;
        let mut file_reader = BufReader::new(file);

        result.bfs_file_path = path;

        // bfs_header
        let mut bfs_header = [0; BfsHeader::BYTE_COUNT];
        file_reader.read_exact(&mut bfs_header)?;
        result.bfs_header = BfsHeader::from_bytes(Vec::from(bfs_header));

        // file_header_offset_table
        for _ in 0..result.bfs_header.file_count {
            let mut offset = [0; size_of::<u32>()];
            file_reader.read_exact(&mut offset)?;
            result.file_header_offset_table.push(u32_from_le_bytes(&mut Vec::from(offset)));
        }

        // file_info_table_entry_count
        let mut file_info_table_entry_count = [0; size_of::<u32>()];
        file_reader.read_exact(&mut file_info_table_entry_count)?;
        result.file_info_table_entry_count = u32_from_le_bytes(&mut Vec::from(file_info_table_entry_count));

        // file_info_table
        for _ in 0..result.file_info_table_entry_count {
            let mut file_info_table_entry = [0; FileInfoTableEntry::BYTE_COUNT];
            file_reader.read_exact(&mut file_info_table_entry)?;
            result.file_info_table.push(FileInfoTableEntry::from_bytes(Vec::from(file_info_table_entry)));
        }

        // file_headers and file_names
        for _ in 0..result.bfs_header.file_count {
            let mut file_header = [0; FileHeader::BYTE_COUNT];
            file_reader.read_exact(&mut file_header)?;
            let mut file_header = FileHeader::from_bytes(Vec::from(file_header));
            let mut file_name = vec![0u8; file_header.file_name_length as usize];
            file_reader.read_exact(&mut file_name)?;
            for _ in 0..file_header.file_copies {
                let mut offset = [0; size_of::<u32>()];
                file_reader.read_exact(&mut offset)?;
                file_header.file_copies_offsets.push(u32::from_le_bytes(offset));
            }
            result.file_headers.push(file_header);
            result.raw_file_names.push(file_name);
        }

        // BFS file fully read

        // Convert filenames
        for index in 0..result.bfs_header.file_count {
            let raw_file_name = result.raw_file_names.get(index as usize).unwrap();
            let file_name = CString::new(raw_file_name.clone())?;
            let mut is_valid = true;
            for byte in file_name.as_bytes() {
                if byte < &0x20 || byte >= &0x7F { // Valid ascii characters are from Space (0x20), inclusive, to DEL (0x7F), not inclusive
                    is_valid = false;
                }
            }
            if file_name.as_bytes().len() == 0 { // Empty file names can't be valid
                is_valid = false;
            }
            if is_valid {
                result.file_name_to_header_map.insert(
                    file_name.to_string_lossy().to_string(),
                    index as usize,
                );
            } else {
                let file_header = result.file_headers.get(index as usize).unwrap();
                let file_name = format!("{:08x}.dat", file_header.data_offset);
                println!("Invalid file name detected - {file_name} will be used instead");
                result.file_name_to_header_map.insert(
                    file_name,
                    index as usize,
                );
            }
        }

        if (result.file_headers.get(0).unwrap().method == 0 ||
            result.file_headers.get(0).unwrap().method == 1
        ) && format == Format::V1 {
            println!("File is in the v1a format and v1 was selected.");
            println!("Listing and extraction will work, but created archives may fail to load");
        } else if (result.file_headers.get(0).unwrap().method == 4 ||
            result.file_headers.get(0).unwrap().method == 5
        ) && format == Format::V1a {
            println!("File is in the v1 format and v1a was selected.");
            println!("Listing and extraction will work, but created archives may fail to load");
        }

        Ok(result)
    }

    fn archive(format: Format, bfs_path: String, input_folder_path: String, input_files: Vec<String>, verbose: bool, filters: Vec<String>, copy_filters: Vec<String>, level: Option<u32>, bar: &ProgressBar, file_version: [u8; 4], deduplicate: bool, _compression: Compression, _align_front: bool, _align_bytes: u32) -> io::Result<()> {
        let mut bfs_file = V1BfsFile::default();

        bfs_file.bfs_header.magic = 0x31736662; // "bfs1"
        bfs_file.bfs_header.file_version = u32::from_le_bytes(file_version);
        bfs_file.file_info_table_entry_count = 0x3E5; // Always 0x3E5 entries

        bfs_file.bfs_file_path = bfs_path;

        bfs_file.bfs_header.file_count = input_files.len() as u32;

        let file_names = sanitize_file_list(&format!("{}/", input_folder_path.replace("\\", "/")), input_files);

        let mut name_lua_hash_map = HashMap::new();
        let mut lua_hash_count_map = HashMap::new();
        let mut lua_hash_header_size_map = HashMap::new();

        let copy_filters = apply_copy_filters(
            file_names.keys().cloned().collect(),
            copy_filters,
        );

        file_names.keys().cloned().for_each(|name| {
            let c_name = CString::new(name.clone()).unwrap();
            let hash = lua_hash(c_name.into_bytes());
            name_lua_hash_map.insert(name.clone(), hash);
            let header_size = lua_hash_header_size_map.get(&hash).unwrap_or(&0).clone();
            let (file_copies, file_copies_a) = copy_filters.get(&name).unwrap().clone();
            let new_header_size = header_size + FileHeader::BYTE_COUNT as u32 + (file_copies as u32 * 4) + (file_copies_a as u32 * 4);
            lua_hash_header_size_map.insert(hash, new_header_size);
            let count = lua_hash_count_map.get(&hash).unwrap_or(&0).clone();
            lua_hash_count_map.insert(hash, count + 1);
        });

        let mut file_count_so_far = 0;

        for hash in 0..bfs_file.file_info_table_entry_count {
            let file_count = lua_hash_count_map.get(&hash).unwrap_or(&0).clone();
            bfs_file.file_info_table.push(
                FileInfoTableEntry {
                    starting_file: if file_count == 0 {
                        0
                    } else {
                        file_count_so_far
                    },
                    file_count,
                }
            );
            file_count_so_far += file_count;
        }

        // Calculate the end offset for headers

        let mut file_names_size = 0;
        file_names.keys().cloned().for_each(|name| {
            file_names_size += name.len();
        });

        let file_header_start_offset = BfsHeader::BYTE_COUNT as u32 +
            size_of::<u32>() as u32 * bfs_file.bfs_header.file_count +
            size_of::<u32>() as u32 +
            FileInfoTableEntry::BYTE_COUNT as u32 * bfs_file.file_info_table_entry_count;

        let mut file_headers_size = 0;
        for hash in 0..0x3E5 {
            let header_size = lua_hash_header_size_map.get(&hash).unwrap_or(&0).clone();
            file_headers_size += header_size;
        }

        bfs_file.bfs_header.data_offset = file_header_start_offset +
            file_headers_size +
            file_names_size as u32;

        let file = File::create(bfs_file.bfs_file_path)?;
        let mut file_writer = BufWriter::new(file);

        let data_start = (bfs_file.bfs_header.data_offset as usize + 3) & !3;

        // Empty values where the metadata will be later
        file_writer.write_all(&vec![0u8; data_start])?;

        let files_to_compress = apply_filters(
            file_names.keys().cloned().collect(),
            filters,
        );

        // Pack the files

        const JAMCRC: Crc<u32> = Crc::<u32>::new(&CRC_32_JAMCRC);
        let mut dedupe_hash_to_header = HashMap::<u64, FileHeader>::new();

        let mut sorted_file_names = file_names.keys().cloned().collect::<Vec<String>>();
        sorted_file_names.sort_unstable();
        let mut current_file_header_offset = file_header_start_offset;

        let mut hash_header_offsets_map = HashMap::new();

        for sorted_file_name_index in 0..sorted_file_names.len() {
            let file_name = sorted_file_names.get(sorted_file_name_index).unwrap();
            let file_path = file_names.get(file_name).unwrap();

            let hash = name_lua_hash_map.get(file_name).unwrap().clone();
            let mut headers_for_hash = hash_header_offsets_map.get(&hash).unwrap_or(&Vec::new()).clone();
            headers_for_hash.push(current_file_header_offset);
            hash_header_offsets_map.insert(hash, headers_for_hash);

            let mut file = File::open(file_path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            current_file_header_offset += FileHeader::BYTE_COUNT as u32 +
                file_name.len() as u32;

            let (file_copies, file_copies_a) = copy_filters.get(file_name).unwrap().clone();

            let mut file_header = FileHeader {
                method: 0,
                file_copies,
                file_copies_a,
                data_offset: file_writer.stream_position()? as u32,
                unpacked_size: data.len() as u32,
                packed_size: 0,
                crc32: 0,
                file_name_length: file_name.len() as u16,
                file_copies_offsets: vec![],
            };

            let mut status = String::new();
            if deduplicate {
                // Note: We hash separately using a hash with longer value as I (Sewer) don't like
                // probability of collision with 32-bit hash.
                let dedupe_hash: u64 = xxh64(&data, 0);

                // Note: We have to account for the case where one file is compressed but another file isn't, so make
                // sure the compress state matches existing file.
                let should_compress_file = Self::should_compress_file(level, &files_to_compress, file_path);

                if let Some(cached_header) = dedupe_hash_to_header.get(&dedupe_hash) {
                    if should_compress_file == cached_header.is_compressed() && cached_header.unpacked_size == file_header.unpacked_size {
                        file_header.crc32 = cached_header.crc32;
                        file_header.method = cached_header.method;
                        file_header.packed_size = cached_header.packed_size;
                        file_header.data_offset = cached_header.data_offset;
                        status = format!("{} bytes, deduplicated", file_header.packed_size);
                    }
                    else {
                        Self::write_file_to_output(&format, level, &mut file_writer, &files_to_compress, file_name, data, &mut file_header, &mut status, JAMCRC)?;
                    }
                }
                else {
                    Self::write_file_to_output(&format, level, &mut file_writer, &files_to_compress, file_name, data, &mut file_header, &mut status, JAMCRC)?;
                    dedupe_hash_to_header.insert(dedupe_hash, file_header.clone());
                }
            }
            else {
                Self::write_file_to_output(&format, level, &mut file_writer, &files_to_compress, file_name, data, &mut file_header, &mut status, JAMCRC)?;
            }
            
            bfs_file.file_headers.push(file_header);
            let c_name = CString::new(file_name.clone()).unwrap();
            bfs_file.raw_file_names.push(c_name.into_bytes());

            if verbose {
                bar.println(format!("{file_name:?} {status}"));
            }
            bar.inc(1);
        }

        for hash in 0..0x3E5 {
            let headers_for_hash = hash_header_offsets_map.get(&hash).unwrap_or(&Vec::new()).clone();
            for offset in headers_for_hash {
                bfs_file.file_header_offset_table.push(offset);
            }
        }

        if verbose {
            bar.println("Writing headers");
        }

        file_writer.seek(SeekFrom::Start(0))?;

        let file_count = bfs_file.bfs_header.file_count;

        file_writer.write_all(&bfs_file.bfs_header.to_bytes())?;
        for file_header_offset_table in bfs_file.file_header_offset_table {
            file_writer.write_all(&file_header_offset_table.to_le_bytes())?;
        }
        file_writer.write_all(&bfs_file.file_info_table_entry_count.to_le_bytes())?;
        for file_info_table_entry in bfs_file.file_info_table {
            file_writer.write_all(&file_info_table_entry.to_bytes())?;
        }

        for index in 0..file_count {
            let file_header = bfs_file.file_headers.get(index as usize).unwrap().clone();
            let file_name = bfs_file.raw_file_names.get(index as usize).unwrap();
            file_writer.write_all(&file_header.to_bytes())?;
            file_writer.write_all(file_name.as_slice())?;
        }

        Ok(())
    }

    fn get_file_count(&self) -> u32 {
        self.bfs_header.file_count
    }

    fn get_data_offset(&self) -> u32 {
        self.bfs_header.data_offset
    }

    fn get_file_headers(&self) -> Vec<Box<dyn FileHeaderTrait>> {
        self.file_headers.iter().map(|file_header| {
            Box::new(file_header.clone()) as Box<dyn FileHeaderTrait>
        }).collect()
    }

    fn get_file_name_to_header_map(&self) -> &HashMap<String, usize> {
        &self.file_name_to_header_map
    }

    fn get_file_version(&self) -> u32 {
        self.bfs_header.file_version
    }
}

impl V1BfsFile {
    fn write_file_to_output(format: &Format, level: Option<u32>, mut file_writer: &mut BufWriter<File>,
                            files_to_compress: &Vec<String>, file_name: &String, data: Vec<u8>,
                            file_header: &mut FileHeader, status: &mut String,
                            crc: Crc<u32>) -> io::Result<()> {
        if Self::should_compress_file(level, files_to_compress, file_name) {
            file_header.method = if format == &Format::V1 {
                5
            } else {
                1
            }; // zlib
            let compressed_data = zlib_compress(data, level)?;
            file_header.crc32 = if format == &Format::V1 {
                crc.checksum(&compressed_data)
            } else {
                0
            };
            file_header.packed_size = io::copy(&mut compressed_data.as_slice(), &mut file_writer)? as u32;
            for _ in 0..(file_header.file_copies as u16 + file_header.file_copies_a) {
                file_header.file_copies_offsets.push(file_writer.stream_position()? as u32);
                io::copy(&mut compressed_data.as_slice(), &mut file_writer)?;
            }
            *status = format!("{} -> {} bytes", file_header.unpacked_size, file_header.packed_size);
        } else {
            file_header.method = if format == &Format::V1 {
                4
            } else {
                0
            }; // store
            file_header.crc32 = if format == &Format::V1 {
                crc.checksum(&data)
            } else {
                0
            };
            file_header.packed_size = file_header.unpacked_size;
            io::copy(&mut data.as_slice(), &mut file_writer)?;
            for _ in 0..(file_header.file_copies as u16 + file_header.file_copies_a) {
                file_header.file_copies_offsets.push(file_writer.stream_position()? as u32);
                io::copy(&mut data.as_slice(), &mut file_writer)?;
            }
            *status = format!("{} bytes", file_header.unpacked_size);
        }

        Ok(())
    }
    
    fn should_compress_file(level: Option<u32>, files_to_compress: &Vec<String>, file_name: &String) -> bool {
        files_to_compress.contains(file_name) && level != Some(0)
    }
}