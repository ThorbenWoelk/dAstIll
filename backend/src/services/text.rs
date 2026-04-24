/// Truncate `text` to at most `max_chars` Unicode scalar values.
pub fn limit_text(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

pub fn strip_emoji(text: &str) -> String {
    text.chars().filter(|ch| !is_emoji_scalar(*ch)).collect()
}

fn is_emoji_scalar(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F000..=0x1FAFF
            | 0x2600..=0x26FF
            | 0x2700..=0x27BF
            | 0xFE00..=0xFE0F
            | 0x200D
            | 0x20E3
    )
}

#[cfg(test)]
mod tests {
    use super::{limit_text, strip_emoji};

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

    #[test]
    fn strip_emoji_removes_common_emoji_sequences() {
        let input = "Done ✅ Great 😀 #️⃣ text";

        assert_eq!(strip_emoji(input), "Done  Great  # text");
    }

    #[test]
    fn strip_emoji_keeps_markdown_and_citations() {
        let input = "## Result\nThe claim is supported.[1] -> check.";

        assert_eq!(strip_emoji(input), input);
    }
}
