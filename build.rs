use std::{env, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use pulldown_cmark::{html, Options, Parser};
use serde_xml_rs::from_str;

use build_structs::*;

mod build_structs;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=bfs_file_dat.md");

    if cfg!(target_os = "windows") {
        let res = winres::WindowsResource::new();
        res.compile()?;
    }

    let mut bfs_file_dat = File::open("bfs_file_dat.md")?;
    let mut bfs_file_dat_md = String::new();
    bfs_file_dat.read_to_string(&mut bfs_file_dat_md)?;

    let mut parser_options = Options::empty();
    parser_options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&bfs_file_dat_md, parser_options);

    let mut bfs_file_dat_html = String::new();
    html::push_html(&mut bfs_file_dat_html, parser);

    bfs_file_dat_html = format!("<html>{bfs_file_dat_html}</html>").replace("<br>", "\n");

    let html: Html = from_str(&bfs_file_dat_html).unwrap();

    let file_info_map = html.table.tbody.tr.into_iter().map(|tr| {
        let file_info_vec = tr.td.into_iter().map(|td| {
            td.value
        }).collect::<Vec<String>>();
        let file_info = FileInfo {
            file_name: file_info_vec.get(0).cloned().unwrap_or_default(),
            game: file_info_vec.get(1).cloned().unwrap_or_default(),
            platform: file_info_vec.get(2).cloned().unwrap_or_default(),
            format: file_info_vec.get(3).cloned().unwrap_or_default(),
            filter: file_info_vec.get(4).cloned().unwrap_or_default(),
            copy_filter: file_info_vec.get(5).cloned().unwrap_or_default(),
            source: file_info_vec.get(6).cloned().unwrap_or_default(),
            crc32: file_info_vec.get(8).cloned().unwrap_or_default(),
            md5: file_info_vec.get(9).cloned().unwrap_or_default(),
            sha1: file_info_vec.get(10).cloned().unwrap_or_default(),
        };
        (file_info.crc32.clone(), file_info)
    }).collect::<HashMap<String, FileInfo>>();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dat_path = Path::new(&out_dir).join("bfs_file_dat.json");

    let file = File::create(dat_path)?;

    serde_json::to_writer(file, &file_info_map).unwrap();

    Ok(())
}