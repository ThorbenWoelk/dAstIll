use std::process::Command;
use std::time::Instant;
use thiserror::Error;

use crate::services::youtube::placeholder::is_site_wide_placeholder_description;

#[derive(Error, Debug)]
pub enum TranscriptError {
    #[error("Transcript extraction failed: {0}")]
    CommandFailed(String),
    #[error("Video has no transcript available")]
    NoTranscript,
    #[error("Rate limited, try again later")]
    RateLimited,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct TranscriptService {
    summarize_path: String,
    ytdlp_path: String,
}

impl TranscriptService {
    pub fn new() -> Self {
        Self {
            summarize_path: "/opt/homebrew/bin/summarize".to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
        }
    }

    pub fn with_path(summarize_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
        }
    }

    pub fn with_paths(summarize_path: &str, ytdlp_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: ytdlp_path.to_string(),
        }
    }

    /// Extract transcript from a YouTube video using the summarize CLI.
    /// Returns (raw_text, formatted_markdown, timed_segments).
    /// Timed segments are only populated by the yt-dlp fallback path.
    pub async fn extract(
        &self,
        video_id: &str,
    ) -> Result<(String, String, Vec<crate::models::TimedSegment>), TranscriptError> {
        let video_url = format!("https://www.youtube.com/watch?v={video_id}");
        let started_at = Instant::now();

        tracing::info!(video_id = %video_id, "running summarize --extract for transcript");

        // Flags rationale:
        // --youtube auto   explicit mode; tries captionTracks/youtubei first
        // --extract        print raw transcript and exit, no LLM summarization
        // --format text    plain text output (not markdown)
        // --plain          strip ANSI/OSC terminal formatting from stdout
        // --firecrawl off  disable web-scraping fallback that silently returns the YouTube
        //                  site-wide og:description blurb when captions are unavailable
        let output = tokio::task::spawn_blocking({
            let path = self.summarize_path.clone();
            let url = video_url.clone();
            move || {
                Command::new(&path)
                    .arg(&url)
                    .arg("--youtube")
                    .arg("auto")
                    .arg("--extract")
                    .arg("--format")
                    .arg("text")
                    .arg("--plain")
                    .arg("--firecrawl")
                    .arg("off")
                    .output()
            }
        })
        .await
        .map_err(|e| TranscriptError::CommandFailed(e.to_string()))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                video_id = %video_id,
                status = output.status.code().unwrap_or(-1),
                error_output = %stderr.trim(),
                "summarize transcript command failed"
            );
            let stderr_lower = stderr.to_lowercase();
            if stderr_lower.contains("rate limit") || stderr_lower.contains("429") {
                return Err(TranscriptError::RateLimited);
            }
            if stderr_lower.contains("no transcript")
                || stderr_lower.contains("subtitles are disabled")
            {
                return Err(TranscriptError::NoTranscript);
            }
            return Err(TranscriptError::CommandFailed(stderr.to_string()));
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        // summarize prefixes transcript output with "Transcript:\n"; strip it.
        let raw = raw
            .strip_prefix("Transcript:\n")
            .unwrap_or(&raw)
            .to_string();

        if raw.trim().is_empty() {
            tracing::info!(video_id = %video_id, "summarize returned empty output - trying yt-dlp fallback");
            return self.extract_with_ytdlp(video_id).await;
        }

        if is_site_wide_placeholder_description(&raw) {
            tracing::warn!(video_id = %video_id, "summarize returned YouTube site-wide blurb - trying yt-dlp fallback");
            return self.extract_with_ytdlp(video_id).await;
        }

        let formatted = raw.clone();
        tracing::info!(
            video_id = %video_id,
            elapsed_ms = started_at.elapsed().as_millis(),
            raw_bytes = raw.len(),
            formatted_bytes = formatted.len(),
            "transcript extraction completed"
        );

        // Summarize CLI path produces no timed segments.
        Ok((raw, formatted, Vec::new()))
    }

    /// Fallback transcript extraction using yt-dlp with the iOS YouTube client.
    /// Called when `summarize` exits 0 with empty output (GCP IP blocking).
    /// Uses `--extractor-args youtube:player_client=ios` to hit YouTube's mobile API,
    /// which uses different endpoints and is less likely to be blocked on cloud IPs.
    async fn extract_with_ytdlp(
        &self,
        video_id: &str,
    ) -> Result<(String, String, Vec<crate::models::TimedSegment>), TranscriptError> {
        if !std::path::Path::new(&self.ytdlp_path).exists() {
            tracing::debug!(
                video_id = %video_id,
                path = %self.ytdlp_path,
                "yt-dlp not found, skipping fallback"
            );
            return Err(TranscriptError::NoTranscript);
        }

        tracing::info!(video_id = %video_id, "running yt-dlp fallback for transcript");

        let tmp_dir = std::env::temp_dir().join(format!("ytdlp_{video_id}"));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let output_template = tmp_dir.join("%(id)s").to_string_lossy().to_string();
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        let ytdlp_path = self.ytdlp_path.clone();

        let _ = tokio::task::spawn_blocking({
            let url = url.clone();
            let template = output_template.clone();
            move || {
                Command::new(&ytdlp_path)
                    .arg(&url)
                    .arg("--skip-download")
                    .arg("--write-auto-subs")
                    .arg("--sub-lang")
                    .arg("en")
                    .arg("--sub-format")
                    .arg("json3")
                    .arg("-o")
                    .arg(&template)
                    .arg("--quiet")
                    .arg("--no-warnings")
                    // iOS client uses mobile API endpoints, bypassing web-scraper blocking on GCP IPs.
                    .arg("--extractor-args")
                    .arg("youtube:player_client=ios")
                    .output()
            }
        })
        .await;

        // Search the tmp dir for any *.json3 file yt-dlp may have written.
        let json3_content = std::fs::read_dir(&tmp_dir)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "json3")
                            .unwrap_or(false)
                    })
                    .and_then(|e| std::fs::read_to_string(e.path()).ok())
            })
            .unwrap_or_default();

        let _ = std::fs::remove_dir_all(&tmp_dir);

        if json3_content.trim().is_empty() {
            tracing::info!(video_id = %video_id, "yt-dlp returned no captions");
            return Err(TranscriptError::NoTranscript);
        }

        let (raw, timed) = parse_json3_transcript(&json3_content);
        if raw.trim().is_empty() {
            tracing::info!(video_id = %video_id, "yt-dlp json3 parsed to empty text");
            return Err(TranscriptError::NoTranscript);
        }

        tracing::info!(
            video_id = %video_id,
            bytes = raw.len(),
            timed_segments = timed.len(),
            "yt-dlp transcript extracted"
        );
        Ok((raw.clone(), raw, timed))
    }

    /// Check if summarize CLI is available.
    pub fn is_available(&self) -> bool {
        std::path::Path::new(&self.summarize_path).exists()
    }
}

