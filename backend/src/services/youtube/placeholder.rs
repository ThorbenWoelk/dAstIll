//! YouTube sometimes serves site-wide marketing copy in `og:description` / `<meta name="description">`
//! when a per-video snippet is not present. Detect and drop those so we prefer `shortDescription`
//! from the player response or show nothing.

/// True when `desc` matches known YouTube homepage / site-wide blurbs (multiple locales).
pub fn is_site_wide_placeholder_description(desc: &str) -> bool {
    let t = desc.trim();
    if t.len() < 32 {
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
mod tests {
    use super::*;

    #[test]
    fn detects_german_youtube_home_blurb() {
        let s = "Auf YouTube findest du die angesagtesten Videos und Tracks. Außerdem kannst du eigene Inhalte hochladen und mit Freunden oder gleich der ganzen Welt teilen.";
        assert!(is_site_wide_placeholder_description(s));
    }

    #[test]
    fn keeps_typical_creator_description() {
        let s = "In this episode we walk through the migration and answer questions from chat.";
        assert!(!is_site_wide_placeholder_description(s));
    }
}
