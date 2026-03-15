use std::process::Command;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscriptError {
    #[error("Summarize command failed: {0}")]
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
}

impl TranscriptService {
    pub fn new() -> Self {
        Self {
            summarize_path: "/opt/homebrew/bin/summarize".to_string(),
        }
    }

    pub fn with_path(path: &str) -> Self {
        Self {
            summarize_path: path.to_string(),
        }
    }

    /// Extract transcript from a YouTube video using the summarize CLI.
    /// Returns (raw_text, formatted_markdown).
    pub async fn extract(&self, video_id: &str) -> Result<(String, String), TranscriptError> {
        let video_url = format!("https://www.youtube.com/watch?v={video_id}");
        let started_at = Instant::now();

        tracing::info!(video_id = %video_id, "running summarize --extract for transcript");

        // Run summarize with --extract to get just the transcript
        let output = tokio::task::spawn_blocking({
            let path = self.summarize_path.clone();
            let url = video_url.clone();
            move || {
                Command::new(&path)
                    .arg(&url)
                    .arg("--extract")
                    .arg("--format")
                    .arg("md")
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

        let formatted = String::from_utf8_lossy(&output.stdout).to_string();
        tracing::info!(
            video_id = %video_id,
            elapsed_ms = started_at.elapsed().as_millis(),
            markdown_bytes = formatted.len(),
            "transcript markdown extracted"
        );

        // Also get raw text version for potential processing
        let raw_output = tokio::task::spawn_blocking({
            let path = self.summarize_path.clone();
            move || {
                Command::new(&path)
                    .arg(&video_url)
                    .arg("--extract")
                    .arg("--format")
                    .arg("txt")
                    .output()
            }
        })
        .await
        .map_err(|e| TranscriptError::CommandFailed(e.to_string()))??;

        let raw = if raw_output.status.success() {
            String::from_utf8_lossy(&raw_output.stdout).to_string()
        } else {
            let raw_stderr = String::from_utf8_lossy(&raw_output.stderr);
            tracing::warn!(
                video_id = %video_id,
                status = raw_output.status.code().unwrap_or(-1),
                error_output = %raw_stderr.trim(),
                "raw text transcript extraction failed - falling back to markdown"
            );
            // Fall back to using formatted as raw if txt extraction fails
            formatted.clone()
        };

        tracing::info!(
            video_id = %video_id,
            elapsed_ms = started_at.elapsed().as_millis(),
            raw_bytes = raw.len(),
            "transcript extraction completed"
        );

        Ok((raw, formatted))
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
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::TranscriptService;
    use tempfile::tempdir;

    #[tokio::test]
    async fn extract_uses_direct_transcript_extraction_without_llm_formatting() {
        let dir = tempdir().expect("temp dir should be created");
        let script_path = dir.path().join("fake_summarize.sh");
        let script = r#"#!/bin/sh
set -eu

format=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "--format" ]; then
    format="$arg"
    break
  fi
  prev="$arg"
done

if [ "$format" = "md" ]; then
  echo "OPENAI_BASE_URL=${OPENAI_BASE_URL:-}"
  echo "OPENAI_API_KEY=${OPENAI_API_KEY:-}"
  echo "ARGS=$*"
else
  echo "raw text"
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

        let (raw, formatted) = service
            .extract("abc123def45")
            .await
            .expect("extract should succeed");

        assert_eq!(raw.trim(), "raw text");
        assert!(formatted.contains("OPENAI_BASE_URL="));
        assert!(formatted.contains("OPENAI_API_KEY="));
        assert!(formatted.contains("ARGS="));
        assert!(!formatted.contains("--markdown-mode"));
        assert!(!formatted.contains("--model"));
        assert!(formatted.contains("--extract --format md"));
    }
}
