use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use dastill::services::{SummarizerService, build_http_client, is_cloud_model};
use libsql::{Connection, params};
use serde::Deserialize;
use tokio::time::timeout;

const DEFAULT_TITLE: &str = "AI images just got dangerously good";
const DEFAULT_PER_MODEL_TIMEOUT_SECS: u64 = 20 * 60;

#[derive(Debug)]
struct CliArgs {
    title: String,
    output_dir: Option<PathBuf>,
    per_model_timeout: Duration,
}

#[derive(Debug)]
struct TargetVideo {
    id: String,
    title: String,
    transcript_text: String,
    transcript_source: &'static str,
    watch_url: Option<String>,
    duration_seconds: Option<u64>,
    published_at: Option<String>,
    match_mode: &'static str,
    candidate_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaTaggedModel>,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaTaggedModel {
    name: String,
    remote_model: Option<String>,
    #[serde(default)]
    details: OllamaModelDetails,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    family: String,
    #[serde(default)]
    parameter_size: String,
}

#[derive(Debug)]
struct ComparisonOutcome {
    requested_model: String,
    model_used: String,
    status: &'static str,
    duration: Duration,
    summary: Option<String>,
    error: Option<String>,
    notes: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;
    load_env_if_present(Path::new(".env"))?;

    let db_url =
        env::var("DB_URL").context("DB_URL must be set in backend/.env or the environment")?;
    let db_pass = env::var("DB_PASS").unwrap_or_default();
    let ollama_url =
        env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let database = libsql::Builder::new_remote(db_url, db_pass)
        .build()
        .await
        .context("failed to build libsql remote connection")?;
    let conn = database.connect().context("failed to connect to libsql")?;

    let target = resolve_target_video(&conn, &args.title).await?;
    let models = discover_local_models(&ollama_url).await?;
    if models.is_empty() {
        bail!("no local Ollama models were discovered at {ollama_url}");
    }

    let output_dir = args
        .output_dir
        .unwrap_or_else(|| default_output_dir(&args.title).expect("default output dir"));
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let mut outcomes = Vec::with_capacity(models.len());
    for model in &models {
        let outcome = run_comparison(
            &ollama_url,
            model,
            &target.title,
            &target.transcript_text,
            args.per_model_timeout,
        )
        .await;
        let filename = per_model_filename(outcomes.len(), &model.name);
        write_model_report(&output_dir.join(&filename), &target, model, &outcome)?;
        outcomes.push(outcome);
    }

    write_index_report(
        &output_dir.join("README.md"),
        &target,
        &ollama_url,
        args.per_model_timeout,
        &models,
        &outcomes,
    )?;

    println!("comparison complete");
    println!("report_dir={}", output_dir.display());
    println!("video_id={}", target.id);
    println!("video_title={}", target.title);
    for outcome in &outcomes {
        println!(
            "{}\t{}\t{}ms",
            outcome.requested_model,
            outcome.status,
            outcome.duration.as_millis()
        );
    }

    Ok(())
}

fn parse_args() -> Result<CliArgs> {
    let mut title = DEFAULT_TITLE.to_string();
    let mut output_dir = None;
    let mut per_model_timeout = Duration::from_secs(DEFAULT_PER_MODEL_TIMEOUT_SECS);

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--title" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --title"))?;
                title = value;
            }
            "--output-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --output-dir"))?;
                output_dir = Some(PathBuf::from(value));
            }
            "--per-model-timeout-secs" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --per-model-timeout-secs"))?;
                let secs = value
                    .parse::<u64>()
                    .with_context(|| format!("invalid timeout seconds: {value}"))?;
                per_model_timeout = Duration::from_secs(secs);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    Ok(CliArgs {
        title,
        output_dir,
        per_model_timeout,
    })
}

fn print_usage() {
    println!("Compare production summaries across local Ollama models");
    println!();
    println!("Usage:");
    println!(
        "  cargo run --bin compare_ollama_summaries -- [--title <title>] [--output-dir <dir>] [--per-model-timeout-secs <secs>]"
    );
}

