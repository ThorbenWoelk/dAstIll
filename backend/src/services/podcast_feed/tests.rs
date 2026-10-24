use super::{
    PodcastFeedService, build_podcast_resolved_source, build_podcast_sync_batch,
    caption_payload_to_text, item_transcript_references, json_transcript_to_text,
    podcast_feed_identity, podcast_source_id_for_feed_url, transcript_payload_to_text,
};
use crate::models::{ContentItemKind, ContentSourceKind, MediaAssetKind, ProviderKind};

fn sample_feed() -> rss::Channel {
    rss::Channel::read_from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
          <channel>
            <title>Example Podcast</title>
            <link>https://example.com/podcast</link>
            <description>Weekly deep dives</description>
            <image>
              <url>https://example.com/artwork.jpg</url>
              <title>Example Podcast</title>
              <link>https://example.com/podcast</link>
            </image>
            <item>
              <title>Episode 1</title>
              <guid>episode-1</guid>
              <pubDate>Tue, 07 Jan 2025 10:00:00 GMT</pubDate>
              <description>Episode 1 show notes</description>
              <podcast:transcript url="https://example.com/transcript.vtt" type="text/vtt" />
              <enclosure url="https://example.com/audio.mp3" length="42" type="audio/mpeg" />
            </item>
          </channel>
        </rss>"#
            .as_bytes(),
    )
    .expect("rss should parse")
}

#[test]
fn resolve_source_builds_podcast_series_contract() {
    let feed = sample_feed();
    let resolved = build_podcast_resolved_source("https://example.com/feed.xml", &feed);

    assert_eq!(resolved.source.provider, ProviderKind::PodcastRss);
    assert_eq!(
        resolved.source.source_kind,
        ContentSourceKind::PodcastSeries
    );
    assert_eq!(resolved.container.source_ids.len(), 1);
    assert_eq!(
        resolved.source.thumbnail_url.as_deref(),
        Some("https://example.com/artwork.jpg")
    );
    let feed_key = podcast_feed_identity("https://example.com/feed.xml");
    assert_eq!(resolved.source.id, format!("podcast:rss:{feed_key}"));
    assert_eq!(resolved.container.id, format!("podcast:series:{feed_key}"));
    assert_eq!(resolved.source.container_id, resolved.container.id);
}

#[test]
fn podcast_feed_identity_distinguishes_slug_collisions() {
    let hyphen_path = podcast_feed_identity("https://podcasts.example.com/show-feed.xml");
    let slash_path = podcast_feed_identity("https://podcasts.example.com/show/feed.xml");
    let token_hyphen =
        podcast_feed_identity("https://feeds.example.com/show.xml?token=a-b");
    let token_underscore =
        podcast_feed_identity("https://feeds.example.com/show.xml?token=a_b");

    assert_ne!(hyphen_path, slash_path);
    assert_ne!(token_hyphen, token_underscore);
    assert!(hyphen_path.starts_with("https-podcasts-example-com-show-feed-xml:"));
    assert!(slash_path.starts_with("https-podcasts-example-com-show-feed-xml:"));
    assert_eq!(
        podcast_source_id_for_feed_url("https://podcasts.example.com/show-feed.xml"),
        format!("podcast:rss:{hyphen_path}")
    );
}

#[test]
fn podcast_feed_identity_is_stable_for_exact_url() {
    let first = podcast_feed_identity("https://feeds.simplecast.com/6HKOhNgS");
    let second = podcast_feed_identity("https://feeds.simplecast.com/6HKOhNgS");
    assert_eq!(first, second);
}

#[test]
fn sync_batch_maps_episode_show_notes_and_audio() {
    let feed = sample_feed();
    let resolved = build_podcast_resolved_source("https://example.com/feed.xml", &feed);
    let batch = build_podcast_sync_batch(&resolved.source, &feed);

    assert_eq!(batch.items.len(), 1);
    assert_eq!(batch.items[0].item_kind, ContentItemKind::PodcastEpisode);
    assert_eq!(batch.parts.len(), 2);
    assert_eq!(
        batch.parts[0].part_kind,
        crate::models::ContentPartKind::ShowNotes
    );
    assert_eq!(
        batch.parts[1].part_kind,
        crate::models::ContentPartKind::Transcript
    );
    assert_eq!(batch.media_assets.len(), 1);
    assert_eq!(
        batch.media_assets[0].asset_kind,
        MediaAssetKind::SourceAudio
    );
    assert_eq!(
        batch.media_assets[0].url.as_deref(),
        Some("https://example.com/audio.mp3")
    );
    assert_eq!(
        batch.media_assets[0].mime_type.as_deref(),
        Some("audio/mpeg")
    );
}

#[test]
fn transcript_references_are_read_from_podcast_namespace() {
    let feed = sample_feed();
    let references = item_transcript_references(&feed, &feed.items()[0]);

    assert_eq!(references.len(), 1);
    assert_eq!(references[0].url, "https://example.com/transcript.vtt");
    assert_eq!(references[0].mime_type, "text/vtt");
}

#[test]
fn caption_payload_strips_timestamps_and_cue_metadata() {
    let text = caption_payload_to_text(
        "WEBVTT\n\ncue-1\n00:00:00.000 --> 00:00:02.000\n<v Alex>Hello there</v>\n\n2\n00:00:02,000 --> 00:00:04,000\nSecond line\n",
    );

    assert_eq!(text, "Hello there\nSecond line");
}

#[test]
fn json_payload_groups_segments_by_speaker() {
    let text = json_transcript_to_text(
        r#"{
            "version": "1.0.0",
            "segments": [
                { "speaker": "Alex", "startTime": 0.0, "body": "Hello" },
                { "speaker": "Alex", "startTime": 0.5, "body": "world." },
                { "speaker": "Sam", "startTime": 1.0, "body": "Reply." }
            ]
        }"#,
    );

    assert_eq!(text.as_deref(), Some("Alex: Hello world.\nSam: Reply."));
}

#[test]
fn transcript_payload_uses_actual_transcript_formats_not_description() {
    let text = transcript_payload_to_text(
        "<html><body><p>Speaker: Real transcript line.</p></body></html>",
        "text/html",
        "https://example.com/transcript.html",
    );

    assert_eq!(text.as_deref(), Some("Speaker: Real transcript line."));
}

#[test]
fn service_is_constructible() {
    let _service = PodcastFeedService::new();
}
