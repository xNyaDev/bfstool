use serde::{Deserialize, Serialize};

/// The Keys.toml document holding all encryption keys
#[derive(Deserialize, Serialize)]
pub struct Keys {
    /// Keys for the Bzf2001 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bzf2001: Option<Bzf2001Keys>,
}

/// Keys for the Bzf2001 format
#[derive(Deserialize, Serialize)]
pub struct Bzf2001Keys {
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    /// Decryption key for Bzf2001
    pub key: [u8; 256],
}
