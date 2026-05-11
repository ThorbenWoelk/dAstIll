use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::thread;

use super::{
    TranscriptService, extract_json3_caption_url_from_ytdlp_metadata, is_public_ip,
    is_retryable_asr_status,
};
use tempfile::tempdir;

#[test]
fn local_asr_url_policy_rejects_private_and_metadata_ips() {
    assert!(!is_public_ip("127.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("10.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("172.16.0.1".parse().unwrap()));
    assert!(!is_public_ip("192.168.0.1".parse().unwrap()));
    assert!(!is_public_ip("192.88.99.1".parse().unwrap()));
    assert!(!is_public_ip("100.64.0.1".parse().unwrap()));
    assert!(!is_public_ip("169.254.169.254".parse().unwrap()));
    assert!(!is_public_ip("::ffff:127.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("64:ff9b::192.0.2.33".parse().unwrap()));
    assert!(!is_public_ip("100::1".parse().unwrap()));
    assert!(!is_public_ip("2001::1".parse().unwrap()));
    assert!(!is_public_ip("2001:2::1".parse().unwrap()));
    assert!(!is_public_ip("2001:db8::1".parse().unwrap()));
    assert!(!is_public_ip("2002::1".parse().unwrap()));
    assert!(!is_public_ip("3fff::1".parse().unwrap()));
    assert!(!is_public_ip("5f00::1".parse().unwrap()));
    assert!(!is_public_ip("fc00::1".parse().unwrap()));
    assert!(!is_public_ip("fe80::1".parse().unwrap()));
    assert!(!is_public_ip("ff02::1".parse().unwrap()));
    assert!(is_public_ip("93.184.216.34".parse().unwrap()));
    assert!(is_public_ip(
        "2606:2800:220:1:248:1893:25c8:1946".parse().unwrap()
    ));
    assert!(is_public_ip("2001:4860:4860::8888".parse().unwrap()));
}

#[test]
fn local_asr_status_policy_retries_temporary_failures() {
    assert!(is_retryable_asr_status(
        reqwest::StatusCode::REQUEST_TIMEOUT
    ));
    assert!(is_retryable_asr_status(
        reqwest::StatusCode::TOO_MANY_REQUESTS
    ));
    assert!(is_retryable_asr_status(
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    ));
    assert!(!is_retryable_asr_status(reqwest::StatusCode::BAD_REQUEST));
}

#[test]
fn local_asr_response_parser_accepts_openai_json_shape() {
    assert_eq!(
        super::parse_asr_transcription_response(r#"{"text":" This is a local whisper test.\n"}"#)
            .as_deref(),
        Some("This is a local whisper test.")
    );
}

#[tokio::test]
async fn extract_returns_command_failed_on_non_zero_exit() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = "#!/bin/sh\necho 'something went wrong' >&2\nexit 1\n";
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let err = service
        .extract("abc123def45")
        .await
        .expect_err("should fail on non-zero exit");

    assert!(
        matches!(err, super::TranscriptError::CommandFailed(_)),
        "expected CommandFailed, got {err:?}"
    );
}

#[tokio::test]
async fn extract_detects_rate_limit_from_stderr() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = "#!/bin/sh\necho 'Error: rate limit exceeded (429)' >&2\nexit 1\n";
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let err = service
        .extract("abc123def45")
        .await
        .expect_err("should fail on rate limit");

    assert!(
        matches!(err, super::TranscriptError::RateLimited),
        "expected RateLimited, got {err:?}"
    );
}

#[tokio::test]
async fn extract_detects_no_transcript_from_stderr() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = "#!/bin/sh\necho 'Error: no transcript available for this video' >&2\nexit 1\n";
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let err = service
        .extract("abc123def45")
        .await
        .expect_err("should fail when no transcript");

    assert!(
        matches!(err, super::TranscriptError::NoTranscript),
        "expected NoTranscript, got {err:?}"
    );
}

#[tokio::test]
async fn extract_rejects_empty_output_as_no_transcript() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    // Mimics summarize exiting 0 with only whitespace when captions are unavailable.
    let script = "#!/bin/sh\nprintf '\\n'\n";
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let err = service
        .extract("abc123def45")
        .await
        .expect_err("should fail when output is empty");

    assert!(
        matches!(err, super::TranscriptError::NoTranscript),
        "expected NoTranscript, got {err:?}"
    );
}

#[tokio::test]
async fn extract_strips_transcript_header_prefix() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = "#!/bin/sh\nprintf 'Transcript:\\nHello world.\\n'\n";
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let (raw, _, _timed) = service
        .extract("abc123def45")
        .await
        .expect("extract should succeed");

    assert_eq!(raw, "Hello world.\n");
    assert!(!raw.starts_with("Transcript:"));
}

#[tokio::test]
async fn extract_retries_with_youtube_web_when_summarize_auto_output_truncates() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");

    // Simulate summarize's `--youtube auto` returning only a tiny snippet, while
    // `--youtube web` returns a longer transcript.
    let script = r#"#!/bin/sh
set -eu
if echo "$*" | grep -q "youtube auto"; then
  printf 'Transcript:\nSup nerds we got things to discuss.\n'
else
  printf 'Transcript:\nThis is a full transcript extracted via youtube web mode with enough words to avoid our short-snippet heuristic. It continues with additional lines for robust extraction.\n'
