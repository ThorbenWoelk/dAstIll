use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use crate::models::VideoInfo;
use crate::services::youtube::YouTubeError;
use crate::services::youtube::YouTubeService;

use super::{
    read_response_text_limited, video_info_missing_channel_identity, youtube_page_url_is_allowed,
    MAX_YOUTUBE_PAGE_BYTES,
};

fn build_video_info(channel_id: Option<&str>) -> VideoInfo {
    VideoInfo {
        video_id: "video-123".to_string(),
        watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
        title: "Video".to_string(),
        description: None,
        thumbnail_url: None,
        channel_name: None,
        channel_id: channel_id.map(str::to_string),
        published_at: None,
        duration_iso8601: None,
        duration_seconds: None,
        view_count: None,
    }
}

#[test]
fn missing_channel_identity_detects_absent_or_blank_channel_ids() {
    assert!(video_info_missing_channel_identity(&build_video_info(None)));
    assert!(video_info_missing_channel_identity(&build_video_info(
        Some("   ")
    )));
    assert!(!video_info_missing_channel_identity(&build_video_info(
        Some("UC1234567890123456789012")
    )));
}

#[test]
fn youtube_page_url_allowlist_rejects_lookalike_hosts() {
    assert!(youtube_page_url_is_allowed(
        "https://www.youtube.com/@veritasium"
    ));
    assert!(youtube_page_url_is_allowed(
        "https://m.youtube.com/channel/UC1234567890123456789012"
    ));
    assert!(youtube_page_url_is_allowed("https://youtu.be/dQw4w9WgXcQ"));
    assert!(youtube_page_url_is_allowed(
        "https://music.youtube.com/channel/UC1234567890123456789012"
    ));

    assert!(
        !youtube_page_url_is_allowed("https://youtube.com.evil.example/huge"),
        "suffix lookalike host must not be fetched"
    );
    assert!(
        !youtube_page_url_is_allowed("https://evil.example/?q=youtube.com"),
        "query-string lookalike must not be fetched"
    );
    assert!(
        !youtube_page_url_is_allowed("http://127.0.0.1/?youtube.com"),
        "loopback SSRF lookalike must not be fetched"
    );
    assert!(
        !youtube_page_url_is_allowed("http://169.254.169.254/?youtube.com"),
        "metadata SSRF lookalike must not be fetched"
    );
    assert!(!youtube_page_url_is_allowed("https://notyoutube.com/"));
    assert!(!youtube_page_url_is_allowed("ftp://www.youtube.com/@x"));
}

#[tokio::test]
async fn resolve_channel_rejects_lookalike_hosts_without_fetching() {
    let service = YouTubeService::new();
    let err = service
        .resolve_channel("https://youtube.com.evil.example/huge")
        .await
        .expect_err("lookalike YouTube host should be rejected");
    assert!(
        matches!(err, YouTubeError::NotYouTubeUrl),
        "unexpected error: {err}"
    );

    let err = service
        .resolve_channel("https://evil.example/?ref=youtube.com")
        .await
        .expect_err("query-string lookalike should be rejected");
    assert!(
        matches!(err, YouTubeError::NotYouTubeUrl),
        "unexpected error: {err}"
    );
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

#[tokio::test]
async fn read_response_text_limited_rejects_content_length_over_cap() {
    let url = serve_once(
        "HTTP/1.1 200 OK",
        &format!("Content-Length: {}", MAX_YOUTUBE_PAGE_BYTES + 1),
        b"x",
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_text_limited(response, MAX_YOUTUBE_PAGE_BYTES)
        .await
        .expect_err("oversized content-length should fail");
    assert!(
        matches!(err, YouTubeError::PageTooLarge),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn read_response_text_limited_rejects_streamed_body_over_cap() {
    let oversized = vec![b'a'; (MAX_YOUTUBE_PAGE_BYTES as usize) + 8];
    let url = serve_once("HTTP/1.1 200 OK", "Content-Type: text/html", &oversized);
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_text_limited(response, MAX_YOUTUBE_PAGE_BYTES)
        .await
        .expect_err("oversized body should fail");
    assert!(
        matches!(err, YouTubeError::PageTooLarge),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn read_response_text_limited_accepts_small_body() {
    let url = serve_once(
        "HTTP/1.1 200 OK",
        "Content-Type: text/html\r\nContent-Length: 5",
        b"hello",
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let body = read_response_text_limited(response, MAX_YOUTUBE_PAGE_BYTES)
        .await
        .expect("small body should be accepted");
    assert_eq!(body, "hello");
}
