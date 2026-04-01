use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::model::{
    CapabilityClass, CapabilitySummary, FAILURE_SINGLE_VIDEO, PromptRunResult, SweepReport,
    SweepSummary,
};

pub(crate) fn build_summary(results: &[PromptRunResult]) -> SweepSummary {
    let total_prompts = results.len();
    let passed_prompts = results
        .iter()
        .filter(|result| prompt_passed(result))
        .count();
    let answerability_passed = results
        .iter()
        .filter(|result| result.rubric_answerability_pass)
        .count();
    let grounding_passed = results
        .iter()
        .filter(|result| result.rubric_grounding_pass)
        .count();
    let shape_passed = results
        .iter()
        .filter(|result| result.rubric_shape_pass)
        .count();
    let average_score = if total_prompts == 0 {
        0.0
    } else {
        results
            .iter()
            .map(|result| result.rubric_capability_score as f32)
            .sum::<f32>()
            / total_prompts as f32
    };

    let prompts_without_sources = results
        .iter()
        .filter(|result| result.source_count == 0)
        .map(|result| result.prompt_id.clone())
        .collect();

    let single_video_prompts = results
        .iter()
        .filter(|result| {
            result.failure_class.as_deref() == Some(FAILURE_SINGLE_VIDEO)
                || (result.capability_class != CapabilityClass::DirectLookup
                    && result.source_videos.len() == 1)
        })
        .map(|result| result.prompt_id.clone())
        .collect();

    let mut failure_counts = BTreeMap::<String, usize>::new();
    for result in results {
        if let Some(failure) = &result.failure_class {
            *failure_counts.entry(failure.clone()).or_insert(0) += 1;
        }
    }

    let mut grouped = HashMap::<CapabilityClass, Vec<&PromptRunResult>>::new();
    for result in results {
        grouped
            .entry(result.capability_class)
            .or_default()
            .push(result);
    }

    let mut by_capability_class = grouped
        .into_iter()
        .map(|(capability_class, grouped_results)| {
            let total = grouped_results.len();
            let passed = grouped_results
                .iter()
                .filter(|result| prompt_passed(result))
                .count();
            let answerability_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_answerability_pass)
                .count();
            let grounding_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_grounding_pass)
                .count();
            let shape_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_shape_pass)
                .count();
            let average_score = grouped_results
                .iter()
                .map(|result| result.rubric_capability_score as f32)
                .sum::<f32>()
                / total as f32;
            let mut common_failure_counts = BTreeMap::<String, usize>::new();
            for result in grouped_results {
                if let Some(failure) = &result.failure_class {
                    *common_failure_counts.entry(failure.clone()).or_insert(0) += 1;
                }
            }
            CapabilitySummary {
                capability_class,
                total,
                passed,
                answerability_passed,
                grounding_passed,
                shape_passed,
                average_score,
                common_failure_classes: common_failure_counts.keys().cloned().collect(),
            }
        })
        .collect::<Vec<_>>();
    by_capability_class.sort_by_key(|summary| summary.capability_class);

    SweepSummary {
        total_prompts,
        passed_prompts,
        answerability_passed,
        grounding_passed,
        shape_passed,
        average_score,
        prompts_without_sources,
        single_video_prompts,
        failure_counts,
        by_capability_class,
    }
}

pub(crate) fn prompt_passed(result: &PromptRunResult) -> bool {
    result.rubric_answerability_pass
        && result.rubric_grounding_pass
        && result.rubric_shape_pass
        && result.rubric_capability_score >= 2
}

pub(crate) fn write_reports(output_dir: &Path, report: &SweepReport) -> Result<()> {
    let results_json = output_dir.join("results.json");
    let results_md = output_dir.join("results.md");
    let failures_json = output_dir.join("failures-by-class.json");

    fs::write(
        &results_json,
        serde_json::to_vec_pretty(report).context("failed to encode report JSON")?,
    )
    .with_context(|| format!("failed to write {}", results_json.display()))?;

    let failure_map = grouped_failures(&report.results);
    fs::write(
        &failures_json,
        serde_json::to_vec_pretty(&failure_map).context("failed to encode failure JSON")?,
    )
    .with_context(|| format!("failed to write {}", failures_json.display()))?;

    fs::write(&results_md, render_markdown_report(report))
        .with_context(|| format!("failed to write {}", results_md.display()))?;

    Ok(())
}

