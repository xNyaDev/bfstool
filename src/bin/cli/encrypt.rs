use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

use bfstool::keys::Keys;

use crate::CryptFormat;

#[derive(Parser)]
pub struct Arguments {
    /// Encrypted archive file name
    input: PathBuf,
    /// Decrypted archive file name
    output: PathBuf,
    /// Keys.toml file name
    #[clap(long, default_value = "Keys.toml")]
    keys: PathBuf,
    /// Format of the encrypted file
    #[clap(short, long)]
    format: CryptFormat,
    /// Use Big Endian (console) encryption
    #[clap(long)]
    big_endian: bool,
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(arguments.keys)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let keys = toml::from_str::<Keys>(&contents)?;
    match arguments.format {
        CryptFormat::Bzf2001 => bfstool::crypt::bzf2001::encrypt_file(
            arguments.input,
            arguments.output,
            keys.bzf2001.expect("Missing encryption key").key,
        )?,
        CryptFormat::Bzf2002 => bfstool::crypt::bzf2002::decrypt_file(
            arguments.input,
            arguments.output,
            keys.bzf2002.expect("Missing encryption key").key,
        )?,
        CryptFormat::Bfs2011 => {
            let keys = keys.bfs2011.expect("Missing encryption key");
            bfstool::crypt::bfs2011::encrypt_file(
                arguments.input,
                arguments.output,
                keys.key,
                keys.header_key,
                arguments.big_endian,
            )?
        }
    }
    Ok(())
}
