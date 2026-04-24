use anyhow::{Context, Result, bail};
use std::fs;

#[path = "chat_capability_eval/cli.rs"]
mod cli;
#[path = "chat_capability_eval/grading.rs"]
mod grading;
#[path = "chat_capability_eval/model.rs"]
mod model;
#[path = "chat_capability_eval/report.rs"]
mod report;
#[path = "chat_capability_eval/runner.rs"]
mod runner;
#[path = "chat_capability_eval/sse.rs"]
mod sse;

use cli::{ensure_backend_ready, filter_prompts, load_prompt_specs, parse_args};
use model::{SweepReport, SweepRunner};
use report::{build_summary, prompt_passed, write_reports};

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;
    let prompts = load_prompt_specs(&config.dataset_path)?;
    let filtered = filter_prompts(prompts, &config.class_filters, &config.prompt_id_filters);
    if filtered.is_empty() {
        bail!("no prompts matched the provided filters");
    }

    ensure_backend_ready(&config.base_url).await?;
    fs::create_dir_all(&config.output_dir)
        .with_context(|| format!("failed to create {}", config.output_dir.display()))?;

    let runner = SweepRunner::new(&config.base_url, config.timeout)?;
    let mut results = Vec::with_capacity(filtered.len());

    for (index, spec) in filtered.iter().enumerate() {
        println!(
            "[{}/{}] {} {}",
            index + 1,
            filtered.len(),
            spec.id,
            spec.prompt
        );
        let result = runner
            .run_prompt(spec, config.deep_research, config.model.as_deref())
            .await
            .with_context(|| format!("failed prompt {}", spec.id))?;
        let status_label = if prompt_passed(&result) {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "  -> {} score={} sources={} failure={}",
            status_label,
            result.rubric_capability_score,
            result.source_count,
            result.failure_class.as_deref().unwrap_or("-")
        );
        results.push(result);
    }

    let summary = build_summary(&results);
    let report = SweepReport {
        generated_at_utc: chrono::Utc::now().to_rfc3339(),
        base_url: config.base_url.clone(),
        dataset_path: config.dataset_path.display().to_string(),
        prompt_count: results.len(),
        summary,
        results,
    };

    write_reports(&config.output_dir, &report)?;
    println!("Wrote reports to {}", config.output_dir.as_path().display());

    Ok(())
}

#[cfg(test)]
#[path = "chat_capability_eval/tests.rs"]
mod tests;