fi
"#;
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let (raw, _formatted, _timed) = service
        .extract("abc123def45")
        .await
        .expect("extract should succeed");

    assert!(
        raw.contains("full transcript extracted via youtube web mode"),
        "expected youtube=web transcript, got: {raw}"
    );
    assert!(
        !raw.contains("Sup nerds we got things to discuss."),
        "should not keep the truncated auto snippet"
    );
    assert!(!raw.starts_with("Transcript:"), "header should be stripped");
}

#[tokio::test]
async fn extract_rejects_youtube_site_wide_blurb() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = r#"#!/bin/sh
echo "Enjoy the videos and music you love, upload original content, and share it all with friends, family, and the world on YouTube."
"#;
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let err = service
        .extract("abc123def45")
        .await
        .expect_err("should fail when output is a site-wide placeholder");

    assert!(
        matches!(err, super::TranscriptError::NoTranscript),
        "expected NoTranscript, got {err:?}"
    );
}

#[tokio::test]
async fn extract_uses_single_direct_transcript_extraction_without_llm_formatting() {
    let dir = tempdir().expect("temp dir should be created");
    let script_path = dir.path().join("fake_summarize.sh");
    let script = r#"#!/bin/sh
set -eu
echo "OPENAI_BASE_URL=${OPENAI_BASE_URL:-}"
echo "OPENAI_API_KEY=${OPENAI_API_KEY:-}"
echo "HOME=${HOME:-}"
echo "XDG_CACHE_HOME=${XDG_CACHE_HOME:-}"
echo "XDG_CONFIG_HOME=${XDG_CONFIG_HOME:-}"
echo "ARGS=$*"
"#;
    fs::write(&script_path, script).expect("script should be written");
    let mut perms = fs::metadata(&script_path)
        .expect("metadata should be readable")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("script should be executable");

    let service =
        TranscriptService::with_path(script_path.to_str().expect("script path should be utf-8"));

    let (raw, formatted, timed) = service
        .extract("abc123def45")
        .await
        .expect("extract should succeed");

    assert_eq!(raw, formatted);
    assert!(
        timed.is_empty(),
        "summarize path should produce no timed segments"
    );
    assert!(formatted.contains("OPENAI_BASE_URL="));
    assert!(formatted.contains("OPENAI_API_KEY="));
    assert!(formatted.contains("HOME=/"));
    assert!(formatted.contains("XDG_CACHE_HOME=/"));
    assert!(formatted.contains("XDG_CONFIG_HOME=/"));
    assert!(formatted.contains("ARGS="));
    assert!(!formatted.contains("--markdown-mode"));
    assert!(!formatted.contains("--model"));
    assert!(formatted.contains("--youtube auto --extract --format text --plain --firecrawl off"));
}

#[test]
fn extract_json3_caption_url_prefers_en_orig_then_english_variants() {
    let metadata = r#"{
      "automatic_captions": {
        "fr": [{ "ext": "json3", "url": "https://example.test/fr.json3" }],
        "en": [{ "ext": "json3", "url": "https://example.test/en.json3" }],
        "en-orig": [{ "ext": "json3", "url": "https://example.test/en-orig.json3" }],
        "en-US": [{ "ext": "json3", "url": "https://example.test/en-us.json3" }]
      }
    }"#;

    assert_eq!(
        extract_json3_caption_url_from_ytdlp_metadata(metadata).as_deref(),
        Some("https://example.test/en-orig.json3")
    );
}

#[tokio::test]
async fn extract_uses_ytdlp_metadata_url_when_subtitle_file_is_not_written() {
    let dir = tempdir().expect("temp dir should be created");

    let summarize_path = dir.path().join("fake_summarize.sh");
    let summarize_script = r#"#!/bin/sh
set -eu
printf 'Transcript:\nSup nerds we got things to discuss.\n'
"#;
    fs::write(&summarize_path, summarize_script).expect("script should be written");

    let ytdlp_path = dir.path().join("fake_ytdlp.sh");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener addr should resolve");
    let json3_body = r#"{
      "events": [
        { "tStartMs": 0, "segs": [{ "utf8": "Hello from metadata" }] },
        { "tStartMs": 1000, "segs": [{ "utf8": "full transcript fallback" }] }
      ]
    }"#;
    let server_body = json3_body.to_string();
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request_buf = [0_u8; 1024];
            let _ = stream.read(&mut request_buf);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                server_body.len(),
                server_body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    let metadata_url = format!("http://{addr}/captions.json3");
    let ytdlp_script = format!(
        "#!/bin/sh\nset -eu\nif [ \"${{1:-}}\" = \"-J\" ]; then\n  printf '%s\\n' '{{\"automatic_captions\":{{\"en-orig\":[{{\"ext\":\"json3\",\"url\":\"{metadata_url}\"}}]}}}}'\n  exit 0\nfi\nexit 0\n"
    );
    fs::write(&ytdlp_path, ytdlp_script).expect("yt-dlp script should be written");

    for path in [&summarize_path, &ytdlp_path] {
        let mut perms = fs::metadata(path)
            .expect("metadata should be readable")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("script should be executable");
    }

    let service = TranscriptService::with_paths(
        summarize_path
            .to_str()
            .expect("summarize path should be utf-8"),
        ytdlp_path.to_str().expect("yt-dlp path should be utf-8"),
    );

    let (raw, _formatted, timed) = service
        .extract("abc123def45")
        .await
        .expect("extract should succeed");

    server.join().expect("server thread should join");

    assert_eq!(raw, "Hello from metadata full transcript fallback");
    assert_eq!(timed.len(), 2);
}
