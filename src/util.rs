use std::{fs, io};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::CString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::Endianness;

pub trait AsBytes {
    const BYTE_COUNT: usize;
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(self) -> Vec<u8>;
}

pub trait FileHeaderTrait {
    fn get_method(&self) -> u8;
    fn get_data_offset(&self) -> u32;
    fn get_unpacked_size(&self) -> u32;
    fn get_packed_size(&self) -> u32;
    fn get_file_copies_offsets(&self) -> Vec<u32>;
    fn get_file_copies_num(&self) -> (u8, u16);
    fn is_compressed(&self) -> bool;
}

/// Modified Lua 4.0 string hash function
///
/// Used for file identification
///
/// Original at https://www.lua.org/source/4.0/lstring.c.html
///
/// In FO2 GOG/PL exe at 560AB0h
pub fn lua_hash(string: Vec<u8>) -> u32 {
    let mut hash = string.len() as u64;
    let step = (string.len() >> 5) + 1;
    for index in (step..=string.len()).rev().step_by(step) {
        hash ^= (hash << 5) + (hash >> 2) + string.get(index as usize - 1).unwrap_or(&0).clone() as u64;
        hash &= 0xFFFFFFFF; // Prevent overflow, original function works on u32 overflows, obviously not allowed in Rust
    }
    (hash % 0x3E5) as u32
}

/// Create an u32 from the first 4 bytes of a Vec<u8>
pub fn u32_from_le_bytes(bytes: &mut Vec<u8>) -> u32 {
    u32::from_le_bytes((&*(bytes.drain(0..4).collect::<Vec<u8>>())).try_into().unwrap())
}

/// Create an u32 from the first 4 bytes of a Vec<u8>
pub fn u32_from_be_bytes(bytes: &mut Vec<u8>) -> u32 {
    u32::from_be_bytes((&*(bytes.drain(0..4).collect::<Vec<u8>>())).try_into().unwrap())
}

/// Create an u16 from the first 2 bytes of a Vec<u8>
pub fn u16_from_le_bytes(bytes: &mut Vec<u8>) -> u16 {
    u16::from_le_bytes((&*(bytes.drain(0..2).collect::<Vec<u8>>())).try_into().unwrap())
}

/// Create an u8 from the first byte of a Vec<u8>
pub fn u8_from_le_bytes(bytes: &mut Vec<u8>) -> u8 {
    u8::from_le_bytes((&*(bytes.drain(0..1).collect::<Vec<u8>>())).try_into().unwrap())
}

/// Create an u8 from the first 8 bits of a VecDeque<bool>
pub fn u8_from_bits(bits: &mut VecDeque<bool>) -> u8 {
    let mut byte = 0;
    for _ in 0..8 {
        byte <<= 1;
        if bits.pop_front() == Some(true) {
            byte |= 1;
        }
    }
    byte
}

/// Count C characters in a Vec<String>
pub fn c_char_count(strings: Vec<String>) -> HashMap<u8, u32> {
    let mut character_count = HashMap::new();
    for string in strings {
        let c_string = CString::new(string).unwrap();
        for char in c_string.as_bytes() {
            let count = character_count.get(char).unwrap_or(&0).clone();
            character_count.insert(char.clone(), count + 1);
        }
    }
    character_count
}

/// List all files in a directory, recursively
pub fn list_files_recursively<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let mut contents = list_files_recursively(entry.path());
                        result.append(&mut contents);
                    } else {
                        result.push(entry.path().to_string_lossy().to_string().replace("\\", "/"))
                    }
                }
            }
        }
    }
    result
}

/// Get all unique file and folder names
pub fn unique_file_names(paths: Vec<String>) -> Vec<String> {
    let mut uniques = HashSet::new();
    for path in paths {
        if let Some((folder, file)) = path.rsplit_once("/") {
            uniques.insert(folder.to_string());
            uniques.insert(file.to_string());
        }
    }
    uniques.into_iter().collect()
}

/// Remove base path and ensure there are no non-ASCII characters in paths
///
/// Add "data/" prefix if not already present
pub fn sanitize_file_list(base: &String, paths: Vec<String>) -> HashMap<String, String> {
    let base = format!("{}/", base.trim_end_matches('/'));
    paths.into_iter().map(
        |path| {
            let mut new_path = path.strip_prefix(&base).unwrap_or(&path).to_string();
            if !new_path.is_ascii() {
                panic!("One of the files contains a non-ASCII character in its' filename. \nAffected file: {}", path)
            }
            if !new_path.starts_with("data/") {
                new_path.insert_str(0, "data/");
            }
            (new_path, path)
        }
    ).collect()
}

/// Split string into a vec by lines
pub fn string_lines_to_vec(string: String) -> Vec<String> {
    string.lines().into_iter().map(
        |line| {
            line.to_string()
        }
    ).collect()
}

pub fn write_data_to_file_endian(file_writer: &mut BufWriter<File>, data: Vec<u32>, endianness: Endianness) -> io::Result<()> {
    for data in data {
        match endianness {
            Endianness::Le => {
                file_writer.write_all(
                    data.to_le_bytes().as_slice(),
                ).expect("Failed to write to output file");
            }
            Endianness::Be => {
                file_writer.write_all(
                    data.to_be_bytes().as_slice(),
                ).expect("Failed to write to output file");
            }
        }
    }
    Ok(())
}

/// Gets all files from a hash map and orders them by name to hopefully group files loaded together
pub fn get_all_files(lua_hash_files_map: &mut HashMap<u32, Vec<String>>) -> (Vec<&String>, Vec<usize>) {
    let mut all_files = Vec::new();

    for hash in 0..0x3E5 {
        if let Some(files) = lua_hash_files_map.get(&hash) {
            for file_path in files {
                all_files.push(file_path);
            }
        }
    }

    let sorted_indices = get_sorted_indices(all_files.as_slice());
    (all_files, sorted_indices)
}

/// Returns indices of vector in sorted order.
pub fn get_sorted_indices<T: Ord>(data: &[T]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &data[i]);
    indices
}