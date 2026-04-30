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
#[path = "text_tests.rs"]
mod text_tests;