/// Parse YouTube's json3 subtitle format into (plain_text, timed_segments).
/// Each event has `tStartMs` (start time in milliseconds) and `segs` (text segments).
/// Timed segments use the event's start time; events without `tStartMs` are included
/// in the plain text but omitted from the timed list.
fn parse_json3_transcript(content: &str) -> (String, Vec<crate::models::TimedSegment>) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return (String::new(), Vec::new());
    };
    let Some(events) = value["events"].as_array() else {
        return (String::new(), Vec::new());
    };

    let mut plain_parts: Vec<String> = Vec::new();
    let mut timed: Vec<crate::models::TimedSegment> = Vec::new();

    for event in events {
        let start_ms = event["tStartMs"].as_f64();
        let mut event_words: Vec<String> = Vec::new();

        if let Some(segs) = event["segs"].as_array() {
            for seg in segs {
                if let Some(utf8) = seg["utf8"].as_str() {
                    let text = utf8.replace('\n', " ");
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        event_words.push(text);
                    }
                }
            }
        }

        if event_words.is_empty() {
            continue;
        }

        let event_text = event_words.join(" ");
        plain_parts.push(event_text.clone());

        if let Some(ms) = start_ms {
            timed.push(crate::models::TimedSegment {
                start_sec: (ms / 1000.0) as f32,
                text: event_text,
            });
        }
    }

    let plain = plain_parts
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    (plain, timed)
}

impl Default for TranscriptService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::TranscriptService;
    use tempfile::tempdir;

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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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
        let script =
            "#!/bin/sh\necho 'Error: no transcript available for this video' >&2\nexit 1\n";
        fs::write(&script_path, script).expect("script should be written");
        let mut perms = fs::metadata(&script_path)
            .expect("metadata should be readable")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("script should be executable");

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

        let (raw, _, _timed) = service
            .extract("abc123def45")
            .await
            .expect("extract should succeed");

        assert_eq!(raw, "Hello world.\n");
        assert!(!raw.starts_with("Transcript:"));
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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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
echo "ARGS=$*"
"#;
        fs::write(&script_path, script).expect("script should be written");
        let mut perms = fs::metadata(&script_path)
            .expect("metadata should be readable")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("script should be executable");

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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
        assert!(formatted.contains("ARGS="));
        assert!(!formatted.contains("--markdown-mode"));
        assert!(!formatted.contains("--model"));
        assert!(
            formatted.contains("--youtube auto --extract --format text --plain --firecrawl off")
        );
    }
}
