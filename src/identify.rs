use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::ValueEnum;
use crc::{Crc, CRC_32_ISO_HDLC};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

use crate::Format;

#[derive(Clone, Deserialize)]
pub struct FileInfo {
    pub file_name: String,
    pub game: String,
    pub platform: String,
    pub format: String,
    pub filter: String,
    pub copy_filter: String,
    pub file_version: String,
    pub source: String,
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

pub fn identify_format(bfs_name: &String, no_progress: bool, fast_identify: bool, format: Option<Format>) -> Format {
    if let Some(format) = format {
        format
    } else {
        if let Some(file_info) = identify(
            &bfs_name,
            no_progress,
            fast_identify,
        ) {
            if file_info.format == "N/A (Encrypted)" {
                println!("BFS file is encrypted, decrypt it before using.");
                println!("See README.md on details how to decrypt.");
                std::process::exit(1);
            } else {
                Format::from_str(&file_info.format, false).unwrap()
            }
        } else {
            println!("File not found in BFS file database.");
            println!("Please provide an appropriate format to use.");
            if fast_identify {
                println!("Try removing --fast-identify and running again.");
            }
            std::process::exit(1);
        }
    }
}

pub fn identify(bfs_name: &String, no_progress: bool, fast_identify: bool) -> Option<FileInfo> {
    let file_info_map = serde_json::from_str::<HashMap<String, FileInfo>>(include_str!(concat!(env!("OUT_DIR"), "/bfs_file_dat.json"))).unwrap();

    println!("Identifying archive: {}", bfs_name);

    let crc_string = if fast_identify {
        let path = PathBuf::from(bfs_name);
        path.file_stem().unwrap_or_default().to_string_lossy().to_string()
    } else {
        let file = File::open(bfs_name).expect("Failed to open BFS file");
        let mut file_reader = BufReader::new(file);

        let archive_size = fs::metadata(bfs_name).unwrap().len();

        const ISO_HDLC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = ISO_HDLC.digest();

        let mut buffer = [0; 0x10000];

        let bar = if no_progress {
            ProgressBar::hidden()
        } else {
            ProgressBar::new(archive_size)
        };
        bar.set_style(ProgressStyle::default_bar().template("[{elapsed}] {wide_bar} [{bytes}/{total_bytes}]").unwrap().progress_chars("##-"));

        loop {
            match file_reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    digest.update(&buffer[..n]);
                    bar.inc(n as u64);
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => panic!("Failed to calculate CRC with error: {}", e),
            }
        }
        bar.finish_and_clear();

        let crc = digest.finalize();
        format!("{:08X}", crc)
    };

    file_info_map.get(&crc_string).cloned()
}