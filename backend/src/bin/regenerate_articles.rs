//! Prepare reviewed article replacements without writing to shared storage.
//! cargo run --bin regenerate_articles -- input.json output-directory
use std::path::Path;

use dastill::{
    config::OllamaRuntimeConfig,
    models::{Summary, Transcript, TranscriptRenderMode, VocabularyReplacement},
    services::{OllamaCore, SummarizerService, SummaryEvaluatorService, TranscriptService},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ArticleRepair {
    video_id: String,
    title: String,
    transcript: Transcript,
    #[serde(default)]
    refetch_captions: bool,
    #[serde(default)]
    vocabulary_replacements: Vec<VocabularyReplacement>,
}

#[derive(Serialize)]
struct ArticleReplacement {
    transcript: Transcript,
    summary: Summary,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dastill::startup::install_crypto_providers();
    dastill::local_env::load_envs();
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(
        args.len() == 3,
        "usage: regenerate_articles input.json output-directory"
    );
    let repairs: Vec<ArticleRepair> = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let output = Path::new(&args[2]);
    std::fs::create_dir_all(output)?;
    let config = OllamaRuntimeConfig::from_env(false).map_err(anyhow::Error::msg)?;
    let summarizer = SummarizerService::new(
        OllamaCore::new(&config.url, &config.summary_model).with_api_key(config.api_key.clone()),
    );
    let evaluator = SummaryEvaluatorService::new(
        OllamaCore::new(&config.url, &config.summary_evaluator_model)
            .with_api_key(config.api_key.clone()),
    );
    let extractor = TranscriptService::with_paths(
        &std::env::var("SUMMARIZE_PATH").unwrap_or_else(|_| "/opt/homebrew/bin/summarize".into()),
        &std::env::var("YTDLP_PATH").unwrap_or_else(|_| "/opt/homebrew/bin/yt-dlp".into()),
    );
    let mut failed = 0;
    for repair in repairs {
        let destination = output.join(format!("{}.json", repair.video_id));
        anyhow::ensure!(!repair.video_id.contains(['/', '\\']), "invalid video id");
        if destination.exists() {
            eprintln!("{}: replacement already prepared", repair.video_id);
            continue;
        }
        match prepare_replacement(&repair, &extractor, &summarizer, &evaluator).await {
            Ok(replacement) => {
                std::fs::write(destination, serde_json::to_vec_pretty(&replacement)?)?;
                eprintln!(
                    "{}: prepared, score {:?}",
                    repair.video_id, replacement.summary.quality_score
                );
            }
            Err(error) => {
                failed += 1;
                eprintln!("{}: unchanged: {error}", repair.video_id);
            }
        }
    }
    anyhow::ensure!(
        failed == 0,
        "{failed} articles need further review; successful replacements were saved locally"
    );
    Ok(())
}

async fn prepare_replacement(
    repair: &ArticleRepair,
    extractor: &TranscriptService,
    summarizer: &SummarizerService,
    evaluator: &SummaryEvaluatorService,
) -> anyhow::Result<ArticleReplacement> {
    let transcript = if repair.refetch_captions {
        let (raw, formatted, timed) = extractor.extract_youtube_captions(&repair.video_id).await?;
        Transcript {
            video_id: repair.video_id.clone(),
            raw_text: Some(raw),
            formatted_markdown: Some(formatted),
            render_mode: TranscriptRenderMode::PlainText,
            timed_text: Some(timed),
        }
    } else {
        repair.transcript.clone()
    };
    let text = transcript
        .raw_text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .or(transcript.formatted_markdown.as_deref())
        .unwrap_or("");
    anyhow::ensure!(!text.trim().is_empty(), "missing transcript");
    let normalized = dastill::services::summarizer::apply_vocabulary_replacements(
        text,
        &repair.vocabulary_replacements,
    );
    let (content, model) = summarizer
        .summarize(
            &normalized,
            &repair.title,
            &repair.video_id,
            "repair",
            &repair.vocabulary_replacements,
        )
        .await?;
    let evaluation = evaluator
        .evaluate(&normalized, &content, &repair.title)
        .await?;
    anyhow::ensure!(
        evaluation.quality_score.is_some_and(|score| score >= 7),
        "replacement did not pass evaluation: {:?}",
        evaluation.quality_note
    );
    Ok(ArticleReplacement {
        transcript,
        summary: Summary {
            video_id: repair.video_id.clone(),
            content,
            model_used: Some(model),
            quality_score: evaluation.quality_score,
            quality_note: evaluation.quality_note,
            quality_model_used: evaluation.quality_model_used,
            summary_tags: evaluation.summary_tags,
            summary_tags_evaluated: true,
        },
    })
}
