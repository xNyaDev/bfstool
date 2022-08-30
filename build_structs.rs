use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct FileInfo {
    pub file_name: String,
    pub game: String,
    pub platform: String,
    pub format: String,
    pub filter: String,
    pub source: String,
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

#[derive(Deserialize)]
pub struct Html {
    pub table: Table,
}

#[derive(Deserialize)]
pub struct Table {
    pub tbody: TBody,
}

#[derive(Deserialize)]
pub struct TBody {
    pub tr: Vec<Tr>,
}

#[derive(Deserialize)]
pub struct Tr {
    pub td: Vec<Td>,
}

#[derive(Deserialize)]
pub struct Td {
    #[serde(rename = "$value")]
    pub value: String,
}