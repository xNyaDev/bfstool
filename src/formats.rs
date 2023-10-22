/// Support for the Bfs2004a archive format
pub mod bfs2004a;
/// Support for the Bfs2004b archive format
pub mod bfs2004b;
/// Support for the Bfs2007 archive format
pub mod bfs2007;
/// Support for the Bzf2001 archive format
pub mod bzf2001;

/// Available archive formats to use
pub enum Format {
    /// `bbzf` v2001.06.06 format
    ///
    /// Used by:
    /// - Rally Trophy
    Bzf2001,
    /// `bzf2` v2002.01.11 format
    ///
    /// Used by:
    /// - Bugbear Retro Demo 2002,
    /// - Tough Trucks: Modified Monsters
    Bzf2002,
    /// `bfs1` v2004.05.05a format
    ///
    /// Used by:
    /// - FlatOut
    Bfs2004a,
    /// `bfs1` v2004.05.05b format
    ///
    /// Used by:
    /// - FlatOut 2
    /// - FlatOut: Head On
    Bfs2004b,
    /// `bfs1` v2007.03.10 format
    ///
    /// Used by:
    /// - FlatOut: Ultimate Carnage
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
