use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

use crate::archive_reader::NomParseable;

/// Archive Header for archive of format Bfs2004a
pub struct ArchiveHeader {
    /// File identification magic
    ///
    /// `62 66 73 31`, `"bfs1"`
    pub magic: u32,
    /// File version
    ///
    /// `05 05 04 20`, v2004.05.05a
    pub version: u32,
    /// Offset at which the header section ends
    pub header_end: u32,
    /// Number of files in the archive
    pub file_count: u32,
}

impl NomParseable for ArchiveHeader {
    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (magic, version, header_end, file_count)) =
            tuple((le_u32, le_u32, le_u32, le_u32))(input)?;
        Ok((
            input,
            Self {
                magic,
                version,
                header_end,
                file_count,
            },
        ))
    }
}
