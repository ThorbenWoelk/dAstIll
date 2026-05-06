use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::multipart::{Form, Part};
use reqwest::redirect::Policy;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::config::{LocalAsrAuthMode, LocalAsrRuntimeConfig};
use crate::services::http::build_http_client;
use crate::services::youtube::placeholder::is_site_wide_placeholder_description;

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

fn extract_json3_caption_url_from_ytdlp_metadata(content: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(content).ok()?;

    find_json3_caption_url_in_track_map(value.get("automatic_captions"))
        .or_else(|| find_json3_caption_url_in_track_map(value.get("subtitles")))
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

fn is_retryable_asr_status(status: reqwest::StatusCode) -> bool {
    status.is_server_error()
        || matches!(
            status,
            reqwest::StatusCode::REQUEST_TIMEOUT | reqwest::StatusCode::TOO_MANY_REQUESTS
        )
}

async fn fetch_google_identity_token(audience: &str) -> Result<String, TranscriptError> {
    let mut url = reqwest::Url::parse(
        "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity",
    )
    .map_err(|error| {
        TranscriptError::AsrTemporarilyUnavailable(format!("invalid metadata URL: {error}"))
    })?;
    url.query_pairs_mut()
        .append_pair("audience", audience)
        .append_pair("format", "full");

    let response = build_http_client()
        .get(url)
        .header("Metadata-Flavor", "Google")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?;

    if !response.status().is_success() {
        return Err(TranscriptError::AsrTemporarilyUnavailable(format!(
            "metadata identity token returned HTTP {}",
            response.status()
        )));
    }

    let token = response
        .text()
        .await
        .map(|token| token.trim().to_string())
        .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?;
    if token.is_empty() {
        return Err(TranscriptError::AsrTemporarilyUnavailable(
            "metadata identity token response was empty".to_string(),
        ));
    }
    Ok(token)
}

async fn authorize_local_asr_request(
    request: reqwest::RequestBuilder,
    config: &LocalAsrRuntimeConfig,
) -> Result<reqwest::RequestBuilder, TranscriptError> {
    match config.auth_mode {
        LocalAsrAuthMode::ApiKey => Ok(request.bearer_auth(&config.api_key)),
        LocalAsrAuthMode::GoogleIdToken => {
            let token = fetch_google_identity_token(&config.audience_url()).await?;
            Ok(request.bearer_auth(token))
        }
    }
}

fn parse_asr_transcription_response(body: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(body).ok()?;
    value
        .get("text")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn is_public_ipv4(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    !(ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_documentation()
        || octets[0] == 0
        || octets[0] >= 240
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 169 && octets[1] == 254)
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 192 && octets[1] == 88 && octets[2] == 99)
        || (octets[0] == 198 && (18..=19).contains(&octets[1]))
        || octets == [169, 254, 169, 254])
}

fn is_public_ipv6(ip: std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    let is_global_unicast = (segments[0] & 0xe000) == 0x2000;
    is_global_unicast
        && !(
            // NAT64 well-known and local-use prefixes can tunnel private IPv4 targets.
            (segments[0] == 0x0064 && segments[1] == 0xff9b)
                || (segments[0] == 0x0064 && segments[1] == 0xff9b && segments[2] == 0x0001)
                // Discard-only prefix.
                || (segments[0] == 0x0100 && segments[1] == 0)
                // IETF protocol assignments, including Teredo, benchmarking, and documentation.
                || (segments[0] == 0x2001 && segments[1] <= 0x01ff)
                || (segments[0] == 0x2001 && segments[1] == 0x0db8)
                // 6to4.
                || segments[0] == 0x2002
                // Documentation range from the IANA special-purpose registry.
                || (segments[0] == 0x3fff && (segments[1] & 0xf000) == 0)
        )
}

pub(crate) fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(is_public_ipv4)
            .unwrap_or_else(|| is_public_ipv6(ip)),
    }
}