fn load_env_if_present(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            if env::var(key).is_ok() {
                continue;
            }
            // SAFETY: this runs during single-threaded startup before async work starts.
            unsafe { env::set_var(key, value.trim()) };
        }
    }

    Ok(())
}

async fn resolve_target_video(conn: &Connection, title: &str) -> Result<TargetVideo> {
    let exact = query_candidates(conn, title, false).await?;
    if let Some(target) = exact.first() {
        return Ok(TargetVideo {
            match_mode: "exact",
            candidate_count: exact.len(),
            ..target.clone()
        });
    }

    let partial = query_candidates(conn, title, true).await?;
    if let Some(target) = partial.first() {
        return Ok(TargetVideo {
            match_mode: "partial",
            candidate_count: partial.len(),
            ..target.clone()
        });
    }

    bail!("no existing transcript-ready video found for title {title:?}");
}

async fn query_candidates(
    conn: &Connection,
    title: &str,
    partial: bool,
) -> Result<Vec<TargetVideo>> {
    let sql = if partial {
        r#"
        SELECT
            v.id,
            v.title,
            t.raw_text,
            t.formatted_markdown,
            vi.watch_url,
            vi.duration_seconds,
            vi.published_at
        FROM videos v
        LEFT JOIN transcripts t ON t.video_id = v.id
        LEFT JOIN video_info vi ON vi.video_id = v.id
        WHERE v.transcript_status = 'ready'
          AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''
          AND (
                lower(v.title) LIKE '%' || lower(?1) || '%'
             OR lower(COALESCE(vi.title, '')) LIKE '%' || lower(?1) || '%'
          )
        ORDER BY COALESCE(vi.duration_seconds, 0) DESC, v.published_at DESC
        LIMIT 10
        "#
    } else {
        r#"
        SELECT
            v.id,
            v.title,
            t.raw_text,
            t.formatted_markdown,
            vi.watch_url,
            vi.duration_seconds,
            vi.published_at
        FROM videos v
        LEFT JOIN transcripts t ON t.video_id = v.id
        LEFT JOIN video_info vi ON vi.video_id = v.id
        WHERE v.transcript_status = 'ready'
          AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''
          AND (
                lower(v.title) = lower(?1)
             OR lower(COALESCE(vi.title, '')) = lower(?1)
          )
        ORDER BY COALESCE(vi.duration_seconds, 0) DESC, v.published_at DESC
        LIMIT 10
        "#
    };

    let mut rows = conn.query(sql, params![title]).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        let raw_text: Option<String> = row.get(2)?;
        let formatted_markdown: Option<String> = row.get(3)?;
        let (transcript_text, transcript_source) = select_transcript(raw_text, formatted_markdown)
            .ok_or_else(|| anyhow!("row returned without usable transcript"))?;
        let duration_seconds = row.get::<Option<i64>>(5)?.map(|value| value.max(0) as u64);

        results.push(TargetVideo {
            id: row.get(0)?,
            title: row.get(1)?,
            transcript_text,
            transcript_source,
            watch_url: row.get(4)?,
            duration_seconds,
            published_at: row.get(6)?,
            match_mode: if partial { "partial" } else { "exact" },
            candidate_count: 1,
        });
    }

    Ok(results)
}

fn select_transcript(
    raw_text: Option<String>,
    formatted_markdown: Option<String>,
) -> Option<(String, &'static str)> {
    if let Some(raw_text) = raw_text {
        let trimmed = raw_text.trim();
        if !trimmed.is_empty() {
            return Some((trimmed.to_string(), "raw_text"));
        }
    }
    if let Some(formatted_markdown) = formatted_markdown {
        let trimmed = formatted_markdown.trim();
        if !trimmed.is_empty() {
            return Some((trimmed.to_string(), "formatted_markdown"));
        }
    }
    None
}

