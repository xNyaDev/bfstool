# bfstool

bfstool is a tool for BFS (Bugbear File System) archive creation, extraction and more. BFS archives are present in most games developed by BugBear.

For a complete list of known official BFS archives please see bfs_file_dat.md.

Support of unofficial archives is present on a best-effort basis and is not guaranteed to work.

## File format compatibility list

| Game                      | Extracting | Archiving | Notes                                                                                                          |
|:--------------------------|:----------:|:---------:|:---------------------------------------------------------------------------------------------------------------|
| Rally Trophy              |    N/A     |    N/A    | Files are not packed                                                                                           |
| Tough Trucks              |    N/A     |    N/A    | Files are not packed                                                                                           |
| FlatOut                   |     ✔      |     ?     | Running with files created by bfstool is not extensively tested                                                |
| Glimmerati                |    N/A     |    N/A    | Files are packed in standard ZIP files                                                                         |
| FlatOut 2                 |     ✔      |     ?     | Running with files created by bfstool is not extensively tested                                                |
| FlatOut: Ultimate Carnage |     ✔      |     ?     | Running with files created by bfstool is not extensively tested                                                |
| FlatOut: Head On          |     ✔      |     ?     | Running with files created by bfstool is not extensively tested                                                |
| Sega Rally Revo           |     ✔      |     ?     | Running with files created by bfstool is not extensively tested                                                |
| Ridge Racer Unbounded     |     ✔      |     ❌     | BFS files are encrypted. Encryption is not supported. No guide for extracting encryption key from the game yet |
| Ridge Racer Driftopia     |     ❌      |     ❌     | BFS files are encrypted. Encryption/decryption is not supported                                                |
| Wreckfest                 |     ❌      |     ❌     | BFS files are encrypted. Encryption/decryption is not supported                                                |

## Examples

List files in FlatOut europe.bfs:
```console
$ bfstool list --format v1 europe.bfs
Listing archive: europe.bfs
Physical size: 4531
Headers size: 4058
File count: 1
| Method | Size | Compressed | Copies |  Offset  | File Name                 |
|--------|------|------------|--------|----------|---------------------------|
|   zlib | 1103 |        471 |    0+0 | 00000fdc | data/language/version.ini |
```

Extract all files from FlatOut 2 fo2b.bfs to directory fo2b
```console
$ bfstool extract fo2b.bfs fo2b --format v2
Extracted 14 files.
```

Extract all tga files from FlatOut common1.bfs to directory common1
```console
$ bfstool extract -f v1 common1.bfs common1 "**/*.tga" -v
"data/menu/bg/longjump.tga" 1228844 bytes
"data/menu/bg-mainmenu.tga" 1281282 -> 2359340 bytes
"data/menu/bg/bowling.tga" 1228844 bytes
...
"data/menu/bg/class_a_finish.tga" 1228844 bytes
Extracted 111 files.
```

Identify an unknown file (Only works with files listed in bfs_file_dat.md)
```console
$ bfstool identify unknown.bfs
Identifying archive: unknown.bfs
File name: FLATOUT.BFS
Game: FlatOut
Platform: PlayStation 2
Format: v1
Filter: fo1
Copy filter: fo1-ps2-usa
Source:
- Redump (USA)
- Non-Redump (Beta) (2005.03.23)
CRC32: CBB29ACA
MD5: 8b7c307a1ad5483b0bee958687d6d94e
SHA1: c02f2111a97d9b24f6be2db433d25eee97bb890a
```
```console
$ bfstool id ZMenuFO2BFS.bfs
Identifying archive: ZMenuFO2BFS.bfs
File not found in the BFS file database.
Perhaps it's a modded file or not yet supported by bfstool.
```
Fast identify - the file name needs to be it's CRC32 and will be used for identification:
```console
$ bfstool id --fast-identify E2FA4AFC.bfs
Identifying archive: E2FA4AFC.bfs
File name: common1.bfs
Game: FlatOut
Platform: PC
Format: v1
Copy filter: fo1-pc
Filter: fo1
Source:
- All full PC releases
CRC32: E2FA4AFC
MD5: 95a606038261bfd36c6e48874e644c44
SHA1: 51b6671ab55665521a29b010b86d53fb8d324967
```
```console
$ bfstool id --fast-identify common1.bfs
Identifying archive: common1.bfs
File not found in the BFS file database.
Perhaps it's a modded file or not yet supported by bfstool.
Try removing --fast-identify and running again.
```

If the file is present in bfs_file_dat.md, you don't need to specify the format for subcommands reading it, like list or extract
```console
$ bfstool list europe.bfs
Identifying archive: europe.bfs
Listing archive: europe.bfs
Physical size: 4531
Headers size: 4058
File count: 1
| Method | Size | Compressed | Copies |  Offset  | File Name                 |
|--------|------|------------|--------|----------|---------------------------|
|   zlib | 1103 |        471 |    0+0 | 00000fdc | data/language/version.ini |
```

## Filtering

Pattern matching glob syntax is described [here](https://docs.rs/globset/latest/globset/#syntax) 

To use a filter pass it as `--filter-file <FILTER_FILE>` to the `archive` subcommand. There are a couple preconfigured filters to use with `--filter <FILTER>`.

Example:
```text
# Compress all files
+ **
# Do not compress any files in data/menu/bg/
- data/menu/bg/*
# Compress data/menu/bg/town2.tga and data/menu/bg/town3.tga
+ data/menu/bg/town2.tga
+ data/menu/bg/town3.tga
```
This would result in:

- `data/cars/car_1/skin1.dds` being compressed, because it matches `+ **`
- `data/menu/bg/town1.tga` not being compressed, because it matches `- data/menu/bg/*`
- `data/menu/bg/town2.tga` being compressed, because it matches `+ data/menu/bg/town2.tga`