use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use super::{
    MAX_PODCAST_FETCH_BYTES, PodcastFeedService, build_podcast_resolved_source,
    build_podcast_sync_batch, caption_payload_to_text, item_transcript_references,
    json_transcript_to_text, read_response_bytes_limited, read_response_text_limited,
    transcript_payload_to_text,
};
use crate::models::{ContentItemKind, ContentSourceKind, MediaAssetKind, ProviderKind};
use crate::services::providers::FeedSourceAdapter;

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

fn serve_once(status_line: &str, headers: &str, body: &[u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener should have addr");
    let status_line = status_line.to_string();
    let headers = headers.to_string();
    let body = body.to_vec();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let response = [
            status_line.as_bytes(),
            b"\r\n",
            headers.as_bytes(),
            b"\r\n\r\n",
            body.as_slice(),
        ]
        .concat();
        stream.write_all(&response).expect("response should write");
    });
    format!("http://{addr}/")
}

fn sample_feed_body() -> Vec<u8> {
    br#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Example Podcast</title>
            <link>https://example.com/podcast</link>
            <description>Weekly deep dives</description>
          </channel>
        </rss>"#
        .to_vec()
}

#[tokio::test]
async fn read_response_bytes_limited_rejects_content_length_over_cap() {
    let url = serve_once(
        "HTTP/1.1 200 OK",
        &format!("Content-Length: {}", MAX_PODCAST_FETCH_BYTES + 1),
        b"x",
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_bytes_limited(response, MAX_PODCAST_FETCH_BYTES)
        .await
        .expect_err("oversized content-length should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn read_response_text_limited_rejects_streamed_body_over_cap() {
    let oversized = vec![b'a'; (MAX_PODCAST_FETCH_BYTES as usize) + 8];
    let url = serve_once(
        "HTTP/1.1 200 OK",
        "Content-Type: application/rss+xml",
        &oversized,
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_text_limited(response, MAX_PODCAST_FETCH_BYTES)
        .await
        .expect_err("oversized body should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn resolve_feed_source_rejects_oversized_rss_without_buffering_full_body() {
    let oversized = vec![b'a'; (MAX_PODCAST_FETCH_BYTES as usize) + 32];
    let url = serve_once(
        "HTTP/1.1 200 OK",
        "Content-Type: application/rss+xml",
        &oversized,
    );
    let service = PodcastFeedService::new();
    let err = service
        .resolve_feed_source(&url)
        .await
        .expect_err("oversized podcast RSS should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn resolve_feed_source_accepts_small_rss() {
    let body = sample_feed_body();
    let url = serve_once(
        "HTTP/1.1 200 OK",
        &format!(
            "Content-Type: application/rss+xml\r\nContent-Length: {}",
            body.len()
        ),
        &body,
    );
    let service = PodcastFeedService::new();
    let resolved = service
        .resolve_feed_source(&url)
        .await
        .expect("small podcast RSS should resolve");
    assert_eq!(resolved.source.title, "Example Podcast");
}