fn grouped_failures(results: &[PromptRunResult]) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for result in results {
        if let Some(failure) = &result.failure_class {
            grouped
                .entry(failure.clone())
                .or_default()
                .push(result.prompt_id.clone());
        }
    }
    grouped
}

fn render_markdown_report(report: &SweepReport) -> String {
    let mut md = String::new();
    md.push_str("# Chat Capability Sweep Results\n\n");
    md.push_str(&format!(
        "- Generated: `{}`\n- Base URL: `{}`\n- Dataset: `{}`\n- Prompt count: `{}`\n\n",
        report.generated_at_utc, report.base_url, report.dataset_path, report.prompt_count
    ));

    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "- Passed prompts: `{}/{}`
- Answerability pass: `{}/{}`
- Grounding pass: `{}/{}`
- Shape pass: `{}/{}`
- Average score: `{:.2}`\n\n",
        report.summary.passed_prompts,
        report.summary.total_prompts,
        report.summary.answerability_passed,
        report.summary.total_prompts,
        report.summary.grounding_passed,
        report.summary.total_prompts,
        report.summary.shape_passed,
        report.summary.total_prompts,
        report.summary.average_score
    ));

    md.push_str("## Capability Classes\n\n");
    for summary in &report.summary.by_capability_class {
        md.push_str(&format!(
            "- `{}`: passed `{}/{}`, avg score `{:.2}`, failures `{}`\n",
            capability_class_name(summary.capability_class),
            summary.passed,
            summary.total,
            summary.average_score,
            if summary.common_failure_classes.is_empty() {
                "-".to_string()
            } else {
                summary.common_failure_classes.join(", ")
            }
        ));
    }
    md.push('\n');

    md.push_str("## Failures By Class\n\n");
    for (failure, ids) in grouped_failures(&report.results) {
        md.push_str(&format!("- `{failure}`: {}\n", ids.join(", ")));
    }
    md.push('\n');

    md.push_str("## Prompt Results\n\n");
    for result in &report.results {
        md.push_str(&format!(
            "### {} {}\n\n",
            result.prompt_id,
            if prompt_passed(result) {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        md.push_str(&format!(
            "- Prompt: {}\n- Class: `{}`\n- Status: `{:?}`\n- Score: `{}`\n- Sources: `{}`\n- Failure: `{}`\n",
            result.prompt,
            capability_class_name(result.capability_class),
            result.status,
            result.rubric_capability_score,
            result.source_count,
            result.failure_class.as_deref().unwrap_or("-")
        ));
        if !result.source_videos.is_empty() {
            md.push_str(&format!(
                "- Source videos: {}\n",
                result.source_videos.join(" | ")
            ));
        }
        if !result.tool_calls.is_empty() {
            let tools = result
                .tool_calls
                .iter()
                .map(|tool| format!("{} ({})", tool.label, tool.name))
                .collect::<Vec<_>>()
                .join(", ");
            md.push_str(&format!("- Tools: {tools}\n"));
        }
        if !result.notes.is_empty() {
            md.push_str(&format!("- Notes: {}\n", result.notes.join(" | ")));
        }
        md.push_str("\n#### Answer\n\n");
        if result.assistant_content.trim().is_empty() {
            md.push_str("_No assistant content._\n\n");
        } else {
            md.push_str(&result.assistant_content);
            md.push_str("\n\n");
        }
    }

    md
}

fn capability_class_name(class_name: CapabilityClass) -> &'static str {
    match class_name {
        CapabilityClass::DirectLookup => "direct_lookup",
        CapabilityClass::TopicAggregation => "topic_aggregation",
        CapabilityClass::CrossVideoSynthesis => "cross_video_synthesis",
        CapabilityClass::Comparison => "comparison",
        CapabilityClass::Recommendation => "recommendation",
        CapabilityClass::CreatorStance => "creator_stance",
        CapabilityClass::HighlightLookup => "highlight_lookup",
        CapabilityClass::HighlightClustering => "highlight_clustering",
        CapabilityClass::TranscriptSummaryAlignment => "transcript_summary_alignment",
        CapabilityClass::TimestampNavigation => "timestamp_navigation",
        CapabilityClass::ToneOrStyleInference => "tone_or_style_inference",
        CapabilityClass::MetaLearningOrNextStep => "meta_learning_or_next_step",
    }
}
