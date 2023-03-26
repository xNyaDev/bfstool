/// Support for the Bfs2004a archive format
pub mod bfs2004a;

/// Available archive formats to use
pub enum Format {
    /// `bbzf` format
    ///
    /// Used by:
    /// - Rally Trophy
    Bbzf,
    /// `bzf2` v2002.01.11 format
    ///
    /// Used by:
    /// - Bugbear Retro Demo 2002,
    /// - Tough Trucks: Modified Monsters
    Bzf2,
    /// `bfs1` v2004.05.05a format
    ///
    /// Used by:
    /// - FlatOut
    Bfs2004a,
    /// `bfs1` v2004.05.05b format
    ///
    /// Used by:
    /// - FlatOut 2
    Bfs2004b,
    /// `bfs1` v2007.03.10 format
    ///
    /// Used by:
    /// - FlatOut: Ultimate Carnage
    /// - FlatOut: Head On
    /// - Sega Rally Revo
    Bfs2007,
    /// `bfs1` v2011.12.20 format
    ///
    /// Used by:
    /// - Ridge Racer Unbounded
    Bfs2011,
    /// `bbfs` v2013.03.14 format
    ///
    /// Used by:
    /// - Ridge Racer Driftopia
    /// - Next Car Game Free Technology Demo
    /// - Next Car Game Technology Sneak Peek 2.0
    Bfs2013,
}
