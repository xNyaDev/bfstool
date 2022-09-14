# bfstool

bfstool is a tool for BFS (Bugbear File System) archive creation, extraction and more. BFS archives are present in most games developed by BugBear.

For a complete list of known official BFS archives please see bfs_file_dat.md.

Support of unofficial archives is present on a best-effort basis and is not guaranteed to work.

## File format compatibility list

| Game                      | Extracting | Archiving | Notes                                                                                 |
|:--------------------------|:----------:|:---------:|:--------------------------------------------------------------------------------------|
| Rally Trophy              |    N/A     |    N/A    | Files are not packed                                                                  |
| Tough Trucks              |    N/A     |    N/A    | Files are not packed                                                                  |
| FlatOut                   |     ✔      |     ✔     |                                                                                       |
| Glimmerati                |    N/A     |    N/A    | Files are packed in standard ZIP files                                                |
| FlatOut 2                 |     ✔      |     ✔     |                                                                                       |
| FlatOut: Ultimate Carnage |     ✔      |     ✔     |                                                                                       |
| FlatOut: Head On          |     ✔      |     ✔     |                                                                                       |
| Sega Rally Revo           |     ✔      |     ✔     |                                                                                       |
| Ridge Racer Unbounded     |     ✔      |  Partial  | BFS files are encrypted. See below for decrypting. Cannot be encrypted back currently |
| Wreckfest                 |     ?      |     ?     | BFS files are encrypted                                                               |

## Examples

List files in FlatOut europe.bfs:
```console
$ bfstool list --format v1 europe.bfs
Listing archive: europe.bfs
Physical size: 4531
Headers size: 4058
File count: 1
| Method | Size | Compressed |  Offset  | File Name                 |
|--------|------|------------|----------|---------------------------|
|   zlib | 1103 |        471 | 00000fdc | data/language/version.ini |
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

If the file is present in bfs_file_dat.md, you don't need to specify the format for list, extract and dump subcommands
```console
$ bfstool list europe.bfs
Identifying archive: europe.bfs
Listing archive: europe.bfs
Physical size: 4531
Headers size: 4058
File count: 1
| Method | Size | Compressed |  Offset  | File Name                 |
|--------|------|------------|----------|---------------------------|
|   zlib | 1103 |        471 | 00000fdc | data/language/version.ini |
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