async fn discover_local_models(ollama_url: &str) -> Result<Vec<OllamaTaggedModel>> {
    let response = build_http_client()
        .get(format!("{ollama_url}/api/tags"))
        .send()
        .await
        .with_context(|| format!("failed to query {ollama_url}/api/tags"))?
        .error_for_status()
        .with_context(|| format!("Ollama tag query failed at {ollama_url}/api/tags"))?;

    let mut models = response
        .json::<OllamaTagsResponse>()
        .await
        .context("failed to decode Ollama tag response")?
        .models
        .into_iter()
        .filter(is_local_candidate)
        .collect::<Vec<_>>();

    models.sort_by(|left, right| {
        let left_embedding = looks_like_embedding_model(left);
        let right_embedding = looks_like_embedding_model(right);
        left_embedding
            .cmp(&right_embedding)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(models)
}

fn is_local_candidate(model: &OllamaTaggedModel) -> bool {
    model.remote_model.is_none() && !is_cloud_model(&model.name)
}

fn looks_like_embedding_model(model: &OllamaTaggedModel) -> bool {
    model.name.contains("embed") || model.details.family.contains("bert")
}

async fn run_comparison(
    ollama_url: &str,
    model: &OllamaTaggedModel,
    title: &str,
    transcript_text: &str,
    per_model_timeout: Duration,
) -> ComparisonOutcome {
    let mut notes = Vec::new();
    if looks_like_embedding_model(model) {
        notes.push(
            "Model looks embedding-oriented from its name/family and may reject chat completion requests."
                .to_string(),
        );
    }
    if !model.details.parameter_size.is_empty() {
        notes.push(format!(
            "Advertised parameter size: {}.",
            model.details.parameter_size
        ));
    }

    let service = SummarizerService::with_config(ollama_url, &model.name);
    let started = Instant::now();
    let result = timeout(per_model_timeout, service.summarize(transcript_text, title)).await;
    let duration = started.elapsed();

    match result {
        Ok(Ok((summary, model_used))) => ComparisonOutcome {
            requested_model: model.name.clone(),
            model_used,
            status: "success",
            duration,
            summary: Some(summary),
            error: None,
            notes,
        },
        Ok(Err(err)) => ComparisonOutcome {
            requested_model: model.name.clone(),
            model_used: model.name.clone(),
            status: "error",
            duration,
            summary: None,
            error: Some(err.to_string()),
            notes,
        },
        Err(_) => ComparisonOutcome {
            requested_model: model.name.clone(),
            model_used: model.name.clone(),
            status: "timeout",
            duration,
            summary: None,
            error: Some(format!(
                "summary generation exceeded {} seconds",
                per_model_timeout.as_secs()
            )),
            notes,
        },
    }
}

fn default_output_dir(title: &str) -> Result<PathBuf> {
    let cwd = env::current_dir().context("failed to resolve current directory")?;
    let repo_root = if cwd.file_name().and_then(|value| value.to_str()) == Some("backend") {
        cwd.parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("backend directory has no parent"))?
    } else {
        cwd
    };
    Ok(repo_root
        .join("reports")
        .join(format!("ollama-summary-comparison-{}", slugify(title))))
}

fn write_index_report(
    path: &Path,
    target: &TargetVideo,
    ollama_url: &str,
    per_model_timeout: Duration,
    models: &[OllamaTaggedModel],
    outcomes: &[ComparisonOutcome],
) -> Result<()> {
    let transcript_word_count = target.transcript_text.split_whitespace().count();
    let transcript_char_count = target.transcript_text.chars().count();
    let cloud_models = models
        .iter()
        .filter(|model| is_cloud_model(&model.name))
        .map(|model| model.name.as_str())
        .collect::<Vec<_>>();

    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Ollama Summary Comparison - {}\n\n",
        target.title
    ));
    markdown.push_str("## Run Metadata\n");
    markdown.push_str(&format!("- Generated at: {}\n", Utc::now().to_rfc3339()));
    markdown.push_str("- Production summary flow: `backend/src/services/summarizer.rs::SummarizerService::summarize()`\n");
    markdown.push_str("- Transcript source precedence: `raw_text`, then `formatted_markdown` (`backend/src/handlers/content.rs`)\n");
    markdown.push_str(&format!("- Ollama URL: `{ollama_url}`\n"));
    markdown.push_str(&format!(
        "- Per-model timeout guard: {}\n",
        format_duration(per_model_timeout)
    ));
    markdown.push_str(&format!("- Video ID: `{}`\n", target.id));
    markdown.push_str(&format!("- Match mode: `{}`\n", target.match_mode));
    markdown.push_str(&format!(
        "- Candidate count for selected match: {}\n",
        target.candidate_count
    ));
    if let Some(url) = &target.watch_url {
        markdown.push_str(&format!("- Watch URL: <{url}>\n"));
    }
    if let Some(duration_seconds) = target.duration_seconds {
        markdown.push_str(&format!(
            "- Video duration: {} ({duration_seconds}s)\n",
            format_seconds(duration_seconds)
        ));
    }
    if let Some(published_at) = &target.published_at {
        markdown.push_str(&format!("- Published at: `{published_at}`\n"));
    }
    markdown.push_str(&format!(
        "- Transcript field used: `{}`\n",
        target.transcript_source
    ));
    markdown.push_str(&format!(
        "- Transcript size: {transcript_word_count} words / {transcript_char_count} chars\n\n"
    ));

    markdown.push_str("## Compared Local Models\n");
    for model in models {
        let family = if model.details.family.is_empty() {
            "unknown"
        } else {
            model.details.family.as_str()
        };
        let parameter_size = if model.details.parameter_size.is_empty() {
            "unknown"
        } else {
            model.details.parameter_size.as_str()
        };
        markdown.push_str(&format!(
            "- `{}` - family `{family}`, parameters `{parameter_size}`\n",
            model.name
        ));
    }
    if !cloud_models.is_empty() {
        markdown.push('\n');
        markdown.push_str("## Excluded Models\n");
        markdown.push_str("Cloud models were excluded from this run because the request was limited to local Ollama models.\n");
        for model in cloud_models {
            markdown.push_str(&format!("- `{model}`\n"));
        }
    }

    markdown.push_str("\n## Results\n");
    markdown.push_str("| Model | Status | Duration | Output | Notes | Document |\n");
    markdown.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for (index, outcome) in outcomes.iter().enumerate() {
        let output = match &outcome.summary {
            Some(summary) => format!("{} chars", summary.chars().count()),
            None => "n/a".to_string(),
        };
        let notes = if outcome.notes.is_empty() {
            "-".to_string()
        } else {
            escape_table_cell(&outcome.notes.join(" "))
        };
        let document = per_model_filename(index, &outcome.requested_model);
        markdown.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | [{}]({}) |\n",
            outcome.requested_model,
            outcome.status,
            format_duration(outcome.duration),
            output,
            notes,
            document,
            document
        ));
    }

    markdown.push_str("\n## Notes\n");
    markdown.push_str("- Each model was invoked sequentially to avoid local VRAM contention affecting the comparison.\n");
    markdown.push_str("- Failures and timeouts are preserved in the per-model documents instead of being retried with a different model.\n");
    markdown.push_str("- Successful runs include the exact summary text returned after the production heading-strip step.\n");

    fs::write(path, markdown).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_model_report(
    path: &Path,
    target: &TargetVideo,
    model: &OllamaTaggedModel,
    outcome: &ComparisonOutcome,
) -> Result<()> {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Model Result - {}\n\n", outcome.requested_model));
    markdown.push_str("## Metadata\n");
    markdown.push_str(&format!("- Generated at: {}\n", Utc::now().to_rfc3339()));
    markdown.push_str(&format!(
        "- Requested model: `{}`\n",
        outcome.requested_model
    ));
    markdown.push_str(&format!("- Model used: `{}`\n", outcome.model_used));
    markdown.push_str(&format!("- Status: `{}`\n", outcome.status));
    markdown.push_str(&format!(
        "- Duration: {}\n",
        format_duration(outcome.duration)
    ));
    markdown.push_str(&format!("- Video title: {}\n", target.title));
    markdown.push_str(&format!("- Video ID: `{}`\n", target.id));
    if let Some(url) = &target.watch_url {
        markdown.push_str(&format!("- Watch URL: <{url}>\n"));
    }
    if let Some(duration_seconds) = target.duration_seconds {
        markdown.push_str(&format!(
            "- Video duration: {} ({duration_seconds}s)\n",
            format_seconds(duration_seconds)
        ));
    }
    if !model.details.family.is_empty() {
        markdown.push_str(&format!("- Ollama family: `{}`\n", model.details.family));
    }
    if !model.details.parameter_size.is_empty() {
        markdown.push_str(&format!(
            "- Parameter size: `{}`\n",
            model.details.parameter_size
        ));
    }
    markdown.push_str(&format!(
        "- Transcript field used: `{}`\n",
        target.transcript_source
    ));
    markdown.push_str(&format!(
        "- Transcript size: {} words / {} chars\n",
        target.transcript_text.split_whitespace().count(),
        target.transcript_text.chars().count()
    ));
    for note in &outcome.notes {
        markdown.push_str(&format!("- Note: {note}\n"));
    }
    markdown.push('\n');

    match (&outcome.summary, &outcome.error) {
        (Some(summary), _) => {
            markdown.push_str("## Summary\n\n");
            markdown.push_str(summary);
            markdown.push('\n');
        }
        (_, Some(error)) => {
            markdown.push_str("## Error\n\n");
            markdown.push_str("```text\n");
            markdown.push_str(error);
            markdown.push_str("\n```\n");
        }
        _ => {
            markdown.push_str("## Error\n\n```text\nNo summary and no error were captured.\n```\n");
        }
    }

    fs::write(path, markdown).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn per_model_filename(index: usize, model: &str) -> String {
    format!("{:02}-{}.md", index + 1, slugify(model))
}

fn slugify(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut prev_dash = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else {
            None
        };
        match normalized {
            Some(ch) => {
                slug.push(ch);
                prev_dash = false;
            }
            None if !prev_dash => {
                slug.push('-');
                prev_dash = true;
            }
            None => {}
        }
    }
    slug.trim_matches('-').to_string()
}

