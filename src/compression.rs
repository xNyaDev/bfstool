use flate2::bufread::ZlibDecoder;
use std::io;
use std::io::{BufRead, Read, Write};

use crate::CompressionMethod;

pub fn extract_data<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    size: u64,
    method: CompressionMethod,
) -> io::Result<u64> {
    let mut data = reader.take(size);
    match method {
        CompressionMethod::None => io::copy(&mut data, writer),
        CompressionMethod::Zlib => {
            let mut decoder = ZlibDecoder::new(data);
            io::copy(&mut decoder, writer)
        }
    }
}
