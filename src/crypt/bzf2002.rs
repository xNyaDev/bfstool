use crate::crypt::xxtea;
pub use decrypt::{decrypt, decrypt_file};
pub use encrypt::{encrypt, encrypt_file};

mod decrypt;
mod encrypt;

/// Key used in bzf2002 encryption
pub type Key = [u8; 16];

/// MX function for XXTEA
const MXFN: xxtea::MxFn = |_, z, sum, key_fn_out| {
    sum.wrapping_add(key_fn_out) ^ z.wrapping_add(z.wrapping_shl(4) ^ z.wrapping_shr(5))
};

/// KEY function for XXTEA
const KEYFN: xxtea::KeyFn = |key, p, e| key[e ^ p & 3];
