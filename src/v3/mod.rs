use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use indicatif::ProgressBar;

pub use structs::*;

use crate::{apply_copy_filters, Compression, Format};
use crate::archived_data::zlib_compress;
use crate::bfs::BfsFileTrait;
use crate::filter::apply_filters;
use crate::util::{AsBytes, FileHeaderTrait, lua_hash, sanitize_file_list, unique_file_names};
use crate::v2::util::{create_huffman_tree, huffman_decode, huffman_encode, huffman_tree_to_map};

mod structs;

/// A v3 BFS file is structured this way, starting at `0h`:
///
/// - `BfsHeader` (24 bytes)
///
/// - `Vec<FileInfoTableEntry>` of size `BfsHeader.file_info_table_entry_count * (8 bytes)`
///
/// - `FileNameTableHeader` (20 bytes)
///
/// - `Vec<u32>` where every u32 is an offset from `FileNameTableHeader.huffman_data_offset`,
/// of size `FileNameTableHeader.file_size_table_offset - FileNameTableHeader.file_offset_table_offset`
///
/// - `Vec<u16>` where every u16 is a length of the respective file name
/// of size `FileNameTableHeader.huffman_tree_offset - FileNameTableHeader.file_size_table_offset`
///
/// - `Vec<HuffmanTreeEntry>` - the Huffman tree for file names
/// of size `FileNameTableHeader.huffman_data_offset - FileNameTableHeader.huffman_tree_offset`
///
/// - `Vec<u8>` - the actual Huffman coded data
/// of size `FileNameTableHeader.file_headers_offset - FileNameTableHeader.huffman_data_offset`
///
/// - `Vec<FileHeader>` of size `BfsHeader.data_offset - FileNameTableHeader.file_headers_offset`
///
/// - File data, to file end
#[derive(Default)]
pub struct V3BfsFile {
    /// Path to the BFS file
    bfs_file_path: String,
    /// A map with all the FileHeader indexes for a given folder
    folder_name_map: HashMap<String, Vec<usize>>,
    /// All decoded strings from the file name table
    file_name_table: Vec<Vec<u8>>,
    /// The decoded huffman tree
    huffman_tree_map: HashMap<u32, u8>,
    /// All file names with their headers positions in file_headers
    file_name_to_header_map: HashMap<String, usize>,
    // Actual BFS file starts here
    bfs_header: BfsHeader,
    file_info_table: Vec<FileInfoTableEntry>,
    file_name_table_header: FileNameTableHeader,
    file_name_offset_table: Vec<u32>,
    file_name_size_table: Vec<u16>,
    file_name_huffman_tree: VecDeque<HuffmanTreeEntry>,
    file_name_huffman_data: Vec<u8>,
    file_headers: Vec<FileHeader>,
    // Metadata ends here, after this there's only stored file data
}

