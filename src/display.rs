/// Returns a byte slice as a hex value (uppercase) with spaces between the individual bytes
pub fn spaced_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns Some(String) if passed buffer has a valid ascii string, otherwise returns None
///
/// The buffer can be zero-terminated, but does not need to be. No control characters are allowed.
pub fn ascii_value(bytes: &[u8]) -> Option<String> {
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    for byte in bytes {
        if !byte.is_ascii() || byte.is_ascii_control() {
            return None;
        }
    }
    String::from_utf8(bytes.to_vec()).ok()
}
