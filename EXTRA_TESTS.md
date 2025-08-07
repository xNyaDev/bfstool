# Extra tests for bfstool

bfstool has extra tests which require files from the specific games. To run those tests use:

```console
$ cargo test --features extra_tests
```

A specific set of files will be required to tun those tests. The files needed and where can they be obtained from are 
listed below. 

To verify if you have the correct set of files, a blake3 checkfile is provided as `extra_test_data.blake3`. It can be 
checked using the [b3sum](https://crates.io/crates/b3sum) command line utility or [RapidCRC Unicode](https://www.ov2.eu/programs/rapidcrc-unicode).

The `Keys.toml` file is not listed in the checkfile and the keys will not be checked while running the tests. It is up
to the person running the tests to ensure the correct keys are provided.

## Required files for extra tests

- `Keys.toml` - Has to be filled by dumping the keys from respective games either manually or with [my mods](https://github.com/xNyaDev/game-mods).
- `bzf2001/language.bzf` - Sourced from Rally Trophy, version 1.01 EN/DE, `Data/language.bzf`
- `bzf2002/Shader.bzf` - Sourced from Bugbear Retro Demo 2002, `Data/Shader.bzf`
- `bfs2011/PC_00__ridge_racer__` - Sourced from PC version of Ridge Racer Unbounded, oldest version
- `bfs2011/PS3_00__ridge_racer__` - Sourced from PS3 version of Ridge Racer Unbounded Demo