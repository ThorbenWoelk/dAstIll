use super::{
    compatibility_channel_from_source, normalized_text_for_match, transcript_matches_show_notes,
};
use crate::models::{
    ContentSource, ContentSourceKind, ProviderKind, SourceBackingKind, SubscriptionContainerKind,
    Transcript, TranscriptRenderMode,
};

fn transcript(raw_text: Option<&str>, formatted_markdown: Option<&str>) -> Transcript {
    Transcript {
        video_id: "episode-1".to_string(),
        raw_text: raw_text.map(ToOwned::to_owned),
        formatted_markdown: formatted_markdown.map(ToOwned::to_owned),
        render_mode: TranscriptRenderMode::PlainText,
        timed_text: None,
    }
}

#[test]
fn show_notes_match_ignores_whitespace_only() {
    let transcript = transcript(Some("Episode 1\n\nshow notes"), None);

    assert!(transcript_matches_show_notes(
        &transcript,
        "Episode 1 show notes"
    ));
}

#[test]
fn real_transcript_does_not_match_show_notes() {
    let transcript = transcript(Some("Host: Welcome to the show. Guest: Thanks."), None);

    assert!(!transcript_matches_show_notes(
        &transcript,
        "Episode description with links."
    ));
}

#[test]
fn normalized_text_collapses_whitespace() {
    assert_eq!(normalized_text_for_match("a\n b\t c"), "a b c");
}

#[test]
fn compatibility_channel_gets_default_sync_floor() {
    let source = ContentSource {
        id: "podcast:series:show".to_string(),
        provider: ProviderKind::PodcastRss,
        source_kind: ContentSourceKind::PodcastSeries,
        container_id: "podcast:series:show".to_string(),
        container_kind: SubscriptionContainerKind::Series,
        backing_kind: SourceBackingKind::Feed,
        title: "Show".to_string(),
        subtitle: None,
        handle: None,
        thumbnail_url: None,
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: false,
        external_ids: Vec::new(),
    };

    let channel = compatibility_channel_from_source(&source);

    assert!(channel.earliest_sync_date.is_some());
    assert!(!channel.earliest_sync_date_user_set);
}
