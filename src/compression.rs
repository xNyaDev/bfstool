use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, Read, Write};

use flate2::bufread::ZlibDecoder;

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

/// Available compression methods
#[derive(Debug, Default, Eq, PartialEq)]
pub enum CompressionMethod {
    /// No compression
    #[default]
    None,
    /// zlib compression
    Zlib,
}

impl Display for CompressionMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompressionMethod::None => {
                    "none"
                }
                CompressionMethod::Zlib => {
                    "zlib"
                }
            }
        )
    }
}
