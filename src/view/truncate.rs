//! Unicode-width-aware string truncation.
//!
//! Authored by okooo5km(十里).

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Visible (terminal cell) width of `s`, treating CJK / wide characters as 2.
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Take characters from the front of `s` until adding the next one would
/// exceed `max_cols` cells. Returns the prefix.
pub fn take_columns_front(s: &str, max_cols: usize) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > max_cols {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

/// Take characters from the back of `s` until prepending the next one would
/// exceed `max_cols` cells.
pub fn take_columns_back(s: &str, max_cols: usize) -> String {
    let mut buf: Vec<char> = Vec::new();
    let mut used = 0usize;
    for ch in s.chars().rev() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > max_cols {
            break;
        }
        buf.push(ch);
        used += w;
    }
    buf.reverse();
    buf.into_iter().collect()
}

/// Truncate `s` to fit in `max_cols` terminal cells, preserving head and tail
/// with a single `…` in the middle.
///
/// Returns `s` unchanged if it already fits. Always returns at most `max_cols`
/// cells. When `max_cols < 1` returns an empty string.
pub fn truncate_middle(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if display_width(s) <= max_cols {
        return s.to_string();
    }
    if max_cols == 1 {
        return "…".into();
    }
    let budget = max_cols - 1;
    let head = budget * 6 / 10;
    let tail = budget - head;
    let head_str = take_columns_front(s, head);
    let tail_str = take_columns_back(s, tail);
    format!("{head_str}…{tail_str}")
}

/// Take the trailing N *characters* (not cells). Used for things like
/// fingerprint tails where N is conventional.
pub fn tail_chars(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        chars[chars.len() - n..].iter().collect()
    }
}

/// Format a byte count in a human-friendly form (`1.2 MB`).
#[allow(clippy::cast_precision_loss)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_passes_through() {
        assert_eq!(truncate_middle("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_long() {
        let out = truncate_middle("4FB8AC11-3D02-E74E-A105-D9AF11", 12);
        assert!(out.contains('…'));
        assert!(display_width(&out) <= 12);
    }

    #[test]
    fn truncate_cjk_respects_width() {
        let s = "Zipic 永久授权码长名字测试用例";
        let out = truncate_middle(s, 10);
        assert!(display_width(&out) <= 10);
    }

    #[test]
    fn bytes_smoke() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    }
}
