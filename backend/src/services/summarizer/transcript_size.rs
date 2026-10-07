//! Size estimates for transcripts: word counts for prompt guidance, the summary request
//! timeout, and the context window requested from local models.

use std::time::Duration;

use crate::services::ollama::CLOUD_PROMPT_TIMEOUT_SECS;

/// Han (Chinese) and Japanese Kana characters average about two characters per word.
const HAN_KANA_CHARS_PER_WORD: usize = 2;
/// Thai, Lao, Khmer, and Burmese average about four characters per word.
const SOUTHEAST_ASIAN_CHARS_PER_WORD: usize = 4;

/// Transcripts up to this length use the base summary timeout.
const SUMMARY_TIMEOUT_BASE_WORDS: usize = 5_000;
/// Extra seconds the summary request gets per 1,000 words above the base length.
const SUMMARY_TIMEOUT_SECS_PER_1000_WORDS: usize = 20;
pub(crate) const SUMMARY_MAX_TIMEOUT_SECS: u64 = 900;

/// Room left in the local context window for the summary the model writes.
const SUMMARY_OUTPUT_TOKEN_BUDGET: usize = 4_096;
pub(crate) const LOCAL_CONTEXT_MIN_TOKENS: u32 = 8_192;
pub(crate) const LOCAL_CONTEXT_MAX_TOKENS: u32 = 65_536;
const LOCAL_CONTEXT_STEP_TOKENS: usize = 1_024;
/// Rough token density for scripts that separate words with spaces.
const SPACED_SCRIPT_CHARS_PER_TOKEN: usize = 3;

/// True for Chinese characters and Japanese Hiragana or Katakana.
pub(crate) fn is_han_or_kana(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3040..=0x30FF // Hiragana and Katakana
            | 0x31F0..=0x31FF // Katakana phonetic extensions
            | 0x3400..=0x4DBF // CJK extension A
            | 0x4E00..=0x9FFF // CJK unified ideographs
            | 0xF900..=0xFAFF // CJK compatibility ideographs
            | 0xFF66..=0xFF9F // Half-width Katakana
            | 0x20000..=0x323AF // CJK extensions B to H
    )
}

/// True for scripts that are written without spaces between words, besides Han and Kana.
fn is_southeast_asian_unspaced(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0E00..=0x0EFF // Thai and Lao
            | 0x1000..=0x109F // Myanmar
            | 0x1780..=0x17FF // Khmer
    )
}

pub(crate) fn is_unspaced_script(ch: char) -> bool {
    is_han_or_kana(ch) || is_southeast_asian_unspaced(ch)
}

/// Estimate the number of words in a transcript.
///
/// Text with spaces counts one word per space-separated token. Scripts without spaces
/// between words are estimated from their character count, so a Chinese or Thai
/// transcript is not reported as a few dozen "words".
pub(crate) fn count_transcript_words(text: &str) -> usize {
    text.split_whitespace().map(estimate_token_words).sum()
}

fn estimate_token_words(token: &str) -> usize {
    let mut han_kana = 0usize;
    let mut southeast_asian = 0usize;
    let mut has_spaced_text = false;
    for ch in token.chars() {
        if is_han_or_kana(ch) {
            han_kana += 1;
        } else if is_southeast_asian_unspaced(ch) {
            southeast_asian += 1;
        } else {
            has_spaced_text = true;
        }
    }
    usize::from(has_spaced_text)
        + han_kana.div_ceil(HAN_KANA_CHARS_PER_WORD)
        + southeast_asian.div_ceil(SOUTHEAST_ASIAN_CHARS_PER_WORD)
}

/// Request timeout for one summary call. Long transcripts get more time, up to a cap,
/// so multi-hour videos are not cut off by the default prompt timeout.
pub(crate) fn summary_timeout(word_count: usize) -> Duration {
    let extra_words = word_count.saturating_sub(SUMMARY_TIMEOUT_BASE_WORDS);
    let extra_secs = (extra_words * SUMMARY_TIMEOUT_SECS_PER_1000_WORDS).div_ceil(1_000) as u64;
    Duration::from_secs((CLOUD_PROMPT_TIMEOUT_SECS + extra_secs).min(SUMMARY_MAX_TIMEOUT_SECS))
}

/// Context window (`num_ctx`) to request from a local model so the whole prompt fits.
///
/// Ollama's default local context is small and silently drops the start of long prompts.
/// The estimate counts about one token per three characters for spaced scripts and one
/// token per character for scripts without spaces, plus room for the summary itself.
pub(crate) fn local_context_tokens(preamble: &str, prompt: &str) -> u32 {
    let (spaced_chars, unspaced_chars) =
        preamble
            .chars()
            .chain(prompt.chars())
            .fold((0usize, 0usize), |(spaced, unspaced), ch| {
                if is_unspaced_script(ch) {
                    (spaced, unspaced + 1)
                } else {
                    (spaced + 1, unspaced)
                }
            });
    let estimate = spaced_chars.div_ceil(SPACED_SCRIPT_CHARS_PER_TOKEN)
        + unspaced_chars
        + SUMMARY_OUTPUT_TOKEN_BUDGET;
    let rounded = estimate.div_ceil(LOCAL_CONTEXT_STEP_TOKENS) * LOCAL_CONTEXT_STEP_TOKENS;
    u32::try_from(rounded)
        .unwrap_or(u32::MAX)
        .clamp(LOCAL_CONTEXT_MIN_TOKENS, LOCAL_CONTEXT_MAX_TOKENS)
}
