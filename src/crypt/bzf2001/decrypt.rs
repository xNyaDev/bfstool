use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::PathBuf;

use binrw::BinRead;

use crate::crypt::bzf2001::Key;
use crate::crypt::CryptError;
use crate::formats::bzf2001::{ArchiveHeader, FileHeader};

/// Decrypt a bzf2001 archive and write it into `output`
pub fn decrypt<R: BufRead + Seek + 'static, W: Write + Seek + 'static>(
    mut input: R,
    output: &mut BufWriter<W>,
    key: Key,
) -> Result<(), CryptError> {
    input.seek(SeekFrom::Start(0))?;
    output.seek(SeekFrom::Start(0))?;

    let mut archive_header = [0; 0xC]; // 0xC - Size of the physical representation of an ArchiveHeader
    input.read_exact(&mut archive_header)?;
    output.write_all(&archive_header)?;
    let archive_header = ArchiveHeader::read(&mut Cursor::new(archive_header))?;

    let file_headers_size = archive_header.file_count * 0x35; // 0x35 - Size of the physical representation of a FileHeader
    let mut file_headers_data = vec![0; file_headers_size as usize];
    input.read_exact(&mut file_headers_data)?;
    let mut key_position = 0;
    file_headers_data.iter_mut().for_each(|value| {
        *value ^= key[key_position];
        key_position += 1;
        if key_position == 256 {
            key_position = 0;
        }
    });
    output.write_all(&file_headers_data)?;

    let mut file_headers_data = Cursor::new(file_headers_data);
    let file_headers = (0..archive_header.file_count)
        .map(|_| FileHeader::read(&mut file_headers_data))
        .collect::<Result<Vec<FileHeader>, _>>()?;

    let key_reset_offsets = file_headers
        .into_iter()
        .map(|file_header| file_header.data_offset)
        .collect::<Vec<u32>>();
    let mut offset = input.stream_position()? as u32;
    let mut key_resets = 1;
    key_position = 0;

    let mut buffer = [0; 4096];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let bytes = buffer[..n]
                    .iter()
                    .map(|value| {
                        let new_value = value ^ key[key_position];
                        key_position += 1;
                        offset += 1;
                        if key_position == 256 {
                            key_position = 0;
                        }
                        if key_resets != key_reset_offsets.len()
                            && offset == key_reset_offsets[key_resets]
                        {
                            key_resets += 1;
                            key_position = 0;
                        }
                        new_value
                    })
                    .collect::<Vec<u8>>();
                output.write_all(&bytes)?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(CryptError::from(e)),
        };
    }

    Ok(())
}

/// Decrypt a bzf2001 archive and write it into `output`
///
/// Utility function that opens the input file, creates the output file and calls `decrypt` on those
pub fn decrypt_file(input: PathBuf, output: PathBuf, key: Key) -> Result<(), CryptError> {
    let input = File::open(input)?;
    let input = BufReader::new(input);

    let output = File::create(output)?;
    let mut output = BufWriter::new(output);

    decrypt(input, &mut output, key)?;

    Ok(())
}
