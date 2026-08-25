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

#[test]
fn keeps_long_transcript_that_mentions_youtube_marketing_phrases() {
    let s = format!(
        "{} Upload original content if you want to grow, share your videos with friends, \
         and treat YouTube as the home for video. {}",
        "This tutorial walks through captions, thumbnails, and scheduling. ".repeat(8),
        "Then we answer questions from chat about copyright claims and end screens."
    );
    assert!(s.chars().count() > 400);
    assert!(!is_site_wide_placeholder_description(&s));
}
