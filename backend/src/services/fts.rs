use std::sync::Arc;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, TextFieldIndexing, TextOptions, Value};
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument};
use tokio::sync::RwLock;

use crate::services::search::{SearchCandidate, SearchSourceKind};

const WRITER_HEAP_BYTES: usize = 15_000_000; // 15 MB

/// In-memory BM25 index over all indexed search chunks.
/// Thread-safe via an `Arc<RwLock<FtsIndexInner>>`.
#[derive(Clone)]
pub struct FtsIndex(Arc<RwLock<FtsIndexInner>>);

struct FtsIndexInner {
    index: Index,
    writer: IndexWriter,
    // Schema field handles.
    f_chunk_id: Field,
    f_video_id: Field,
    f_channel_id: Field,
    f_source_kind: Field,
    /// Composite deletion key: `{video_id}_{source_kind}` — enables per-source-kind deletion
    /// without affecting the other source kind for the same video.
    f_source_key: Field,
    f_section_title: Field,
    f_chunk_text: Field,
    f_video_title: Field,
    f_channel_name: Field,
    f_published_at: Field,
    f_start_sec: Field,
}

impl FtsIndex {
    pub fn new() -> Result<Self, tantivy::TantivyError> {
        let mut schema_builder = Schema::builder();

        // Stored-only keyword fields (used for result reconstruction).
        let stored_string = || -> TextOptions { TextOptions::default().set_stored() };
        // Full-text indexed + stored fields.
        let text_indexed = || -> TextOptions {
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("en_stem")
                        .set_index_option(
                            tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
                        ),
                )
                .set_stored()
        };

