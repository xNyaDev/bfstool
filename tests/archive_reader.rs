use std::error::Error;
use std::path::PathBuf;

use pretty_assertions::assert_eq;

use bfstool::ArchivedFileInfo;
use bfstool::CompressionMethod;

#[test]
fn test_bfs2004a() -> Result<(), Box<dyn Error>> {
    let archive = bfstool::read_archive_file(
        &PathBuf::from("test_data/bfs2004a/europe.bin"),
        bfstool::Format::Bfs2004a,
        false,
    )?;

    assert_eq!(archive.file_count(), 1);
    assert_eq!(archive.file_names(), vec!["data/language/version.ini"]);
    assert_eq!(
        archive.file_info("data/language/version.ini"),
        vec![ArchivedFileInfo {
            offset: 0xFDC,
            compression_method: CompressionMethod::Zlib,
            size: 0x44F,
            compressed_size: 0x1D7,
            copies: 0,
            hash: Some(0xF6260C6E),
        }]
    );
    assert_eq!(archive.file_info("non_existing_file"), vec![]);

    assert_eq!(
        archive.multiple_file_info(vec![
            "data/language/version.ini".to_string(),
            "non_existing_file".to_string()
        ]),
        vec![(
            "data/language/version.ini".to_string(),
            ArchivedFileInfo {
                offset: 0xFDC,
                compression_method: CompressionMethod::Zlib,
                size: 0x44F,
                compressed_size: 0x1D7,
                copies: 0,
                hash: Some(0xF6260C6E),
            }
        )]
    );

    Ok(())
}

#[test]
fn test_bfs2004b() -> Result<(), Box<dyn Error>> {
    let archive = bfstool::read_archive_file(
        &PathBuf::from("test_data/bfs2004b/fo2a.bin"),
        bfstool::Format::Bfs2004b,
        false,
    )?;

    assert_eq!(archive.file_count(), 6349);

    let names = archive.file_names();

    assert_eq!(
        names[0],
        "data/tracks/fields/fields1/c/lighting/shadowmap_w2.dat"
    );
    assert_eq!(names[names.len() - 1], "data/cars/shared/tire_1.bgm");

    assert_eq!(
        archive.file_info("data/tracks/fields/fields1/c/lighting/shadowmap_w2.dat"),
        vec![ArchivedFileInfo {
            offset: 0x623AD335,
            compression_method: CompressionMethod::Zlib,
            size: 0x40000,
            compressed_size: 0x12664,
            copies: 0,
            hash: Some(0x487CE316),
        }]
    );

    assert_eq!(
        archive.multiple_file_info(vec![
            "data/tracks/fields/fields1/c/lighting/shadowmap_w2.dat".to_string(),
            "data/cars/shared/tire_1.bgm".to_string()
        ]),
        vec![
            (
                "data/tracks/fields/fields1/c/lighting/shadowmap_w2.dat".to_string(),
                ArchivedFileInfo {
                    offset: 0x623AD335,
                    compression_method: CompressionMethod::Zlib,
                    size: 0x40000,
                    compressed_size: 0x12664,
                    copies: 0,
                    hash: Some(0x487CE316),
                }
            ),
            (
                "data/cars/shared/tire_1.bgm".to_string(),
                ArchivedFileInfo {
                    offset: 0x2F27CCFA,
                    compression_method: CompressionMethod::Zlib,
                    size: 0x9187,
                    compressed_size: 0x2AB8,
                    copies: 0,
                    hash: Some(0xAC3BC1F0),
                }
            ),
        ]
    );

    Ok(())
}

#[test]
fn test_bfs2007() -> Result<(), Box<dyn Error>> {
    let archive = bfstool::read_archive_file(
        &PathBuf::from("test_data/bfs2007/fouc_data.bin"),
        bfstool::Format::Bfs2007,
        false,
    )?;

    assert_eq!(archive.file_count(), 9567);

    let names = archive.file_names();

    assert_eq!(names[0], "data/tracks/racing/textures/rac_lamppost4.dds");
    assert_eq!(names[names.len() - 1], "data/cars/car_36/lights.dds");

    assert_eq!(
        archive.file_info("data/tracks/racing/textures/rac_lamppost4.dds"),
        vec![ArchivedFileInfo {
            offset: 0x86B1065A,
            compression_method: CompressionMethod::Zlib,
            size: 0xAB38,
            compressed_size: 0x8749,
            copies: 0,
            hash: Some(0x22434A64),
        }]
    );

    assert_eq!(
        archive.multiple_file_info(vec![
            "data/tracks/racing/textures/rac_lamppost4.dds".to_string(),
            "data/cars/car_36/lights.dds".to_string()
        ]),
        vec![
            (
                "data/tracks/racing/textures/rac_lamppost4.dds".to_string(),
                ArchivedFileInfo {
                    offset: 0x86B1065A,
                    compression_method: CompressionMethod::Zlib,
                    size: 0xAB38,
                    compressed_size: 0x8749,
                    copies: 0,
                    hash: Some(0x22434A64),
                }
            ),
            (
                "data/cars/car_36/lights.dds".to_string(),
                ArchivedFileInfo {
                    offset: 0xCA08A800,
                    compression_method: CompressionMethod::None,
                    size: 0x155F0,
                    compressed_size: 0x155F0,
                    copies: 0,
                    hash: Some(0xFBE9D4BB),
                }
            ),
        ]
    );

    Ok(())
}