fn format_duration(duration: Duration) -> String {
    format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis())
}

fn format_seconds(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

impl Clone for TargetVideo {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            title: self.title.clone(),
            transcript_text: self.transcript_text.clone(),
            transcript_source: self.transcript_source,
            watch_url: self.watch_url.clone(),
            duration_seconds: self.duration_seconds,
            published_at: self.published_at.clone(),
            match_mode: self.match_mode,
            candidate_count: self.candidate_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OllamaModelDetails, OllamaTaggedModel, is_local_candidate, select_transcript, slugify,
    };

    #[test]
    fn select_transcript_prefers_raw_text() {
        let result = select_transcript(
            Some(" raw transcript ".to_string()),
            Some("formatted transcript".to_string()),
        )
        .expect("transcript");
        assert_eq!(result.0, "raw transcript");
        assert_eq!(result.1, "raw_text");
    }

    #[test]
    fn select_transcript_falls_back_to_formatted_markdown() {
        let result = select_transcript(Some("   ".to_string()), Some(" formatted ".to_string()))
            .expect("transcript");
        assert_eq!(result.0, "formatted");
        assert_eq!(result.1, "formatted_markdown");
    }

    #[test]
    fn local_candidate_filter_excludes_remote_and_cloud_models() {
        let local = OllamaTaggedModel {
            name: "qwen3:8b".to_string(),
            remote_model: None,
            details: OllamaModelDetails::default(),
        };
        let remote = OllamaTaggedModel {
            name: "gemma3:27b-cloud".to_string(),
            remote_model: Some("gemma3:27b".to_string()),
            details: OllamaModelDetails::default(),
        };

        assert!(is_local_candidate(&local));
        assert!(!is_local_candidate(&remote));
    }

    #[test]
    fn slugify_normalizes_model_names() {
        assert_eq!(slugify("qwen3-coder:30b"), "qwen3-coder-30b");
        assert_eq!(slugify("qwen3.5:397b-cloud"), "qwen3-5-397b-cloud");
    }
}
