//! UTF-8 safe string helpers.
//!
//! Byte-index slicing (`&s[..n]`) panics when `n` falls inside a multi-byte
//! character, which is common with Vietnamese text. Use these instead.

/// Returns the prefix of `s` containing at most `max_chars` characters.
pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}

/// Like [`truncate_chars`], but appends `...` when `s` was shortened.
pub fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    let prefix = truncate_chars(s, max_chars);
    if prefix.len() < s.len() {
        format!("{prefix}...")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_by_chars_not_bytes() {
        assert_eq!(truncate_chars("ệệệ", 2), "ệệ");
        assert_eq!(truncate_chars("abc", 5), "abc");
        assert_eq!(truncate_chars("", 3), "");
        assert_eq!(truncate_chars("abc", 0), "");
    }

    #[test]
    fn ellipsis_only_when_shortened() {
        assert_eq!(truncate_with_ellipsis("Tiếng Việt", 5), "Tiếng...");
        assert_eq!(truncate_with_ellipsis("Tiếng", 5), "Tiếng");
    }
}
