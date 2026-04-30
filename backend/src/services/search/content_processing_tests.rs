use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use serde::Deserialize;
use serde_json::json;

use super::{
    SearchService, SearchSourceKind, build_embedding_input, chunk_summary_content,
    chunk_transcript_content, extract_keyword_snippet, fuse_ranked_matches, hash_search_content,
    truncate_chunk_for_display,
};
use crate::services::search::{SEARCH_SUMMARY_MAX_CHUNKS, SEARCH_TRANSCRIPT_MAX_CHUNKS};

#[derive(Debug, Deserialize)]
struct TestEmbedRequest {
    input: Vec<String>,
    dimensions: Option<usize>,
}

async fn spawn_embed_test_server(
    max_inputs_per_request: usize,
) -> (String, Arc<AtomicUsize>, tokio::sync::oneshot::Sender<()>) {
    let request_count = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind embed test server");
    let address = listener.local_addr().expect("embed test server address");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let app = Router::new().route(
        "/api/embed",
        post({
            let request_count = request_count.clone();
            move |Json(payload): Json<TestEmbedRequest>| {
                let request_count = request_count.clone();
                async move {
                    let request_number = request_count.fetch_add(1, Ordering::SeqCst) + 1;
                    if payload.input.len() > max_inputs_per_request {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            Json(json!({ "error": "too many inputs" })),
                        )
                            .into_response();
                    }

                    let dimensions = payload.dimensions.unwrap_or(2);
                    let embeddings = payload
                        .input
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            let mut embedding = vec![0.0; dimensions];
                            if dimensions > 0 {
                                embedding[0] = request_number as f32;
                            }
                            if dimensions > 1 {
                                embedding[1] = index as f32;
                            }
                            embedding
                        })
                        .collect::<Vec<_>>();

                    (StatusCode::OK, Json(json!({ "embeddings": embeddings }))).into_response()
                }
            }
        }),
    );

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("run embed test server");
    });

    (format!("http://{address}"), request_count, shutdown_tx)
}

#[test]
fn hash_search_content_changes_when_text_changes() {
    assert_ne!(
        hash_search_content("alpha beta"),
        hash_search_content("alpha gamma")
    );
}

#[test]
fn chunk_summary_content_keeps_full_document_and_heading_sections() {
    let chunks = chunk_summary_content(
        "# Summary\n\n## Overview\nRust ownership basics and borrowing.\n\n## Tooling\nCargo workflows, tests, and release builds.",
        20,
    );

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].source_kind, SearchSourceKind::Summary);
    assert!(chunks[0].is_full_document);
    assert_eq!(chunks[1].section_title.as_deref(), Some("Overview"));
    assert_eq!(chunks[2].section_title.as_deref(), Some("Tooling"));
}

#[test]
fn chunk_summary_content_keeps_full_summary_chunk_untruncated() {
    let long_body = std::iter::repeat_n("alpha beta gamma delta epsilon", 40)
        .collect::<Vec<_>>()
        .join(" ");
    let summary = format!("# Summary\n\n## Overview\n{long_body}");

    let chunks = chunk_summary_content(&summary, 20);

    assert!(chunks[0].is_full_document);
    assert_eq!(chunks[0].text, format!("Summary Overview {long_body}"));
    assert!(chunks[0].text.len() > super::MAX_SNIPPET_CHARS);
}