pub(crate) async fn validate_public_media_url(url: &str) -> Result<(), TranscriptError> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|error| TranscriptError::CommandFailed(format!("invalid audio URL: {error}")))?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(TranscriptError::CommandFailed(format!(
                "unsupported audio URL scheme: {scheme}"
            )));
        }
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| TranscriptError::CommandFailed("audio URL missing host".to_string()))?;
    if host.eq_ignore_ascii_case("localhost") {
        return Err(TranscriptError::CommandFailed(
            "audio URL host is not allowed".to_string(),
        ));
    }

    let port = parsed.port_or_known_default().unwrap_or(443);
    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public_ip(ip) {
            return Err(TranscriptError::CommandFailed(
                "audio URL IP is not allowed".to_string(),
            ));
        }
        return Ok(());
    }

    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| TranscriptError::CommandFailed(format!("audio DNS failed: {error}")))?;
    for addr in addrs {
        if !is_public_ip(addr.ip()) {
            return Err(TranscriptError::CommandFailed(
                "audio URL resolves to a private or local address".to_string(),
            ));
        }
    }

    Ok(())
}

async fn resolve_public_socket_addrs(
    host: &str,
    port: u16,
) -> Result<Vec<SocketAddr>, TranscriptError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_public_ip(ip) {
            return Ok(vec![SocketAddr::new(ip, port)]);
        }
        return Err(TranscriptError::CommandFailed(
            "media URL IP is not allowed".to_string(),
        ));
    }

    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| TranscriptError::CommandFailed(format!("media DNS failed: {error}")))?
        .filter(|addr| is_public_ip(addr.ip()))
        .collect::<Vec<_>>();

    if addrs.is_empty() {
        return Err(TranscriptError::CommandFailed(
            "media URL does not resolve to a public address".to_string(),
        ));
    }

    Ok(addrs)
}

async fn build_pinned_public_media_client(
    url: &reqwest::Url,
    timeout_secs: u64,
) -> Result<reqwest::Client, TranscriptError> {
    let host = url
        .host_str()
        .ok_or_else(|| TranscriptError::CommandFailed("media URL missing host".to_string()))?;
    let port = url.port_or_known_default().unwrap_or(443);
    let addrs = resolve_public_socket_addrs(host, port).await?;
    reqwest::ClientBuilder::new()
        .user_agent("dastill/0.1")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(Policy::none())
        .resolve_to_addrs(host, &addrs)
        .build()
        .map_err(|error| {
            TranscriptError::CommandFailed(format!("http client build failed: {error}"))
        })
}

pub(crate) async fn fetch_public_response(
    initial_url: &str,
    timeout_secs: u64,
) -> Result<reqwest::Response, TranscriptError> {
    let mut url = reqwest::Url::parse(initial_url)
        .map_err(|error| TranscriptError::CommandFailed(format!("invalid media URL: {error}")))?;
    for _ in 0..15 {
        let client = build_pinned_public_media_client(&url, timeout_secs).await?;
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?;
        if !response.status().is_redirection() {
            return Ok(response);
        }
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                TranscriptError::CommandFailed("redirect missing Location header".to_string())
            })?;
        url = url.join(location).map_err(|error| {
            TranscriptError::CommandFailed(format!("invalid redirect URL: {error}"))
        })?;
    }

    Err(TranscriptError::CommandFailed(
        "too many redirects while fetching media".to_string(),
    ))
}

