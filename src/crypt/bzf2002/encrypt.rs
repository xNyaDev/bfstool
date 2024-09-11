use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::PathBuf;

use binrw::BinRead;

use crate::crypt::bzf2002::Key;
use crate::crypt::CryptError;
use crate::crypt::xxtea::xxtea_encode;
use crate::formats::bzf2002::ArchiveHeader;

/// Encrypt a bzf2002 archive and write it into `output`
pub fn encrypt<R: BufRead + Seek + 'static, W: Write + Seek + 'static>(
    mut input: R,
    output: &mut BufWriter<W>,
    key: Key,
) -> Result<(), CryptError> {
    input.seek(SeekFrom::Start(0))?;
    output.seek(SeekFrom::Start(0))?;

    let mut archive_header = [0; 0x10]; // 0x10 - Size of the physical representation of an ArchiveHeader
    input.read_exact(&mut archive_header)?;
    output.write_all(&archive_header)?;
    let archive_header = ArchiveHeader::read(&mut Cursor::new(archive_header))?;

    let file_header_size = (archive_header.header_size - 0x10 + 3) & !3; // Skip the archive header and pad to next multiple of 4 bytes

    let mut file_header_data = vec![0u8; file_header_size as usize];
    input.read_exact(&mut file_header_data)?;
    let mut file_header_data = file_header_data.chunks_exact(4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect();
    xxtea_encode(
        &mut file_header_data,
        key.chunks_exact(4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])).collect::<Vec<_>>().try_into().unwrap(),
        |_, z, sum, key_fn_out| {
            sum.wrapping_add(key_fn_out)^z.wrapping_add(z.wrapping_shl(4) ^ z.wrapping_shr(5))
        },
        |key, p, e| key[e ^ p & 3],
    );
    let file_header_data = file_header_data.into_iter().flat_map(u32::to_le_bytes).collect::<Vec<u8>>();
    output.write_all(&file_header_data)?;

    io::copy(&mut input, output)?;

    Ok(())
}

/// Encrypt a bzf2002 archive and write it into `output`
///
/// Utility function that opens the input file, creates the output file and calls `decrypt` on those
pub fn encrypt_file(input: PathBuf, output: PathBuf, key: Key) -> Result<(), CryptError> {
    let input = File::open(input)?;
    let input = BufReader::new(input);

    let output = File::create(output)?;
    let mut output = BufWriter::new(output);

    encrypt(input, &mut output, key)?;

    Ok(())
}
