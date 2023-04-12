use number_prefix::NumberPrefix;

pub fn display_offset(offset: &u64) -> String {
    format!("{:08x}", offset)
}

pub fn display_size(size: &u64) -> String {
    match NumberPrefix::binary(*size as f64) {
        NumberPrefix::Standalone(bytes) => {
            format!("{} B", bytes)
        }
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{:.1} {}B", n, prefix)
        }
    }
}