async fn download_audio_bytes(
    url: &str,
    max_audio_bytes: u64,
    timeout_secs: u64,
) -> Result<(Vec<u8>, Option<String>), TranscriptError> {
    let response = fetch_public_response(url, timeout_secs).await?;

    if !response.status().is_success() {
        return Err(TranscriptError::CommandFailed(format!(
            "audio fetch returned HTTP {}",
            response.status()
        )));
    }

    if let Some(length) = response.content_length() {
        if length > max_audio_bytes {
            return Err(TranscriptError::CommandFailed(format!(
                "audio file is too large: {length} bytes"
            )));
        }
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let mut body = Vec::new();
    let mut response = response;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?
    {
        let next_len = body.len().saturating_add(chunk.len());
        if next_len as u64 > max_audio_bytes {
            return Err(TranscriptError::CommandFailed(format!(
                "audio file is too large: more than {max_audio_bytes} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }

    Ok((body, content_type))
}

#[derive(Error, Debug)]
pub enum TranscriptError {
    #[error("Transcript extraction failed: {0}")]
    CommandFailed(String),
    #[error("Video has no transcript available")]
    NoTranscript,
    #[error("Local ASR provider is not configured")]
    AsrUnavailable,
    #[error("Local ASR provider is temporarily unavailable: {0}")]
    AsrTemporarilyUnavailable(String),
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
    local_asr: Option<LocalAsrRuntimeConfig>,
    concurrency_semaphore: Option<Arc<Semaphore>>,
}

impl TranscriptService {
    pub fn new() -> Self {
        Self {
            summarize_path: "/opt/homebrew/bin/summarize".to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
            local_asr: None,
            concurrency_semaphore: None,
        }
    }

    pub fn with_path(summarize_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: "/usr/local/bin/yt-dlp".to_string(),
            local_asr: None,
            concurrency_semaphore: None,
        }
    }

    pub fn with_paths(summarize_path: &str, ytdlp_path: &str) -> Self {
        Self {
            summarize_path: summarize_path.to_string(),
            ytdlp_path: ytdlp_path.to_string(),
            local_asr: None,
            concurrency_semaphore: None,
        }
    }

    pub fn with_local_asr(mut self, config: Option<LocalAsrRuntimeConfig>) -> Self {
        self.local_asr = config;
        self
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

    pub async fn extract_podcast_audio(
        &self,
        video_id: &str,
        audio_url: &str,
        mime_type: Option<&str>,
    ) -> Result<(String, String, Vec<crate::models::TimedSegment>), TranscriptError> {
        let Some(config) = self.local_asr.as_ref() else {
            tracing::warn!(
                video_id = %video_id,
                "local podcast ASR requested but LOCAL_ASR is not configured"
            );
            return Err(TranscriptError::AsrUnavailable);
        };

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

        validate_public_media_url(audio_url).await?;
        let started_at = Instant::now();
        let form = match config.auth_mode {
            LocalAsrAuthMode::GoogleIdToken => {
                let mut form = Form::new()
                    .text("model", config.model.clone())
                    .text("response_format", "json")
                    .text("audio_url", audio_url.to_string());
                if let Some(mime_type) = mime_type.filter(|value| !value.trim().is_empty()) {
                    form = form.text("mime_type", mime_type.to_string());
                }
                form
            }
            LocalAsrAuthMode::ApiKey => {
                let (audio, content_type) =
                    download_audio_bytes(audio_url, config.max_audio_bytes, config.timeout_secs)
                        .await?;
                let _content_type = mime_type
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(content_type.as_deref().unwrap_or("audio/mpeg"));
                Form::new()
                    .text("model", config.model.clone())
                    .text("response_format", "text")
                    .part("file", Part::bytes(audio).file_name("podcast-audio"))
            }
        };

        let client = build_http_client();
        let request = client
            .post(config.transcription_url())
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .multipart(form);
        let request = authorize_local_asr_request(request, config).await?;
        let response = request
            .send()
            .await
            .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(
                video_id = %video_id,
                status = %status,
                body = %body.trim(),
                "local ASR transcription request failed"
            );
            if is_retryable_asr_status(status) {
                return Err(TranscriptError::AsrTemporarilyUnavailable(format!(
                    "local ASR returned HTTP {status}"
                )));
            }
            return Err(TranscriptError::CommandFailed(format!(
                "local ASR returned HTTP {status}"
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|error| TranscriptError::AsrTemporarilyUnavailable(error.to_string()))?;
        let raw =
            parse_asr_transcription_response(&body).unwrap_or_else(|| body.trim().to_string());
        if raw.is_empty() {
            return Err(TranscriptError::NoTranscript);
        }

        tracing::info!(
            video_id = %video_id,
            provider = "local-openai-compatible",
            model = %config.model,
            elapsed_ms = started_at.elapsed().as_millis(),
            transcript_bytes = raw.len(),
            "podcast audio transcription completed"
        );
        Ok((raw.clone(), raw, Vec::new()))
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

impl Default for TranscriptService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "transcript_tests.rs"]
mod transcript_tests;