impl BfsFileTrait for V3BfsFile {
    fn read_bfs_from_file(path: String, _: Format) -> io::Result<Self> {
        let mut result = Self::default();

        // Read the BFS file to respective fields
        let file = File::open(&path)?;
        let mut file_reader = BufReader::new(file);

        result.bfs_file_path = path;

        // bfs_header
        let mut bfs_header = [0; BfsHeader::BYTE_COUNT];
        file_reader.read_exact(&mut bfs_header)?;
        result.bfs_header = BfsHeader::from_bytes(Vec::from(bfs_header));

        // file_info_table
        for _ in 0..result.bfs_header.file_info_table_entry_count {
            let mut file_info_table_entry = [0; FileInfoTableEntry::BYTE_COUNT];
            file_reader.read_exact(&mut file_info_table_entry)?;
            result.file_info_table.push(FileInfoTableEntry::from_bytes(Vec::from(file_info_table_entry)));
        }

        // file_name_table_header
        let mut file_name_table_header = [0; FileNameTableHeader::BYTE_COUNT];
        file_reader.read_exact(&mut file_name_table_header)?;
        result.file_name_table_header = FileNameTableHeader::from_bytes(Vec::from(file_name_table_header));

        // file_name_offset_table
        for _ in (result.file_name_table_header.file_offset_table_offset..result.file_name_table_header.file_name_size_table_offset).step_by(size_of::<u32>()) {
            let mut offset = [0; size_of::<u32>()];
            file_reader.read_exact(&mut offset)?;
            result.file_name_offset_table.push(u32::from_le_bytes(offset));
        }

        // file_name_size_table
        for _ in (result.file_name_table_header.file_name_size_table_offset..result.file_name_table_header.huffman_tree_offset).step_by(size_of::<u16>()) {
            let mut offset = [0; size_of::<u16>()];
            file_reader.read_exact(&mut offset)?;
            result.file_name_size_table.push(u16::from_le_bytes(offset));
        }

        // file_name_huffman_tree
        for _ in (result.file_name_table_header.huffman_tree_offset..result.file_name_table_header.huffman_data_offset).step_by(HuffmanTreeEntry::BYTE_COUNT) {
            let mut huffman_tree_entry = [0; HuffmanTreeEntry::BYTE_COUNT];
            file_reader.read_exact(&mut huffman_tree_entry)?;
            result.file_name_huffman_tree.push_back(HuffmanTreeEntry::from_bytes(Vec::from(huffman_tree_entry)));
        }

        // file_name_huffman_data
        for _ in (result.file_name_table_header.huffman_data_offset..result.file_name_table_header.file_headers_offset).step_by(size_of::<u8>()) {
            let mut data = [0; size_of::<u8>()];
            file_reader.read_exact(&mut data)?;
            result.file_name_huffman_data.push(u8::from_le_bytes(data));
        }

        // file_headers
        for _ in 0..result.bfs_header.file_count {
            let mut file_header = [0; FileHeader::BYTE_COUNT];
            file_reader.read_exact(&mut file_header)?;
            let mut file_header = FileHeader::from_bytes(Vec::from(file_header));
            for _ in 0..file_header.file_copies {
                let mut offset = [0; size_of::<u32>()];
                file_reader.read_exact(&mut offset)?;
                file_header.file_copies_offsets.push(u32::from_le_bytes(offset));
            }
            for _ in 0..file_header.file_copies_a {
                let mut offset = [0; size_of::<u32>()];
                file_reader.read_exact(&mut offset)?;
                file_header.file_copies_offsets.push(u32::from_le_bytes(offset));
            }
            result.file_headers.push(file_header);
        }

        // BFS file fully read

        // Parse the Huffman Tree
        huffman_tree_to_map(&mut result.file_name_huffman_tree, 1, &mut result.huffman_tree_map);

        // Decode all names
        for index in 0..result.file_name_offset_table.len() {
            // Get the current and next offset for a range of the filename
            let file_name_offset = result.file_name_offset_table.get(index as usize).unwrap().clone();
            let next_offset = if let Some(next_offset) = result.file_name_offset_table.get(index as usize + 1) {
                next_offset.clone()
            } else {
                result.file_name_table_header.file_headers_offset - result.file_name_table_header.huffman_data_offset
            };
            let decoded_size = result.file_name_size_table.get(index as usize).unwrap().clone();
            let encoded_data = result.file_name_huffman_data.clone().splice(file_name_offset as usize..next_offset as usize, []).collect::<Vec<u8>>();

            let decoded_data = huffman_decode(encoded_data, &result.huffman_tree_map, decoded_size);
            result.file_name_table.push(decoded_data);
        }

        // Join file and folder names and save them to a Vec
        // Save the file name and header indexes to a map
        // Save folder name and a header index Vec to a map
        for file_header_index in 0..result.file_headers.len() {
            let file_header = result.file_headers.get(file_header_index).unwrap();

            // Folder ID and file ID are just an index for a value in the decoded table
            let folder_string = result.file_name_table.get(file_header.folder_id as usize).unwrap();
            let folder_string = CString::new(folder_string.clone())?;
            let file_string = result.file_name_table.get(file_header.file_id as usize).unwrap();
            let file_string = CString::new(file_string.clone())?;

            let file_name = format!(
                "{}/{}",
                &folder_string.to_string_lossy().to_string(),
                &file_string.to_string_lossy().to_string()
            );

            result.file_name_to_header_map.insert(file_name, file_header_index);

            let mut header_indexes = result.folder_name_map.get(&folder_string.to_string_lossy().to_string()).cloned().unwrap_or_default();
            header_indexes.push(file_header_index);
            result.folder_name_map.insert(folder_string.to_string_lossy().to_string(), header_indexes);
        }

        Ok(result)
    }

