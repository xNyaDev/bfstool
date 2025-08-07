use serde::{Deserialize, Serialize};

/// The Keys.toml document holding all encryption keys
#[derive(Deserialize, Serialize)]
pub struct Keys {
    /// Keys for the Bzf2001 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bzf2001: Option<Bzf2001Keys>,
    /// Keys for the Bzf2002 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bzf2002: Option<Bzf2002Keys>,
    /// Keys for the Bfs2011 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bfs2011: Option<Bfs2011Keys>,
}

/// Keys for the Bzf2001 format
#[derive(Deserialize, Serialize)]
pub struct Bzf2001Keys {
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    /// Encryption key for Bzf2001
    pub key: crate::crypt::bzf2001::Key,
}

/// Keys for the Bzf2002 format
#[derive(Deserialize, Serialize)]
pub struct Bzf2002Keys {
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    /// Encryption key for Bzf2002
    pub key: crate::crypt::bzf2002::Key,
}

/// Keys for the Bfs2011 format
#[derive(Deserialize, Serialize)]
pub struct Bfs2011Keys {
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    /// Encryption key for Bfs2011 data
    pub key: crate::crypt::bfs2011::Key,
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    /// Encryption key for Bfs2011 header
    pub header_key: crate::crypt::bfs2011::HeaderKey,
}