        // Keyword field: indexed as a single term (raw tokenizer) for exact-match deletion.
        let raw_keyword = || -> TextOptions {
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("raw")
                        .set_index_option(tantivy::schema::IndexRecordOption::Basic),
                )
                .set_stored()
        };

        let f_chunk_id = schema_builder.add_text_field("chunk_id", stored_string());
        let f_video_id = schema_builder.add_text_field("video_id", stored_string());
        let f_channel_id = schema_builder.add_text_field("channel_id", stored_string());
        let f_source_kind = schema_builder.add_text_field("source_kind", stored_string());
        // Composite deletion key: indexed as keyword for exact term-based deletion.
        let f_source_key = schema_builder.add_text_field("source_key", raw_keyword());
        let f_section_title = schema_builder.add_text_field("section_title", text_indexed());
        let f_chunk_text = schema_builder.add_text_field("chunk_text", text_indexed());
        let f_video_title = schema_builder.add_text_field("video_title", text_indexed());
        let f_channel_name = schema_builder.add_text_field("channel_name", stored_string());
        let f_published_at = schema_builder.add_text_field("published_at", stored_string());
        // start_sec stored as a string representation of f32 (Tantivy has no float stored field).
        let f_start_sec = schema_builder.add_text_field("start_sec", stored_string());

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let writer = index.writer(WRITER_HEAP_BYTES)?;

        Ok(Self(Arc::new(RwLock::new(FtsIndexInner {
            index,
            writer,
            f_chunk_id,
            f_video_id,
            f_channel_id,
            f_source_kind,
            f_source_key,
            f_section_title,
            f_chunk_text,
            f_video_title,
            f_channel_name,
            f_published_at,
            f_start_sec,
        }))))
    }

    /// Add or replace all chunks for a single video+source_kind pair.
    /// Deletes existing documents with the matching video_id + source_kind, then adds the new ones.
    pub async fn upsert_source(&self, meta: FtsSourceMeta<'_>, chunks: &[FtsChunk]) {
        let source_key = format!("{}_{}", meta.video_id, meta.source_kind.as_str());
        let mut inner = self.0.write().await;
        delete_source_docs(&mut inner, &source_key);

        for chunk in chunks {
            let mut doc = TantivyDocument::default();
            doc.add_text(inner.f_chunk_id, &chunk.chunk_id);
            doc.add_text(inner.f_video_id, meta.video_id);
            doc.add_text(inner.f_channel_id, meta.channel_id);
            doc.add_text(inner.f_source_kind, meta.source_kind.as_str());
            doc.add_text(inner.f_source_key, &source_key);
            doc.add_text(inner.f_chunk_text, &chunk.chunk_text);
            doc.add_text(inner.f_video_title, meta.video_title);
            doc.add_text(inner.f_channel_name, meta.channel_name);
            doc.add_text(inner.f_published_at, meta.published_at);
            if let Some(title) = &chunk.section_title {
                doc.add_text(inner.f_section_title, title);
            }
            if let Some(sec) = chunk.start_sec {
                doc.add_text(inner.f_start_sec, sec.to_string());
            }
            let _ = inner.writer.add_document(doc);
        }

        let _ = inner.writer.commit();
        let doc_count = inner
            .index
            .reader_builder()
            .try_into()
            .map(|r: tantivy::IndexReader| r.searcher().num_docs())
            .unwrap_or(0);
        tracing::info!(
            video_id = meta.video_id,
            source_kind = meta.source_kind.as_str(),
            chunks_added = chunks.len(),
            total_docs = doc_count,
            "fts index updated"
        );
    }

    /// Remove all indexed documents for a video+source_kind pair.
    pub async fn delete_source(&self, video_id: &str, source_kind: SearchSourceKind) {
        let source_key = format!("{}_{}", video_id, source_kind.as_str());
        let mut inner = self.0.write().await;
        delete_source_docs(&mut inner, &source_key);
        let _ = inner.writer.commit();
    }

    /// BM25 search. Returns candidates ranked by relevance score.
    /// Applies optional channel_id and source_kind filters as post-processing.
    pub async fn search(
        &self,
        query: &str,
        source_kind: Option<SearchSourceKind>,
        channel_id: Option<&str>,
        limit: usize,
    ) -> Vec<FtsSearchResult> {
        let inner = self.0.read().await;
        let reader = match inner
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
        {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &inner.index,
            vec![
                inner.f_chunk_text,
                inner.f_video_title,
                inner.f_section_title,
            ],
        );
        let parsed = match query_parser.parse_query(query) {
            Ok(q) => q,
            Err(_) => {
                // Fall back to a fuzzy term search on each word.
                let escaped: String = query
                    .split_whitespace()
                    .map(|w| format!("\"{}\"", w.replace('"', "")))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                match query_parser.parse_query(&escaped) {
                    Ok(q) => q,
                    Err(_) => return Vec::new(),
                }
            }
        };

        // Over-fetch when filtering to compensate for post-filter reduction.
        let fetch_limit = if source_kind.is_some() || channel_id.is_some() {
            (limit * 4).min(200)
        } else {
            limit.min(200)
        };

        let top_docs =
            match searcher.search(&parsed, &TopDocs::with_limit(fetch_limit).order_by_score()) {
                Ok(docs) => docs,
                Err(_) => return Vec::new(),
            };

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = match searcher.doc(doc_address) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let get_str = |field: Field| -> String {
                doc.get_first(field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };

            let doc_channel_id = get_str(inner.f_channel_id);
            let doc_source_kind_str = get_str(inner.f_source_kind);
            let doc_source_kind = SearchSourceKind::from_db_value(&doc_source_kind_str);

            // Apply filters.
            if let Some(cid) = channel_id {
                if doc_channel_id != cid {
                    continue;
                }
            }
            if let Some(sk) = source_kind {
                if doc_source_kind != sk {
                    continue;
                }
            }

            let start_sec = get_str(inner.f_start_sec)
                .parse::<f32>()
                .ok()
                .filter(|_| !get_str(inner.f_start_sec).is_empty());

            results.push(FtsSearchResult {
                chunk_id: get_str(inner.f_chunk_id),
                video_id: get_str(inner.f_video_id),
                channel_id: doc_channel_id,
                channel_name: get_str(inner.f_channel_name),
                video_title: get_str(inner.f_video_title),
                source_kind: doc_source_kind,
                section_title: {
                    let s = get_str(inner.f_section_title);
                    if s.is_empty() { None } else { Some(s) }
                },
                chunk_text: get_str(inner.f_chunk_text),
                published_at: get_str(inner.f_published_at),
                start_sec,
                score,
            });

            if results.len() >= limit {
                break;
            }
        }

        results
    }

    /// Total number of documents in the index (approximate).
    pub async fn doc_count(&self) -> u64 {
        let inner = self.0.read().await;
        let Ok(reader) = inner
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
        else {
            return 0;
        };
        reader.searcher().num_docs()
    }
}

fn delete_source_docs(inner: &mut FtsIndexInner, source_key: &str) {
    use tantivy::Term;
    // Delete all docs where source_key == "{video_id}_{source_kind}".
    // This is a precise per-source deletion that does not disturb the other source kind.
    let term = Term::from_field_text(inner.f_source_key, source_key);
    inner.writer.delete_term(term);
}

