/// Truncate `text` to at most `max_chars` Unicode scalar values.
pub fn limit_text(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::limit_text;

    #[test]
    fn limit_text_truncates_at_char_boundary() {
        assert_eq!(limit_text("hello world", 5), "hello");
    }

    #[test]
    fn limit_text_passes_short_input_unchanged() {
        assert_eq!(limit_text("hi", 100), "hi");
    }

    #[test]
    fn limit_text_handles_empty_input() {
        assert_eq!(limit_text("", 10), "");
    }

    #[test]
    fn limit_text_counts_unicode_scalars_not_bytes() {
        // Each emoji is 1 char
        let input = "ab\u{1F600}cd";
        assert_eq!(limit_text(input, 3), "ab\u{1F600}");
    }
}
