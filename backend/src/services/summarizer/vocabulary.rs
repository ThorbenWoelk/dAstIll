//! User vocabulary rules (for example `Open A I` -> `OpenAI`) applied to transcripts.

use crate::models::VocabularyReplacement;

use super::transcript_size::is_unspaced_script;

fn normalize_vocabulary_entry(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Apply the user's vocabulary rules to a transcript, in order.
///
/// Rules match whole words only, so `chat` -> `ChatGPT` leaves "chatting" alone. A match
/// that already sits inside the canonical form is skipped, so `Claude` -> `Claude Code`
/// leaves "Claude Code" unchanged and running the rules twice gives the same text.
///
/// The summarizer sees this text. Give the summary evaluator the same text so both judge
/// the same wording.
pub(crate) fn normalize_transcript_vocabulary(
    transcript: &str,
    replacements: &[VocabularyReplacement],
) -> String {
    let mut normalized = transcript.to_string();

    for replacement in replacements {
        let Some(from) = normalize_vocabulary_entry(&replacement.from) else {
            continue;
        };
        let Some(to) = normalize_vocabulary_entry(&replacement.to) else {
            continue;
        };
        if from == to {
            continue;
        }
        normalized = replace_whole_phrase(&normalized, from, to);
    }

    normalized
}

fn replace_whole_phrase(text: &str, from: &str, to: &str) -> String {
    let offsets_in_canonical: Vec<usize> = to.match_indices(from).map(|(at, _)| at).collect();
    let mut output = String::with_capacity(text.len());
    let mut copied_up_to = 0usize;

    for (start, _) in text.match_indices(from) {
        let end = start + from.len();
        if !starts_and_ends_on_word_boundary(text, start, end, from) {
            continue;
        }
        let already_canonical = offsets_in_canonical.iter().any(|&offset| {
            start
                .checked_sub(offset)
                .and_then(|canonical_start| text.get(canonical_start..))
                .is_some_and(|rest| rest.starts_with(to))
        });
        if already_canonical {
            continue;
        }
        output.push_str(&text[copied_up_to..start]);
        output.push_str(to);
        copied_up_to = end;
    }

    output.push_str(&text[copied_up_to..]);
    output
}

fn is_word_char(ch: char) -> bool {
    (ch.is_alphanumeric() || ch == '_') && !is_unspaced_script(ch)
}

/// A match is a whole word when the text around it does not continue the word.
/// Edges of the phrase that are punctuation or unspaced script need no boundary.
fn starts_and_ends_on_word_boundary(text: &str, start: usize, end: usize, phrase: &str) -> bool {
    let phrase_starts_with_word = phrase.chars().next().is_some_and(is_word_char);
    let phrase_ends_with_word = phrase.chars().next_back().is_some_and(is_word_char);
    let before_is_word = text[..start].chars().next_back().is_some_and(is_word_char);
    let after_is_word = text[end..].chars().next().is_some_and(is_word_char);
    !(phrase_starts_with_word && before_is_word) && !(phrase_ends_with_word && after_is_word)
}
