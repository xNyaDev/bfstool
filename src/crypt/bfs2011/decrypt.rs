use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::crypt::CryptError;
use crate::crypt::bfs2011::{HeaderKey, Key};
use crate::crypt::xxtea::xxtea_decode;
use crate::formats::bfs2011::ArchiveHeader;
use binrw::BinRead;

/// Decrypt one 32 KiB block
fn decrypt_block<R: BufRead + Seek + 'static>(
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
    xxtea_decode(
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

/// Decrypt a bfs2011 archive and write it into `output`
pub fn decrypt<R: BufRead + Seek + 'static, W: Write + Seek + 'static>(
    mut input: R,
    output: &mut BufWriter<W>,
    key: Key,
    header_key: HeaderKey,
    big_endian: bool,
) -> Result<(), CryptError> {
    input.seek(SeekFrom::Start(0))?;
    output.seek(SeekFrom::Start(0))?;

    let mut data = decrypt_block(&mut input, key, big_endian)?;

    let mut archive_header = [0; 0x14];
    archive_header.copy_from_slice(&data[..0x14]);

    output.write_all(&archive_header)?;

    let archive_header = ArchiveHeader::read(&mut Cursor::new(archive_header))?;
    let headers_size = ((archive_header.header_end & 0x7FFFFFFF) as usize - 0x14 + 3) & !3; // Skip the archive header and pad to next multiple of 4 bytes

    data.drain(0..0x14);

    while data.len() < headers_size {
        let mut new_data = decrypt_block(&mut input, key, big_endian)?;
        data.append(&mut new_data);
    }

    let headers_data = data[..headers_size].to_vec();
    data.drain(0..headers_size);

    let map_fn = if big_endian {
        |x: &[u8]| u32::from_be_bytes([x[0], x[1], x[2], x[3]])
    } else {
        |x: &[u8]| u32::from_le_bytes([x[0], x[1], x[2], x[3]])
    };

    let mut headers_data = headers_data.chunks_exact(4).map(map_fn).collect::<Vec<_>>();
    xxtea_decode(
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

    output.write_all(&headers_data)?;
    output.write_all(&data)?;

    while let Ok(data) = decrypt_block(&mut input, key, big_endian) {
        output.write_all(&data)?;
    }

    Ok(())
}

/// Decrypt a bfs2011 archive and write it into `output`
///
/// Utility function that opens the input file, creates the output file and calls `decrypt` on those
pub fn decrypt_file(
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

    decrypt(input, &mut output, key, header_key, big_endian)?;

    Ok(())
}
