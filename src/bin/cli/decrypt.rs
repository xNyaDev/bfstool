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
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(arguments.keys)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let keys = toml::from_str::<Keys>(&contents)?;
    match arguments.format {
        CryptFormat::Bzf2001 => bfstool::crypt::bzf2001::decrypt_file(
            arguments.input,
            arguments.output,
            keys.bzf2001.expect("Missing decryption key").key,
        )?,
    }
    Ok(())
}
