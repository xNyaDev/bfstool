/// TEA DELTA value
const DELTA: u32 = 0x9E3779B9;

/// The `k[p&3^e]` function of the reference implementation
pub type KeyFn = fn(key: [u32; 4], p: usize, e: usize) -> u32;
/// The `MX` define of the reference implementation, this gets changed by Bugbear
pub type MxFn = fn(y: u32, z: u32, sum: u32, key_fn_out: u32) -> u32;

/// Encoding part of XXTEA, based on the reference C implementation
pub fn xxtea_encode(data: &mut [u32], key: [u32; 4], mx: MxFn, key_fn: KeyFn) {
    if data.len() <= 1 {
        return;
    }
    let rounds = 6 + 52 / data.len();
    for round in 1..=rounds {
        let sum = (round as u32).wrapping_mul(DELTA);
        let e = sum.wrapping_shr(2) as usize & 3;
        for p in 0..data.len() {
            let y = if p + 1 == data.len() {
                data[0]
            } else {
                data[p + 1]
            };
            let z = if p == 0 {
                data[data.len() - 1]
            } else {
                data[p - 1]
            };
            data[p] = data[p].wrapping_add(mx(y, z, sum, key_fn(key, p, e)));
        }
    }
}

/// Decoding part of XXTEA, based on the reference C implementation
pub fn xxtea_decode(data: &mut [u32], key: [u32; 4], mx: MxFn, key_fn: KeyFn) {
    if data.len() <= 1 {
        return;
    }
    let rounds = 6 + 52 / data.len();
    for round in (1..=rounds).rev() {
        let sum = (round as u32).wrapping_mul(DELTA);
        let e = sum.wrapping_shr(2) as usize & 3;
        for p in (0..data.len()).rev() {
            let y = if p + 1 == data.len() {
                data[0]
            } else {
                data[p + 1]
            };
            let z = if p == 0 {
                data[data.len() - 1]
            } else {
                data[p - 1]
            };
            data[p] = data[p].wrapping_sub(mx(y, z, sum, key_fn(key, p, e)));
        }
    }
}
