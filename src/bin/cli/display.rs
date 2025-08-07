use number_prefix::NumberPrefix;

pub fn display_offset(offset: &u64) -> String {
    format!("{offset:08x}")
}

pub fn display_size(size: &u64) -> String {
    match NumberPrefix::binary(*size as f64) {
        NumberPrefix::Standalone(bytes) => {
            format!("{bytes} B")
        }
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{n:.1} {prefix}B")
        }
    }
}
