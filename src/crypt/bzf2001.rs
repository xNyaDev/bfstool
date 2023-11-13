pub use decrypt::{decrypt, decrypt_file};
pub use encrypt::{encrypt, encrypt_file};

mod decrypt;
mod encrypt;

/// Key used in bzf2001 encryption
pub type Key = [u8; 256];