#[test]
fn chunk_summary_content_caps_total_chunk_count() {
    let sections = (0..120)
        .map(|index| format!("## Section {index}\nalpha beta gamma delta epsilon"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let summary = format!("# Summary\n\n{sections}");

    let chunks = chunk_summary_content(&summary, 5);

    assert_eq!(chunks.len(), SEARCH_SUMMARY_MAX_CHUNKS);
    assert!(chunks[0].is_full_document);
}

#[test]
fn chunk_transcript_content_splits_long_paragraphs_with_overlap() {
    let transcript = [
        "Paragraph one introduces semantic search and vector indexes with practical examples.",
        "Paragraph two explains why keyword retrieval still matters for exact model names and acronyms.",
        "Paragraph three covers chunking tradeoffs and overlap decisions for transcript search.",
        "Paragraph four closes with deployment implications for local-only Ollama indexing.",
    ]
    .join("\n\n");

    let chunks = chunk_transcript_content(&transcript, 12, 4, None);

    assert!(chunks.len() >= 2);
    assert_eq!(chunks[0].source_kind, SearchSourceKind::Transcript);
    assert!(!chunks[0].is_full_document);
    assert!(
        chunks
            .iter()
            .any(|chunk| chunk.text.contains("matters for exact model names"))
    );
}

#[test]
fn chunk_transcript_content_respects_explicit_paragraph_breaks() {
    let transcript = "Alpha beta gamma delta.\n\nSecond paragraph starts here today.";

    let chunks = chunk_transcript_content(transcript, 5, 0, None);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].text, "Alpha beta gamma delta.");
    assert_eq!(chunks[1].text, "Second paragraph starts here today.");
}

#[test]
fn chunk_transcript_content_caps_total_chunk_count_for_long_inputs() {
    let transcript = std::iter::repeat_n("alpha beta gamma delta epsilon", 2_000)
        .collect::<Vec<_>>()
        .join(" ");

    let chunks = chunk_transcript_content(&transcript, 20, 4, None);

    assert!(
        chunks.len() <= SEARCH_TRANSCRIPT_MAX_CHUNKS,
        "expected at most {} chunks, got {}",
        SEARCH_TRANSCRIPT_MAX_CHUNKS,
        chunks.len()
    );
}

#[test]
fn extract_keyword_snippet_centers_the_matching_region_in_long_text() {
    let prefix = "alpha ".repeat(120);
    let suffix = "omega ".repeat(120);
    let text = format!("{prefix}semantic match appears here{suffix}");

    let snippet = extract_keyword_snippet(&text, &["semantic".to_string()]);

    assert!(snippet.contains("semantic match appears here"));
    assert!(snippet.starts_with("..."));
    assert!(snippet.ends_with("..."));
}

#[test]
fn truncate_chunk_for_display_normalizes_markdown_noise() {
    let text = "# Heading\n\n- First point\n- Second point";

    assert_eq!(
        truncate_chunk_for_display(text),
        "Heading First point Second point"
    );
}

#[test]
fn build_embedding_input_includes_search_metadata() {
    let input = build_embedding_input(
        "Video title",
        "Channel name",
        SearchSourceKind::Summary,
        Some("Overview"),
        "Key summary text",
    );

    assert!(input.contains("Video: Video title"));
    assert!(input.contains("Channel: Channel name"));
    assert!(input.contains("Source: summary"));
    assert!(input.contains("Section: Overview"));
    assert!(input.ends_with("Key summary text"));
}

#[test]
fn fuse_ranked_matches_rewards_items_seen_by_both_retrievers() {
    let fused = fuse_ranked_matches(
        &[("chunk-a", 1), ("chunk-b", 2)],
        &[("chunk-b", 1), ("chunk-c", 2)],
        60.0,
    );

    assert_eq!(fused[0].0, "chunk-b");
    assert!(fused[0].1 > fused[1].1);
}

#[tokio::test]
async fn embed_texts_splits_large_requests_into_multiple_batches() {
    let (base_url, request_count, shutdown_tx) = spawn_embed_test_server(8).await;
    let service = SearchService::with_config(&base_url, Some("embeddinggemma:latest"), 2, true);

    let inputs = (0..9)
        .map(|index| format!("chunk {index}"))
        .collect::<Vec<_>>();
    let embeddings = service
        .embed_texts(&inputs)
        .await
        .expect("batched embeddings");

    assert_eq!(request_count.load(Ordering::SeqCst), 2);
    assert_eq!(embeddings.len(), 9);
    assert_eq!(embeddings[0], vec![1.0, 0.0]);
    assert_eq!(embeddings[7], vec![1.0, 7.0]);
    assert_eq!(embeddings[8], vec![2.0, 0.0]);

    let _ = shutdown_tx.send(());
}
