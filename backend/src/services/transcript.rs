use std::process::Command;
use std::sync::Arc;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;
use tokio::sync::Semaphore;

use crate::services::http::build_http_client;
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
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

pub struct TranscriptService {
    summarize_path: String,
    ytdlp_path: String,
    concurrency_semaphore: Option<Arc<Semaphore>>,
}

impl TranscriptService {
    pub fn new() -> Self {
        Self {
            summarize_path: "/opt/homebrew/bin/summarize".to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
            concurrency_semaphore: None,
        }
    }

    pub fn with_path(summarize_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
            concurrency_semaphore: None,
        }
    }

    pub fn with_paths(summarize_path: &str, ytdlp_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: ytdlp_path.to_string(),
            concurrency_semaphore: None,
        }
    }

    pub fn with_concurrency_semaphore(mut self, semaphore: Arc<Semaphore>) -> Self {
        self.concurrency_semaphore = Some(semaphore);
        self
    }

    fn build_summarize_isolated_home(video_id: &str, youtube_mode: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "summarize-transcript-{video_id}-{youtube_mode}-{stamp}"
        ))
    }

    /// Extract transcript from a YouTube video using the summarize CLI.
    /// Returns (raw_text, formatted_markdown, timed_segments).
    /// Timed segments are only populated by the yt-dlp fallback path.
    pub async fn extract(
        &self,
        video_id: &str,
    ) -> Result<(String, String, Vec<crate::models::TimedSegment>), TranscriptError> {
        let _permit = if let Some(sem) = &self.concurrency_semaphore {
            Some(
                sem.clone()
                    .acquire_owned()
                    .await
                    .map_err(|e| TranscriptError::CommandFailed(e.to_string()))?,
            )
        } else {
            None
        };

        let video_url = format!("https://www.youtube.com/watch?v={video_id}");
        let started_at = Instant::now();

        async fn run_summarize_extract(
            summarize_path: &str,
            video_url: &str,
            video_id: &str,
            youtube_mode: &str,
        ) -> Result<String, TranscriptError> {
            tracing::info!(
                video_id = %video_id,
                youtube_mode = youtube_mode,
                "running summarize --extract for transcript"
            );
            let output = tokio::task::spawn_blocking({
                let path = summarize_path.to_string();
                let url = video_url.to_string();
                let youtube_mode = youtube_mode.to_string();
                let isolated_home =
                    TranscriptService::build_summarize_isolated_home(video_id, &youtube_mode);
                move || {
                    let cache_dir = isolated_home.join(".cache");
                    let config_dir = isolated_home.join(".config");
                    let _ = std::fs::create_dir_all(&cache_dir);
                    let _ = std::fs::create_dir_all(&config_dir);

                    let output = Command::new(&path)
                        .arg(&url)
                        .arg("--youtube")
                        .arg(&youtube_mode)
                        .arg("--extract")
                        .arg("--format")
                        .arg("text")
                        .arg("--plain")
                        .arg("--firecrawl")
                        .arg("off")
                        .env("HOME", &isolated_home)
                        .env("XDG_CACHE_HOME", &cache_dir)
                        .env("XDG_CONFIG_HOME", &config_dir)
                        .output();

                    let _ = std::fs::remove_dir_all(&isolated_home);
                    output
                }
            })
            .await
            .map_err(|e| TranscriptError::CommandFailed(e.to_string()))??;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    video_id = %video_id,
                    youtube_mode = youtube_mode,
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
            Ok(raw
                .strip_prefix("Transcript:\n")
                .unwrap_or(&raw)
                .to_string())
        }

        // Flags rationale:
        // --youtube auto   tries captionTracks/youtubei first
        // --extract        print raw transcript and exit, no LLM summarization
        // --format text    plain text output (not markdown)
        // --plain          strip ANSI/OSC terminal formatting from stdout
        // --firecrawl off  disable web-scraping fallback that silently returns the YouTube
        //                  site-wide og:description blurb when captions are unavailable
        // summarize can also serve a cached HTML page extraction when its transcript provider
        // previously resolved as unavailable. That failure mode produces a short first-cue
        // snippet instead of a real transcript, so we treat tiny outputs as suspect below.
        let raw_auto =
            run_summarize_extract(&self.summarize_path, &video_url, video_id, "auto").await?;

        if raw_auto.trim().is_empty() {
            tracing::info!(
                video_id = %video_id,
                "summarize returned empty output - trying yt-dlp fallback"
            );
            return self.extract_with_ytdlp(video_id).await;
        }

        if is_site_wide_placeholder_description(&raw_auto) {
            tracing::warn!(
                video_id = %video_id,
                "summarize returned YouTube site-wide blurb - trying yt-dlp fallback"
            );
            return self.extract_with_ytdlp(video_id).await;
        }

        // `--youtube auto` can sometimes return only a tiny snippet even when captions exist.
        // If that happens, retry with `--youtube web` and accept it unless it is still clearly
        // truncated and yt-dlp is available (then prefer the fallback extraction).
        if looks_like_summarize_auto_output_truncation(&raw_auto) {
            tracing::warn!(
                video_id = %video_id,
                raw_chars = raw_auto.chars().count(),
                "summarize auto output looks truncated - retrying with youtube=web"
            );

            let raw_web =
                run_summarize_extract(&self.summarize_path, &video_url, video_id, "web").await?;

            let yt_dlp_available = std::path::Path::new(&self.ytdlp_path).exists();
            if raw_web.trim().is_empty() || is_site_wide_placeholder_description(&raw_web) {
                if yt_dlp_available {
                    tracing::info!(
                        video_id = %video_id,
                        "summarize youtube=web retry returned empty/placeholder - trying yt-dlp fallback"
                    );
                    return self.extract_with_ytdlp(video_id).await;
                }

                tracing::info!(
                    video_id = %video_id,
                    "summarize youtube=web retry returned empty/placeholder but yt-dlp is unavailable; returning summarize=auto output"
                );
                let formatted = raw_auto.clone();
                return Ok((raw_auto, formatted, Vec::new()));
            }

            if looks_like_summarize_auto_output_truncation(&raw_web) && !yt_dlp_available {
                // In unit tests we often don't have yt-dlp installed. Prefer returning
                // the best available summarize output over failing the transcript request.
                let formatted = raw_web.clone();
                tracing::info!(
                    video_id = %video_id,
                    elapsed_ms = started_at.elapsed().as_millis(),
                    raw_chars = raw_web.chars().count(),
                    "transcript extraction completed (yt-dlp unavailable, returning summarize=web output)"
                );
                return Ok((raw_web, formatted, Vec::new()));
            }

            if looks_like_summarize_auto_output_truncation(&raw_web) {
                tracing::info!(
                    video_id = %video_id,
                    "summarize web retry still looks truncated - trying yt-dlp fallback"
                );
                return self.extract_with_ytdlp(video_id).await;
            }

            let formatted = raw_web.clone();
            tracing::info!(
                video_id = %video_id,
                elapsed_ms = started_at.elapsed().as_millis(),
                raw_bytes = raw_web.len(),
                formatted_bytes = formatted.len(),
                "transcript extraction completed (summarize youtube=web)"
            );
            return Ok((raw_web, formatted, Vec::new()));
        }

        let formatted = raw_auto.clone();
        tracing::info!(
            video_id = %video_id,
            elapsed_ms = started_at.elapsed().as_millis(),
            raw_bytes = raw_auto.len(),
            formatted_bytes = formatted.len(),
            "transcript extraction completed (summarize youtube=auto)"
        );

        // Summarize CLI path produces no timed segments.
        Ok((raw_auto, formatted, Vec::new()))
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

        if let Some(json3_url) = self.resolve_ytdlp_json3_url(video_id).await? {
            tracing::info!(video_id = %video_id, "yt-dlp resolved json3 caption URL");

            match self.fetch_json3_from_url(&json3_url).await {
                Ok(json3_content) => {
                    if let Some(result) = parse_ytdlp_json3_result(video_id, &json3_content) {
                        return Ok(result);
                    }
                    tracing::warn!(
                        video_id = %video_id,
                        "yt-dlp metadata URL returned json3 that parsed to empty transcript - trying legacy subtitle file fallback"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        video_id = %video_id,
                        error = %err,
                        "fetching yt-dlp json3 caption URL failed - trying legacy subtitle file fallback"
                    );
                }
            }
        } else {
            tracing::info!(
                video_id = %video_id,
                "yt-dlp metadata did not expose a json3 caption URL - trying legacy subtitle file fallback"
            );
        }

        let json3_content = self.extract_with_ytdlp_subtitle_file(video_id).await?;
        if let Some(result) = parse_ytdlp_json3_result(video_id, &json3_content) {
            return Ok(result);
        }

        tracing::info!(
            video_id = %video_id,
            "yt-dlp json3 parsed to empty text after metadata and legacy fallbacks"
        );
        Err(TranscriptError::NoTranscript)
    }

    async fn resolve_ytdlp_json3_url(
        &self,
        video_id: &str,
    ) -> Result<Option<String>, TranscriptError> {
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        let ytdlp_path = self.ytdlp_path.clone();

        let output = tokio::task::spawn_blocking(move || {
            Command::new(&ytdlp_path)
                .arg("-J")
                .arg(&url)
                .arg("--quiet")
                .arg("--no-warnings")
                .arg("--ignore-no-formats-error")
                .arg("--extractor-args")
                .arg("youtube:player_client=ios")
                .output()
        })
        .await
        .map_err(|e| TranscriptError::CommandFailed(e.to_string()))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                video_id = %video_id,
                status = output.status.code().unwrap_or(-1),
                error_output = %stderr.trim(),
                "yt-dlp metadata probe failed"
            );
            return Ok(None);
        }

        let metadata = String::from_utf8_lossy(&output.stdout);
        Ok(extract_json3_caption_url_from_ytdlp_metadata(&metadata))
    }

    async fn fetch_json3_from_url(&self, url: &str) -> Result<String, TranscriptError> {
        let response = build_http_client().get(url).send().await?;
        if !response.status().is_success() {
            return Err(TranscriptError::CommandFailed(format!(
                "json3 caption fetch returned HTTP {}",
                response.status()
            )));
        }
        Ok(response.text().await?)
    }

    async fn extract_with_ytdlp_subtitle_file(
        &self,
        video_id: &str,
    ) -> Result<String, TranscriptError> {
        let tmp_dir = std::env::temp_dir().join(format!("ytdlp_{video_id}"));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let output_template = tmp_dir.join("%(id)s").to_string_lossy().to_string();
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        let ytdlp_path = self.ytdlp_path.clone();

        let output = tokio::task::spawn_blocking({
            let url = url.clone();
            let template = output_template.clone();
            move || {
                Command::new(&ytdlp_path)
                    .arg(&url)
                    .arg("--skip-download")
                    .arg("--write-auto-subs")
                    .arg("--sub-langs")
                    .arg("en.*,en")
                    .arg("--sub-format")
                    .arg("json3")
                    .arg("--ignore-no-formats-error")
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
        .await
        .map_err(|e| TranscriptError::CommandFailed(e.to_string()))??;

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

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                video_id = %video_id,
                status = output.status.code().unwrap_or(-1),
                error_output = %stderr.trim(),
                "yt-dlp legacy subtitle-file fallback exited non-zero"
            );
        }

        if json3_content.trim().is_empty() {
            tracing::info!(video_id = %video_id, "yt-dlp returned no captions");
            return Err(TranscriptError::NoTranscript);
        }

        Ok(json3_content)
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

fn parse_ytdlp_json3_result(
    video_id: &str,
    json3_content: &str,
) -> Option<(String, String, Vec<crate::models::TimedSegment>)> {
    let (raw, timed) = parse_json3_transcript(json3_content);
    if raw.trim().is_empty() {
        return None;
    }

    tracing::info!(
        video_id = %video_id,
        bytes = raw.len(),
        timed_segments = timed.len(),
        "yt-dlp transcript extracted"
    );
    Some((raw.clone(), raw, timed))
}

fn extract_json3_caption_url_from_ytdlp_metadata(content: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(content).ok()?;

    find_json3_caption_url_in_track_map(value.get("automatic_captions"))
        .or_else(|| find_json3_caption_url_in_track_map(value.get("subtitles")))
}

fn find_json3_caption_url_in_track_map(track_map: Option<&serde_json::Value>) -> Option<String> {
    let tracks = track_map?.as_object()?;

    let mut preferred_keys = Vec::new();
    if tracks.contains_key("en-orig") {
        preferred_keys.push("en-orig".to_string());
    }

    let mut english_variants = tracks
        .keys()
        .filter(|key| key.starts_with("en-") && key.as_str() != "en-orig")
        .cloned()
        .collect::<Vec<_>>();
    english_variants.sort();
    preferred_keys.extend(english_variants);

    if tracks.contains_key("en") {
        preferred_keys.push("en".to_string());
    }

    let mut remaining_keys = tracks
        .keys()
        .filter(|key| !preferred_keys.iter().any(|preferred| preferred == *key))
        .cloned()
        .collect::<Vec<_>>();
    remaining_keys.sort();
    preferred_keys.extend(remaining_keys);

    preferred_keys.into_iter().find_map(|key| {
        tracks
            .get(&key)
            .and_then(|entries| entries.as_array())
            .and_then(|entries| {
                entries.iter().find_map(|entry| {
                    let ext = entry.get("ext").and_then(|value| value.as_str())?;
                    let url = entry.get("url").and_then(|value| value.as_str())?;
                    (ext == "json3" && !url.trim().is_empty()).then(|| url.to_string())
                })
            })
    })
}

/// Heuristic for detecting summarize's `--youtube auto` failure mode.
///
/// Some videos return only a first-cue snippet even when captions exist, which produces
/// transcripts that are "non-empty" but far too small to be useful.
fn looks_like_summarize_auto_output_truncation(raw: &str) -> bool {
    let text = raw.trim();
    if text.is_empty() {
        return true;
    }

    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();

    // Tuned to catch single-line/snippet failures while avoiding rejecting "normal" short
    // transcripts.
    char_count < 120 && word_count < 25
}

impl Default for TranscriptService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;
    use std::thread;

    use super::{TranscriptService, extract_json3_caption_url_from_ytdlp_metadata};
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

        let service = TranscriptService::with_path(
            script_path.to_str().expect("script path should be utf-8"),
        );

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
        assert!(formatted.contains("HOME=/"));
        assert!(formatted.contains("XDG_CACHE_HOME=/"));
        assert!(formatted.contains("XDG_CONFIG_HOME=/"));
        assert!(formatted.contains("ARGS="));
        assert!(!formatted.contains("--markdown-mode"));
        assert!(!formatted.contains("--model"));
        assert!(
            formatted.contains("--youtube auto --extract --format text --plain --firecrawl off")
        );
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
}
