use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::crypt::CryptError;
use crate::crypt::bfs2011::{HeaderKey, Key};
use crate::crypt::xxtea::xxtea_encode;
use crate::formats::bfs2011::ArchiveHeader;
use binrw::BinRead;

/// Encrypt one 32 KiB block
fn encrypt_block<R: BufRead + Seek + 'static>(
    input: &mut R,
    key: Key,
    big_endian: bool,
) -> io::Result<Vec<u8>> {
    let mut data = [0; 0x8000];

    input.read_exact(&mut data)?;

    let map_fn = if big_endian {
        |x: &[u8]| u32::from_be_bytes([x[0], x[1], x[2], x[3]])
    } else {
        |x: &[u8]| u32::from_le_bytes([x[0], x[1], x[2], x[3]])
    };

    let mut data = data.chunks_exact(4).map(map_fn).collect::<Vec<_>>();
    xxtea_encode(
        &mut data,
        key.chunks_exact(4)
            .map(|x: &[u8]| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        super::MXFN,
        super::KEYFN,
        true,
        Some(1),
    );

    let map_fn = if big_endian {
        u32::to_be_bytes
    } else {
        u32::to_le_bytes
    };

    Ok(data.into_iter().flat_map(map_fn).collect::<Vec<u8>>())
}

/// Encrypt a bfs2011 archive and write it into `output`
pub fn encrypt<R: BufRead + Seek + 'static, W: Write + Seek + 'static>(
    mut input: R,
    output: &mut BufWriter<W>,
    key: Key,
    header_key: HeaderKey,
    big_endian: bool,
) -> Result<(), CryptError> {
    input.seek(SeekFrom::Start(0))?;
    output.seek(SeekFrom::Start(0))?;

    let mut archive_header = [0; 0x14];
    input.read_exact(&mut archive_header)?;

    let archive_header = ArchiveHeader::read(&mut Cursor::new(archive_header))?;

    input.seek(SeekFrom::Start(0))?;

    let headers_size = ((archive_header.header_end & 0x7FFFFFFF) as usize - 0x14 + 3) & !3; // Skip the archive header and pad to next multiple of 4 bytes

    let mut data = vec![];
    while data.len() < headers_size {
        let mut new_data = [0; 0x8000];
        input.read_exact(&mut new_data)?;
        let mut new_data = new_data.to_vec();
        data.append(&mut new_data);
    }

    let headers_data = data[0x14..(headers_size + 0x14)].to_vec();
    let map_fn = if big_endian {
        |x: &[u8]| u32::from_be_bytes([x[0], x[1], x[2], x[3]])
    } else {
        |x: &[u8]| u32::from_le_bytes([x[0], x[1], x[2], x[3]])
    };

    let mut headers_data = headers_data.chunks_exact(4).map(map_fn).collect::<Vec<_>>();
    xxtea_encode(
        &mut headers_data,
        header_key
            .chunks_exact(4)
            .map(|x: &[u8]| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        super::HEADERSMXFN,
        super::KEYFN,
        false,
        None,
    );

    let map_fn = if big_endian {
        u32::to_be_bytes
    } else {
        u32::to_le_bytes
    };

    let headers_data = headers_data
        .into_iter()
        .flat_map(map_fn)
        .collect::<Vec<u8>>();

    data[0x14..(headers_data.len() + 0x14)].copy_from_slice(&headers_data);

    let mut data_reader = BufReader::new(Cursor::new(data));
    while let Ok(data) = encrypt_block(&mut data_reader, key, big_endian) {
        output.write_all(&data)?;
    }

    while let Ok(data) = encrypt_block(&mut input, key, big_endian) {
        output.write_all(&data)?;
    }

    Ok(())
}

/// Encrypt a bfs2011 archive and write it into `output`
///
/// Utility function that opens the input file, creates the output file and calls `encrypt` on those
pub fn encrypt_file(
    input: PathBuf,
    output: PathBuf,
    key: Key,
    header_key: HeaderKey,
    big_endian: bool,
) -> Result<(), CryptError> {
    let input = File::open(input)?;
    let input = BufReader::new(input);

    let output = File::create(output)?;
    let mut output = BufWriter::new(output);

    encrypt(input, &mut output, key, header_key, big_endian)?;

    Ok(())
}
