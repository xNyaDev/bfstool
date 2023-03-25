use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use flate2::bufread::ZlibDecoder;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub fn zstd_extract(reader: &mut BufReader<File>, writer: &mut File, reader_offset: u32, compressed_size: u32) -> io::Result<usize> {
    reader.seek(SeekFrom::Start(reader_offset as u64))?;
    let compressed_data = reader.take(compressed_size as u64);
    let mut decoder = zstd::Decoder::new(compressed_data)?;
    Ok(io::copy(&mut decoder, writer)? as usize)
}

pub fn lz4_extract(reader: &mut BufReader<File>, writer: &mut File, reader_offset: u32, compressed_size: u32) -> io::Result<usize> {
    reader.seek(SeekFrom::Start(reader_offset as u64))?;
    let compressed_data = reader.take(compressed_size as u64);
    let mut decoder = lz4::Decoder::new(compressed_data)?;
    Ok(io::copy(&mut decoder, writer)? as usize)
}

pub fn zlib_extract(reader: &mut BufReader<File>, writer: &mut File, reader_offset: u32, compressed_size: u32) -> io::Result<usize> {
    reader.seek(SeekFrom::Start(reader_offset as u64))?;
    let compressed_data = reader.take(compressed_size as u64);
    let mut decoder = ZlibDecoder::new(compressed_data);
    Ok(io::copy(&mut decoder, writer)? as usize)
}

pub fn raw_extract(reader: &mut BufReader<File>, writer: &mut File, reader_offset: u32, size: u32) -> io::Result<usize> {
    reader.seek(SeekFrom::Start(reader_offset as u64))?;
    let mut data = reader.take(size as u64);
    Ok(io::copy(&mut data, writer)? as usize)
}

pub fn zlib_compress(data: Vec<u8>, level: Option<u32>) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(
        Vec::new(),
        if let Some(level) = level {
            Compression::new(level)
        } else {
            Compression::default()
        },
    );
    encoder.write_all(&data)?;
    Ok(encoder.finish()?)
}