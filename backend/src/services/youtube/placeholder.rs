//! YouTube sometimes serves site-wide marketing copy in `og:description` / `<meta name="description">`
//! when a per-video snippet is not present. Detect and drop those so we prefer `shortDescription`
//! from the player response or show nothing.

/// Site-wide YouTube blurbs are short `og:description` strings (~100–200 chars).
/// Matching needles with `contains` inside a full transcript would treat real
/// captions as placeholders (e.g. "upload original content") and `ensure_transcript`
/// would discard and overwrite the cached row, including user edits.
const MIN_SITE_WIDE_PLACEHOLDER_CHARS: usize = 32;
const MAX_SITE_WIDE_PLACEHOLDER_CHARS: usize = 400;

/// True when `desc` is itself a known YouTube homepage / site-wide blurb (multiple locales).
pub fn is_site_wide_placeholder_description(desc: &str) -> bool {
    let t = desc.trim();
    let char_count = t.chars().count();
    if char_count < MIN_SITE_WIDE_PLACEHOLDER_CHARS || char_count > MAX_SITE_WIDE_PLACEHOLDER_CHARS
    {
        return false;
    }
    let lower = t.to_lowercase();
    const NEEDLES: &[&str] = &[
        "auf youtube findest du die angesagtesten videos und tracks",
        "auf youtube findest du die angesagtesten videos",
        "enjoy the videos and music you love",
        "upload original content",
        "share your videos with friends",
        "share them with friends, family, and the world",
        "the home for video",
        "find the videos and music you love",
        "discover videos from around the world",
    ];
    NEEDLES.iter().any(|n| lower.contains(n))
}

pub fn sanitize_optional_description(desc: Option<String>) -> Option<String> {
    desc.filter(|d| !is_site_wide_placeholder_description(d))
}

#[cfg(test)]
#[path = "placeholder_tests.rs"]
mod placeholder_tests;
