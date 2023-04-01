use nom::number::streaming::le_u32;
use nom::sequence::tuple;
use nom::IResult;

use crate::archive_reader::NomParseable;

/// Archive Header for archive of format Bfs2004a
#[derive(Debug, Eq, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use nom::Err;
    use nom::Needed::Size;

    #[test]
    fn parsing_test() {
        use super::*;

        // Test data comes from europe.bfs, first 14h bytes
        let test_data = vec![
            0x62, 0x66, 0x73, 0x31, 0x05, 0x05, 0x04, 0x20, 0xDB, 0x0F, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0xAC, 0x0F, 0x00, 0x00,
        ];

        assert_eq!(
            ArchiveHeader::parse(&test_data),
            Ok((
                vec![0xACu8, 0x0F, 0x00, 0x00].as_slice(),
                ArchiveHeader {
                    magic: 0x31736662,
                    version: 0x20040505,
                    header_end: 0xFDB,
                    file_count: 1,
                }
            ))
        );

        assert_eq!(
            ArchiveHeader::parse(&test_data[..0x10]),
            Ok((
                vec![].as_slice(),
                ArchiveHeader {
                    magic: 0x31736662,
                    version: 0x20040505,
                    header_end: 0xFDB,
                    file_count: 1,
                }
            ))
        );

        assert_eq!(
            ArchiveHeader::parse(&test_data[..8]),
            Err(Err::Incomplete(Size(NonZeroUsize::new(4).unwrap())))
        );
    }
}