/// Data for a single chunk to be inserted into the FTS index.
#[derive(Debug, Clone)]
pub struct FtsChunk {
    pub chunk_id: String,
    pub section_title: Option<String>,
    pub chunk_text: String,
    pub start_sec: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct FtsSourceMeta<'a> {
    pub video_id: &'a str,
    pub source_kind: SearchSourceKind,
    pub channel_id: &'a str,
    pub channel_name: &'a str,
    pub video_title: &'a str,
    pub published_at: &'a str,
}

/// A single BM25 search result.
pub struct FtsSearchResult {
    pub chunk_id: String,
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub source_kind: SearchSourceKind,
    pub section_title: Option<String>,
    pub chunk_text: String,
    pub published_at: String,
    pub start_sec: Option<f32>,
    pub score: f32,
}

impl From<FtsSearchResult> for SearchCandidate {
    fn from(r: FtsSearchResult) -> Self {
        Self {
            chunk_id: r.chunk_id,
            video_id: r.video_id,
            channel_id: r.channel_id,
            channel_name: r.channel_name,
            video_title: r.video_title,
            source_kind: r.source_kind,
            section_title: r.section_title,
            chunk_text: r.chunk_text,
            published_at: r.published_at,
            start_sec: r.start_sec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fts_index_returns_bm25_ranked_results() {
        let index = FtsIndex::new().expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "video-1",
                    source_kind: SearchSourceKind::Transcript,
                    channel_id: "channel-a",
                    channel_name: "Channel A",
                    video_title: "Rust ownership and borrowing",
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[
                    FtsChunk {
                        chunk_id: "video-1_transcript_1_0".to_string(),
                        section_title: None,
                        chunk_text: "Ownership in Rust prevents dangling pointers at compile time."
                            .to_string(),
                        start_sec: Some(0.0),
                    },
                    FtsChunk {
                        chunk_id: "video-1_transcript_1_1".to_string(),
                        section_title: None,
                        chunk_text: "Borrowing rules enforce safe concurrent access to data."
                            .to_string(),
                        start_sec: Some(30.0),
                    },
                ],
            )
            .await;

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "video-2",
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-b",
                    channel_name: "Channel B",
                    video_title: "Python async patterns",
                    published_at: "2026-01-02T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "video-2_summary_1_0".to_string(),
                    section_title: Some("Overview".to_string()),
                    chunk_text: "Async programming in Python using asyncio coroutines.".to_string(),
                    start_sec: None,
                }],
            )
            .await;

        let results = index.search("ownership rust", None, None, 10).await;
        assert!(
            !results.is_empty(),
            "should return results for 'ownership rust'"
        );
        assert_eq!(results[0].video_id, "video-1");
        assert_eq!(results[0].start_sec, Some(0.0));
    }

    #[tokio::test]
    async fn fts_index_filters_by_channel_id() {
        let index = FtsIndex::new().expect("index should be created");

        for (vid, cid) in [("v1", "ch-a"), ("v2", "ch-b")] {
            index
                .upsert_source(
                    FtsSourceMeta {
                        video_id: vid,
                        source_kind: SearchSourceKind::Transcript,
                        channel_id: cid,
                        channel_name: cid,
                        video_title: "semantic search",
                        published_at: "2026-01-01T00:00:00Z",
                    },
                    &[FtsChunk {
                        chunk_id: format!("{vid}_transcript_1_0"),
                        section_title: None,
                        chunk_text: "semantic search with vector embeddings".to_string(),
                        start_sec: None,
                    }],
                )
                .await;
        }

        let results = index
            .search("semantic search", None, Some("ch-a"), 10)
            .await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].video_id, "v1");
    }

    #[tokio::test]
    async fn fts_index_delete_source_removes_documents() {
        let index = FtsIndex::new().expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "video-del",
                    source_kind: SearchSourceKind::Transcript,
                    channel_id: "ch",
                    channel_name: "Ch",
                    video_title: "deletion test",
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "video-del_transcript_1_0".to_string(),
                    section_title: None,
                    chunk_text: "this document should be deleted later".to_string(),
                    start_sec: None,
                }],
            )
            .await;

        let before = index.search("deleted", None, None, 10).await;
        assert!(!before.is_empty());

        index
            .delete_source("video-del", SearchSourceKind::Transcript)
            .await;

        let after = index.search("deleted", None, None, 10).await;
        assert!(
            after.is_empty(),
            "deleted source should not appear in results"
        );
    }
}
