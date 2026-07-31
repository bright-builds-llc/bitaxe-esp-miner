pub(super) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub(super) fn count_invalid_utf8_bytes(bytes: &[u8]) -> u64 {
    let mut remainder = bytes;
    let mut count = 0_u64;
    while let Err(error) = std::str::from_utf8(remainder) {
        let invalid_start = error.valid_up_to();
        let invalid_length = error
            .error_len()
            .unwrap_or_else(|| remainder.len().saturating_sub(invalid_start).max(1));
        count = count.saturating_add(u64::try_from(invalid_length).unwrap_or(u64::MAX));
        remainder = &remainder[invalid_start.saturating_add(invalid_length)..];
    }
    count
}
