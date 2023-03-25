`bfstool` is a library to interact with BZF and BFS archives

BZF and BFS archives are present in all games and tech demos developed by Bugbear Entertainment
except Glimmerati (Nokia N-Gage).

This library guarantees support for official files and provides support for custom files
on a best-effort basis. Sometimes specific behaviour is required to support unofficial files,
in which case all the required changes are documented.

# Example apps

3 example apps using bfstool will be provided:

- [ ] `bfstool-cli` - Command-line application to interact with BFS archives providing advanced
  functionality, perfect for various automations as well as power users
- [ ] `bfstool-tui` - Command-line application with a terminal user interface providing most
  options a regular user requires
- [ ] `bfstool-gui` - [egui](https://www.egui.rs/)-based application providing the same
  functionality as `bfstool-tui`

# Supported formats

- [ ] BZF
    - [ ] `bbzf` (Rally Trophy)
    - [ ] `bzf2` v2002.01.11 (Bugbear Retro Demo 2002, Tough Trucks: Modified Monsters)
- [ ] BFS
    - [ ] `bfs1` v2004.05.05a (FlatOut)
    - [ ] `bfs1` v2004.05.05b (FlatOut 2)
    - [ ] `bfs1` v2007.03.10 (FlatOut: Ultimate Carnage, FlatOut: Head On, Sega Rally Revo)
    - [ ] `bfs1` v2011.12.20 (Ridge Racer Unbounded)
    - [ ] `bbfs` v2013.03.14 (Ridge Racer Driftopia, Next Car Game Free Technology Demo, Next Car
      Game Technology Sneak Peek 2.0)