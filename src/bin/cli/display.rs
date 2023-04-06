use number_prefix::NumberPrefix;

pub fn display_offset(offset: &usize) -> String {
    format!("{:08x}", offset)
}

pub fn display_size(size: &usize) -> String {
    match NumberPrefix::binary(*size as f64) {
        NumberPrefix::Standalone(bytes) => {
            format!("{} B", bytes)
        }
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{:.1} {}B", n, prefix)
        }
    }
}
