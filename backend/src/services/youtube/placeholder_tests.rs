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
