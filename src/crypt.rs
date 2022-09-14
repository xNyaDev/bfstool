use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

use crate::{Endianness, u32_from_be_bytes, u32_from_le_bytes};

pub fn read_and_decrypt_block(file_reader: &mut BufReader<File>, key: [u32; 4], endianness: Endianness) -> io::Result<Vec<u32>> {
    let mut buffer = [0; 0x8000];
    file_reader.read_exact(&mut buffer)?;
    let mut vec = buffer.to_vec();
    let mut block_vec = Vec::new();
    for _ in 0..0x2000 {
        block_vec.push(match endianness {
            Endianness::Le => { u32_from_le_bytes(&mut vec) }
            Endianness::Be => { u32_from_be_bytes(&mut vec) }
        });
    }
    decrypt_block(&mut block_vec, key);
    Ok(block_vec)
}

fn decrypt_block(block: &mut Vec<u32>, key: [u32; 4]) { // The algo looks like some variation of TEA
    let last_element_index = block.len() - 1;
    for i in 0..last_element_index {
        let element = block[i];
        let next_element = block[i + 1];
        let temp = next_element.wrapping_add((next_element.wrapping_shl(4)) ^ (next_element >> 5));
        block[i] = element.wrapping_sub(temp ^ get_key(i, key).wrapping_add(0x9e3779b9u32));
    }
    let element = block[last_element_index];
    let next_element = block[0];
    let temp = next_element.wrapping_add((next_element.wrapping_shl(4)) ^ (next_element >> 5));
    block[last_element_index] = element.wrapping_sub(temp ^ get_key(last_element_index, key).wrapping_add(0x9e3779b9u32));
}

fn get_key(i: usize, key: [u32; 4]) -> u32 {
    key[(i ^ 0xFE) & 3]
}

pub fn decrypt_headers_block(block: &mut Vec<u32>, key: [u32; 4]) { // The algo looks like some variation of XXTEA
    let last_element_index = block.len() - 1;
    let rounds = 0x34 / block.len() + 6;
    for round in (1..=rounds).rev() {
        let sum = (round as u32).wrapping_mul(0x9e3779b9u32);
        let sum_key = (sum >> 2) & 3;
        for i in (1..=last_element_index).rev() {
            let element = block[i];
            let prev_element = block[i - 1];
            let temp = prev_element.wrapping_add((prev_element.wrapping_shl(4)) ^ (prev_element >> 5));
            block[i] = element.wrapping_sub(temp ^ get_headers_key(i, sum_key, key).wrapping_add(sum));
        }
        let element = block[0];
        let prev_element = block[last_element_index];
        let temp = prev_element.wrapping_add((prev_element.wrapping_shl(4)) ^ (prev_element >> 5));
        block[0] = element.wrapping_sub(temp ^ get_headers_key(0, sum_key, key).wrapping_add(sum));
    }
}

fn get_headers_key(i: usize, sum_key: u32, key: [u32; 4]) -> u32 {
    key[sum_key as usize ^ i & 3]
}

pub fn create_key(key: [u8; 16], endianness: Endianness) -> [u32; 4] {
    match endianness {
        Endianness::Le => {
            [
                u32::from_le_bytes([key[0], key[1], key[2], key[3]]),
                u32::from_le_bytes([key[4], key[5], key[6], key[7]]),
                u32::from_le_bytes([key[8], key[9], key[10], key[11]]),
                u32::from_le_bytes([key[12], key[13], key[14], key[15]])
            ]
        }
        Endianness::Be => {
            [
                u32::from_be_bytes([key[0], key[1], key[2], key[3]]),
                u32::from_be_bytes([key[4], key[5], key[6], key[7]]),
                u32::from_be_bytes([key[8], key[9], key[10], key[11]]),
                u32::from_be_bytes([key[12], key[13], key[14], key[15]])
            ]
        }
    }
}