    fn archive(_: Format, bfs_path: String, input_folder_path: String, input_files: Vec<String>, verbose: bool, filters: Vec<String>, copy_filters: Vec<String>, level: Option<u32>, bar: &ProgressBar, file_version: [u8; 4], _deduplicate: bool, _compression: Compression, _align_front: bool, _align_bytes: u32) -> io::Result<()> {
        let mut bfs_file = Self::default();

        bfs_file.bfs_header.magic = 0x31736662; // "bfs1"
        bfs_file.bfs_header.unknown = 1;
        bfs_file.bfs_header.file_version = u32::from_le_bytes(file_version);
        bfs_file.bfs_header.file_info_table_entry_count = 0x3E5; // Always 0x3E5 entries

        bfs_file.bfs_file_path = bfs_path;

        bfs_file.bfs_header.file_count = input_files.len() as u32;

        let filenames = sanitize_file_list(&format!("{}/", input_folder_path.replace("\\", "/")), input_files);

        let mut lua_hash_count_map = HashMap::new();
        let mut lua_hash_header_size_map = HashMap::new();
        let mut lua_hash_files_map = HashMap::new();

        let copy_filters = apply_copy_filters(
            filenames.keys().cloned().collect(),
            copy_filters,
        );

        filenames.keys().cloned().for_each(|name| {
            let c_name = CString::new(name.clone()).unwrap();
            let hash = lua_hash(c_name.into_bytes());
            let header_size = lua_hash_header_size_map.get(&hash).unwrap_or(&0).clone();
            let (file_copies, file_copies_a) = copy_filters.get(&name).unwrap().clone();
            let new_header_size = header_size + FileHeader::BYTE_COUNT as u32 + (file_copies as u32 * 4) + (file_copies_a as u32 * 4);
            lua_hash_header_size_map.insert(hash, new_header_size);
            let count = lua_hash_count_map.get(&hash).unwrap_or(&0).clone();
            lua_hash_count_map.insert(hash, count + 1);
            let mut files = lua_hash_files_map.get(&hash).unwrap_or(&Vec::new()).clone();
            files.push(name);
            lua_hash_files_map.insert(hash, files);
        });

        let mut uniques = unique_file_names(
            filenames.keys().cloned().collect()
        );
        uniques.sort_unstable();

        let tree = create_huffman_tree(uniques.clone());
        let mut tree_reader = BufReader::new(tree.as_slice());

        for _ in (0..tree.len()).step_by(HuffmanTreeEntry::BYTE_COUNT) {
            let mut huffman_tree_entry = [0; HuffmanTreeEntry::BYTE_COUNT];
            tree_reader.read_exact(&mut huffman_tree_entry)?;
            bfs_file.file_name_huffman_tree.push_back(HuffmanTreeEntry::from_bytes(Vec::from(huffman_tree_entry)));
        }

        huffman_tree_to_map(&mut bfs_file.file_name_huffman_tree.clone(), 1, &mut bfs_file.huffman_tree_map);
        let encoding_map = bfs_file.huffman_tree_map.into_iter().map(|(k, v)| (v, k)).collect::<HashMap<u8, u32>>();

        let name_ids = uniques.into_iter().map(
            |name| {
                let c_name = CString::new(name.clone()).unwrap();
                bfs_file.file_name_offset_table.push(bfs_file.file_name_huffman_data.len() as u32);
                bfs_file.file_name_size_table.push(c_name.clone().into_bytes().len() as u16);
                let mut encoded = huffman_encode(c_name.into_bytes(), &encoding_map);
                bfs_file.file_name_huffman_data.append(&mut encoded);
                (name, bfs_file.file_name_offset_table.len() as u16 - 1)
            }
        ).collect::<HashMap<String, u16>>();

        // Calculate all offsets

        bfs_file.file_name_table_header.file_offset_table_offset = FileNameTableHeader::BYTE_COUNT as u32;

        bfs_file.file_name_table_header.file_name_size_table_offset =
            bfs_file.file_name_table_header.file_offset_table_offset +
                bfs_file.file_name_offset_table.len() as u32 * 4;

        bfs_file.file_name_table_header.huffman_tree_offset =
            bfs_file.file_name_table_header.file_name_size_table_offset +
                bfs_file.file_name_size_table.len() as u32 * 2;

        bfs_file.file_name_table_header.huffman_data_offset =
            bfs_file.file_name_table_header.huffman_tree_offset +
                (bfs_file.file_name_huffman_tree.len() * HuffmanTreeEntry::BYTE_COUNT) as u32;

        bfs_file.file_name_table_header.file_headers_offset =
            bfs_file.file_name_table_header.huffman_data_offset +
                bfs_file.file_name_huffman_data.len() as u32;

        let file_info_table_size = FileInfoTableEntry::BYTE_COUNT as u32 * 0x3E5;
        let mut file_headers_size = 0;
        for hash in 0..0x3E5 {
            let header_size = lua_hash_header_size_map.get(&hash).unwrap_or(&0).clone();
            file_headers_size += header_size;
        }

        bfs_file.bfs_header.data_offset = 0x80000000 |
            bfs_file.file_name_table_header.file_headers_offset +
                BfsHeader::BYTE_COUNT as u32 +
                file_info_table_size +
                file_headers_size;

        let mut header_offset =
            bfs_file.file_name_table_header.file_headers_offset +
                BfsHeader::BYTE_COUNT as u32 +
                file_info_table_size;

        for hash in 0..0x3E5 {
            let file_count = lua_hash_count_map.get(&hash).unwrap_or(&0).clone();
            let header_size = lua_hash_header_size_map.get(&hash).unwrap_or(&0).clone();
            bfs_file.file_info_table.push(
                FileInfoTableEntry {
                    file_header_offset: if file_count == 0 {
                        0
                    } else {
                        header_offset
                    },
                    file_count,
                }
            );
            header_offset += header_size;
        }

        let file = File::create(bfs_file.bfs_file_path)?;
        let mut file_writer = BufWriter::new(file);

        // Empty values where the metadata will be later
        file_writer.write_all(&vec![0u8; (bfs_file.bfs_header.data_offset & 0x7FFFFFFF) as usize])?;

        let files_to_compress = apply_filters(
            filenames.keys().cloned().collect(),
            filters,
        );

        for hash in 0..0x3E5 {
            if let Some(files) = lua_hash_files_map.get(&hash) {
                for file_path in files {
                    let original_file_path = filenames.get(file_path).unwrap();
                    let mut file = File::open(original_file_path)?;
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    let (file_copies, file_copies_a) = copy_filters.get(file_path).unwrap().clone();

                    let mut file_header = FileHeader {
                        method: 0,
                        file_copies,
                        file_copies_a,
                        data_offset: file_writer.stream_position()? as u32,
                        unpacked_size: data.len() as u32,
                        packed_size: 0,
                        crc32: 0,
                        folder_id: 0,
                        file_id: 0,
                        file_copies_offsets: vec![],
                    };
                    if let Some((folder, file)) = file_path.rsplit_once("/") {
                        file_header.folder_id = name_ids.get(folder).unwrap().clone();
                        file_header.file_id = name_ids.get(file).unwrap().clone();
                    }
                    let status;
                    if files_to_compress.contains(file_path) && level != Some(0) {
                        file_header.method = 1; // zlib
                        let compressed_data = zlib_compress(data, level)?;
                        file_header.packed_size = io::copy(&mut compressed_data.as_slice(), &mut file_writer)? as u32;
                        for _ in 0..(file_header.file_copies as u16 + file_header.file_copies_a) {
                            file_header.file_copies_offsets.push(file_writer.stream_position()? as u32);
                            io::copy(&mut compressed_data.as_slice(), &mut file_writer)?;
                        }
                        status = format!("{} -> {} bytes", file_header.unpacked_size, file_header.packed_size);
                    } else {
                        file_header.method = 0; // store
                        file_header.packed_size = file_header.unpacked_size;
                        io::copy(&mut data.as_slice(), &mut file_writer)?;
                        for _ in 0..(file_header.file_copies as u16 + file_header.file_copies_a) {
                            file_header.file_copies_offsets.push(file_writer.stream_position()? as u32);
                            io::copy(&mut data.as_slice(), &mut file_writer)?;
                        }
                        status = format!("{} bytes", file_header.unpacked_size);
                    }

                    if verbose {
                        bar.println(format!("{file_path:?} {status}"));
                    }
                    bar.inc(1);

                    bfs_file.file_headers.push(file_header);
                }
            }
        }

        if verbose {
            bar.println("Writing headers");
        }

        file_writer.seek(SeekFrom::Start(0))?;

        file_writer.write_all(&bfs_file.bfs_header.to_bytes())?;
        for file_info_table_entry in bfs_file.file_info_table {
            file_writer.write_all(&file_info_table_entry.to_bytes())?;
        }
        file_writer.write_all(&bfs_file.file_name_table_header.to_bytes())?;
        for file_name_offset in bfs_file.file_name_offset_table {
            file_writer.write_all(&file_name_offset.to_le_bytes())?;
        }
        for file_name_size in bfs_file.file_name_size_table {
            file_writer.write_all(&file_name_size.to_le_bytes())?;
        }
        for huffman_tree_entry in bfs_file.file_name_huffman_tree {
            file_writer.write_all(&huffman_tree_entry.to_bytes())?;
        }
        file_writer.write_all(&bfs_file.file_name_huffman_data)?;
        for file_header in bfs_file.file_headers {
            file_writer.write_all(&file_header.to_bytes())?;
        }

        Ok(())
    }

    fn get_file_count(&self) -> u32 {
        self.bfs_header.file_count
    }

    fn get_data_offset(&self) -> u32 {
        self.bfs_header.data_offset & 0x7FFFFFFF
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