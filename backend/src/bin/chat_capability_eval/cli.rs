use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use crate::model::{CapabilityClass, CliConfig, DEFAULT_BASE_URL, PromptSpec};

pub(crate) fn load_prompt_specs(path: &Path) -> Result<Vec<PromptSpec>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str::<Vec<PromptSpec>>(&content)
        .with_context(|| format!("failed to decode {}", path.display()))
}

pub(crate) fn filter_prompts(
    prompts: Vec<PromptSpec>,
    class_filters: &HashSet<CapabilityClass>,
    prompt_id_filters: &HashSet<String>,
) -> Vec<PromptSpec> {
    prompts
        .into_iter()
        .filter(|prompt| {
            (class_filters.is_empty() || class_filters.contains(&prompt.capability_class))
                && (prompt_id_filters.is_empty() || prompt_id_filters.contains(&prompt.id))
        })
        .collect()
}

pub(crate) async fn ensure_backend_ready(base_url: &str) -> Result<()> {
    let url = format!("{}/api/health", base_url.trim_end_matches('/'));
    let status = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build health-check client")?
        .get(url)
        .send()
        .await
        .context("failed to reach backend health endpoint")?;
    if !status.status().is_success() {
        bail!("backend health check failed with {}", status.status());
    }
    Ok(())
}

fn default_dataset_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("chat_capability_prompts.json")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".artifacts")
        .join("chat-capability")
}

pub(crate) fn parse_args() -> Result<CliConfig> {
    let mut config = CliConfig {
        base_url: DEFAULT_BASE_URL.to_string(),
        dataset_path: default_dataset_path(),
        output_dir: default_output_dir(),
        timeout: Duration::from_secs(240),
        deep_research: false,
        model: None,
        class_filters: HashSet::new(),
        prompt_id_filters: HashSet::new(),
    };

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base-url" => {
                config.base_url = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --base-url"))?;
            }
            "--dataset" => {
                config.dataset_path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --dataset"))?,
                );
            }
            "--output-dir" => {
                config.output_dir = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --output-dir"))?,
                );
            }
            "--timeout-seconds" => {
                let seconds = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --timeout-seconds"))?
                    .parse::<u64>()
                    .context("invalid timeout seconds")?;
                config.timeout = Duration::from_secs(seconds.max(1));
            }
            "--class" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --class"))?;
                config.class_filters.insert(parse_capability_class(&value)?);
            }
            "--prompt-id" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --prompt-id"))?;
                config.prompt_id_filters.insert(value);
            }
            "--model" => {
                config.model = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --model"))?,
                );
            }
            "--deep-research" => {
                config.deep_research = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown argument `{other}`"),
        }
    }

    Ok(config)
}

fn print_help() {
    println!(
        "chat_capability_eval

Options:
  --base-url <url>          Backend base URL. Default: {DEFAULT_BASE_URL}
  --dataset <path>          Prompt dataset path.
  --output-dir <path>       Output directory for reports.
  --timeout-seconds <n>     Per-request timeout. Default: 240
  --class <name>            Filter by capability class. Repeatable.
  --prompt-id <id>          Filter by prompt id. Repeatable.
  --model <id>              Optional chat model id to send with the prompt.
  --deep-research           Set deep_research=true for every prompt.
  --help                    Show this help.
"
    );
}

fn parse_capability_class(value: &str) -> Result<CapabilityClass> {
    match value.trim() {
        "direct_lookup" => Ok(CapabilityClass::DirectLookup),
        "topic_aggregation" => Ok(CapabilityClass::TopicAggregation),
        "cross_video_synthesis" => Ok(CapabilityClass::CrossVideoSynthesis),
        "comparison" => Ok(CapabilityClass::Comparison),
        "recommendation" => Ok(CapabilityClass::Recommendation),
        "creator_stance" => Ok(CapabilityClass::CreatorStance),
        "highlight_lookup" => Ok(CapabilityClass::HighlightLookup),
        "highlight_clustering" => Ok(CapabilityClass::HighlightClustering),
        "transcript_summary_alignment" => Ok(CapabilityClass::TranscriptSummaryAlignment),
        "timestamp_navigation" => Ok(CapabilityClass::TimestampNavigation),
        "tone_or_style_inference" => Ok(CapabilityClass::ToneOrStyleInference),
        "meta_learning_or_next_step" => Ok(CapabilityClass::MetaLearningOrNextStep),
        other => bail!("unknown capability class `{other}`"),
    }
}
