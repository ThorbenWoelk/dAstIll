use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use libsql::{Builder, Connection, Database, TransactionBehavior, Value, params};
use tokio::sync::RwLock;

use crate::{
    search_query::{build_fts_phrase_queries, build_fts_query, build_fts_relaxed_query},
    services::search::{SearchCandidate, SearchSourceKind},
};

const LOCAL_DB_FILENAME: &str = "search-fts.db";
static FTS_TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Local libSQL-backed BM25 index over all indexed search chunks.
/// Thread-safe via an `Arc<RwLock<FtsIndexInner>>`.
#[derive(Clone)]
pub struct FtsIndex(Arc<RwLock<FtsIndexInner>>);

struct FtsIndexInner {
    _db: Database,
    conn: Connection,
    db_path: Option<PathBuf>,
}

impl FtsIndex {
    pub async fn new() -> Result<Self, String> {
        let counter = FTS_TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temp_dir = std::env::temp_dir().join(format!(
            "dastill-fts-{}-{unique_suffix}-{counter}",
            std::process::id(),
        ));
        Self::new_in_dir(temp_dir).await
    }

    pub async fn new_in_dir(index_dir: impl Into<PathBuf>) -> Result<Self, String> {
        let index_dir = index_dir.into();
        fs::create_dir_all(&index_dir)
            .map_err(|err| format!("failed to create FTS directory: {err}"))?;
        let db_path = index_dir.join(LOCAL_DB_FILENAME);

        let db = build_database(&db_path).await?;
        let conn = db
            .connect()
            .map_err(|err| format!("failed to connect to FTS database: {err}"))?;
        initialize_schema(&conn).await?;

        Ok(Self(Arc::new(RwLock::new(FtsIndexInner {
            _db: db,
            conn,
            db_path: Some(db_path),
        }))))
    }

    /// Build from an already-opened `Database` shared by the backend store and FTS index.
    pub async fn new_with_db(db: Database, db_path: Option<PathBuf>) -> Result<Self, String> {
        let conn = db
            .connect()
            .map_err(|err| format!("failed to connect to shared libSQL database for FTS: {err}"))?;
        initialize_schema(&conn).await?;

        Ok(Self(Arc::new(RwLock::new(FtsIndexInner {
            _db: db,
            conn,
            db_path,
        }))))
    }

    /// Add or replace all chunks for a single video+source_kind pair.
    /// Deletes existing documents with the matching video_id + source_kind, then adds the new ones.
    pub async fn upsert_source(
        &self,
        meta: FtsSourceMeta<'_>,
        chunks: &[FtsChunk],
    ) -> Result<(), String> {
        let source_key = format!("{}_{}", meta.video_id, meta.source_kind.as_str());
        let inner = self.0.write().await;
        let tx = match inner
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await
        {
            Ok(tx) => tx,
            Err(err) => {
                return Err(format!(
                    "failed to start FTS transaction for {}:{}: {err}",
                    meta.video_id,
                    meta.source_kind.as_str()
                ));
            }
        };

        if let Err(err) = tx
            .execute(
                "DELETE FROM fts_search WHERE source_key = ?1",
                params![source_key.clone()],
            )
            .await
        {
            let _ = tx.rollback().await;
            return Err(format!(
                "failed to clear existing FTS rows for {}:{}: {err}",
                meta.video_id,
                meta.source_kind.as_str()
            ));
        }

        for chunk in chunks {
            if let Err(err) = tx
                .execute(
                    r#"
                    INSERT INTO fts_search (
                        chunk_id,
                        video_id,
                        channel_id,
                        source_kind,
                        source_key,
                        section_title,
                        chunk_text,
                        video_title,
                        channel_name,
                        published_at,
                        start_sec
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                    "#,
                    params![
                        chunk.chunk_id.clone(),
                        meta.video_id,
                        meta.channel_id,
                        meta.source_kind.as_str(),
                        source_key.clone(),
                        chunk.section_title.clone(),
                        chunk.chunk_text.clone(),
                        meta.video_title,
                        meta.channel_name,
                        meta.published_at,
                        chunk.start_sec.map(f64::from),
                    ],
                )
                .await
            {
                let _ = tx.rollback().await;
                return Err(format!(
                    "failed to insert FTS chunk {} for {}:{}: {err}",
                    chunk.chunk_id,
                    meta.video_id,
                    meta.source_kind.as_str()
                ));
            }
        }

        if let Err(err) = tx.commit().await {
            return Err(format!(
                "failed to commit FTS transaction for {}:{}: {err}",
                meta.video_id,
                meta.source_kind.as_str()
            ));
        }

        let doc_count = match query_doc_count(&inner.conn).await {
            Ok(doc_count) => doc_count,
            Err(err) => {
                tracing::warn!(error = %err, "FTS upsert committed but doc count query failed");
                0
            }
        };

        tracing::info!(
            video_id = meta.video_id,
            source_kind = meta.source_kind.as_str(),
            chunks_added = chunks.len(),
            total_docs = doc_count,
            "FTS index updated"
        );
        Ok(())
    }

    /// Remove all indexed documents for a video+source_kind pair.
    pub async fn delete_source(
        &self,
        video_id: &str,
        source_kind: SearchSourceKind,
    ) -> Result<(), String> {
        let source_key = format!("{}_{}", video_id, source_kind.as_str());
        let inner = self.0.write().await;
        inner
            .conn
            .execute(
                "DELETE FROM fts_search WHERE source_key = ?1",
                params![source_key],
            )
            .await
            .map_err(|err| {
                format!(
                    "failed to delete FTS rows for {}:{}: {err}",
                    video_id,
                    source_kind.as_str()
                )
            })?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), String> {
        let inner = self.0.write().await;
        inner
            .conn
            .execute("DELETE FROM fts_search", ())
            .await
            .map_err(|err| format!("failed to clear FTS index: {err}"))?;
        Ok(())
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
        let phrase_queries = build_fts_phrase_queries(query);
        let match_query = build_fts_query(query);
        let relaxed_query = build_fts_relaxed_query(query);
        if phrase_queries.is_empty() && match_query.is_empty() && relaxed_query.is_empty() {
            return Vec::new();
        }

        let inner = self.0.read().await;
        let mut combined_results = Vec::new();
        let mut seen_chunk_ids = std::collections::HashSet::new();
        let max_combined_results = limit.saturating_mul(3).clamp(limit, 200);

        for query_text in phrase_queries
            .into_iter()
            .chain((!match_query.is_empty()).then_some(match_query))
            .chain(
                (!relaxed_query.is_empty() && relaxed_query != build_fts_query(query))
                    .then_some(relaxed_query),
            )
        {
            for result in execute_search_query(
                &inner.conn,
                query_text,
                source_kind,
                channel_id,
                max_combined_results,
            )
            .await
            {
                if seen_chunk_ids.insert(result.chunk_id.clone()) {
                    combined_results.push(result);
                    if combined_results.len() >= max_combined_results {
                        return combined_results;
                    }
                }
            }
        }

        combined_results
    }

    /// Total number of documents in the index.
    pub async fn doc_count(&self) -> u64 {
        let inner = self.0.read().await;
        query_doc_count(&inner.conn).await.unwrap_or(0)
    }

    pub async fn local_db_path(&self) -> Option<PathBuf> {
        let inner = self.0.read().await;
        inner.db_path.clone()
    }
}

async fn execute_search_query(
    conn: &Connection,
    match_query: String,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Vec<FtsSearchResult> {
    let mut sql = String::from(
        r#"
            SELECT
                chunk_id,
                video_id,
                channel_id,
                channel_name,
                video_title,
                source_kind,
                section_title,
                chunk_text,
                published_at,
                start_sec,
                bm25(fts_search, 1.0, 2.5, 2.0) AS rank_score
            FROM fts_search
            WHERE fts_search MATCH ?1
            "#,
    );

    let mut bind_values = vec![Value::Text(match_query)];
    let mut bind_index = 2usize;

    if let Some(source_kind) = source_kind {
        sql.push_str(&format!(" AND source_kind = ?{bind_index}"));
        bind_values.push(Value::Text(source_kind.as_str().to_string()));
        bind_index += 1;
    }

    if let Some(channel_id) = channel_id {
        sql.push_str(&format!(" AND channel_id = ?{bind_index}"));
        bind_values.push(Value::Text(channel_id.to_string()));
        bind_index += 1;
    }

    sql.push_str(" ORDER BY rank_score ASC, published_at DESC");
    sql.push_str(&format!(" LIMIT ?{bind_index}"));
    bind_values.push(Value::Integer(limit.min(200) as i64));

    let mut rows = match conn
        .query(&sql, libsql::params_from_iter(bind_values))
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "FTS query failed");
            return Vec::new();
        }
    };

    let mut results = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        let start_sec = match row.get_value(9) {
            Ok(Value::Null) => None,
            Ok(Value::Real(value)) => Some(value as f32),
            Ok(Value::Integer(value)) => Some(value as f32),
            _ => None,
        };
        let score = match row.get_value(10) {
            Ok(Value::Real(value)) => -(value as f32),
            Ok(Value::Integer(value)) => -(value as f32),
            _ => 0.0,
        };

        let section_title = match row.get_value(6) {
            Ok(Value::Null) => None,
            Ok(Value::Text(value)) => Some(value),
            _ => None,
        };

        let (
            Ok(chunk_id),
            Ok(video_id),
            Ok(channel_id),
            Ok(channel_name),
            Ok(video_title),
            Ok(source_kind),
            Ok(chunk_text),
            Ok(published_at),
        ) = (
            row.get::<String>(0),
            row.get::<String>(1),
            row.get::<String>(2),
            row.get::<String>(3),
            row.get::<String>(4),
            row.get::<String>(5),
            row.get::<String>(7),
            row.get::<String>(8),
        )
        else {
            continue;
        };

        results.push(FtsSearchResult {
            chunk_id,
            video_id,
            channel_id,
            channel_name,
            video_title,
            source_kind: SearchSourceKind::from_db_value(&source_kind),
            section_title,
            chunk_text,
            published_at,
            start_sec,
            score,
        });

        if results.len() >= limit {
            break;
        }
    }

    results
}

async fn build_database(db_path: &Path) -> Result<Database, String> {
    Builder::new_local(db_path)
        .build()
        .await
        .map_err(|err| format!("failed to build local libSQL database: {err}"))
}

async fn initialize_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS fts_search USING fts5 (
            chunk_id UNINDEXED,
            video_id UNINDEXED,
            channel_id UNINDEXED,
            source_kind UNINDEXED,
            source_key UNINDEXED,
            section_title,
            chunk_text,
            video_title,
            channel_name UNINDEXED,
            published_at UNINDEXED,
            start_sec UNINDEXED,
            tokenize = 'porter'
        );
        "#,
    )
    .await
    .map_err(|err| format!("failed to initialize FTS schema: {err}"))?;
    Ok(())
}

async fn query_doc_count(conn: &Connection) -> Result<u64, String> {
    let mut rows = conn
        .query("SELECT COUNT(*) FROM fts_search", ())
        .await
        .map_err(|err| format!("failed to query FTS doc count: {err}"))?;
    let Some(row) = rows
        .next()
        .await
        .map_err(|err| format!("failed to read FTS doc count row: {err}"))?
    else {
        return Ok(0);
    };
    let count = row
        .get::<i64>(0)
        .map_err(|err| format!("failed to decode FTS doc count: {err}"))?;
    Ok(count.max(0) as u64)
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn fts_index_returns_ranked_results() {
        let index = FtsIndex::new().await.expect("index should be created");

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
            .await
            .expect("source should be indexed");

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
            .await
            .expect("source should be indexed");

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
        let index = FtsIndex::new().await.expect("index should be created");

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
                .await
                .expect("source should be indexed");
        }

        let results = index
            .search("semantic search", None, Some("ch-a"), 10)
            .await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].video_id, "v1");
    }

    #[tokio::test]
    async fn fts_index_preserves_exact_phrase_hits_for_common_title_searches() {
        let index = FtsIndex::new().await.expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "target",
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-a",
                    channel_name: "Channel A",
                    video_title: "Anthropic shock wave + One Good Thing",
                    published_at: "2026-01-10T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "target_summary_1_0".to_string(),
                    section_title: Some("One Good Thing".to_string()),
                    chunk_text: "One Good Thing closes the episode after the Sam Altman interview."
                        .to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("target source should be indexed");

        for index_id in 0..6 {
            let video_id = format!("distractor-{index_id}");
            let video_title = format!("Distractor {index_id}");
            let chunk_id = format!("distractor-{index_id}_transcript_1_0");
            let chunk_text =
                format!("A good thing about this demo {index_id} is that the thing works.");
            index
                .upsert_source(
                    FtsSourceMeta {
                        video_id: &video_id,
                        source_kind: SearchSourceKind::Transcript,
                        channel_id: "channel-b",
                        channel_name: "Channel B",
                        video_title: &video_title,
                        published_at: "2026-01-01T00:00:00Z",
                    },
                    &[FtsChunk {
                        chunk_id,
                        section_title: None,
                        chunk_text,
                        start_sec: None,
                    }],
                )
                .await
                .expect("distractor source should be indexed");
        }

        let results = index.search("one good thing", None, None, 5).await;
        assert!(
            !results.is_empty(),
            "exact phrase search should return results"
        );
        assert_eq!(results[0].video_id, "target");
    }

    #[tokio::test]
    async fn fts_index_handles_natural_language_video_queries() {
        let index = FtsIndex::new().await.expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "target",
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-a",
                    channel_name: "Channel A",
                    video_title:
                        "A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming",
                    published_at: "2026-01-10T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "target_summary_1_0".to_string(),
                    section_title: Some("AI Backlash".to_string()),
                    chunk_text: "This episode covers the public AI backlash and an attempted attack on Sam Altman."
                        .to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("target source should be indexed");

        for (index_id, title, chunk_text) in [
            (
                0,
                "I was wrong about GPT-5",
                "This video debates new AI benchmarks and model releases.",
            ),
            (
                1,
                "Can AI Games Be Good?",
                "A discussion about whether AI-generated games can be fun to play.",
            ),
            (
                2,
                "We need to talk about the Claude Code rate limits",
                "A breakdown of the current Claude Code rate limit problems.",
            ),
        ] {
            let video_id = format!("distractor-{index_id}");
            let chunk_id = format!("distractor-{index_id}_summary_1_0");
            index
                .upsert_source(
                    FtsSourceMeta {
                        video_id: &video_id,
                        source_kind: SearchSourceKind::Summary,
                        channel_id: "channel-b",
                        channel_name: "Channel B",
                        video_title: title,
                        published_at: "2026-01-01T00:00:00Z",
                    },
                    &[FtsChunk {
                        chunk_id,
                        section_title: None,
                        chunk_text: chunk_text.to_string(),
                        start_sec: None,
                    }],
                )
                .await
                .expect("distractor source should be indexed");
        }

        let results = index
            .search("find videos about AI backlash", None, None, 5)
            .await;
        assert!(
            !results.is_empty(),
            "natural language search should return results"
        );
        assert_eq!(results[0].video_id, "target");
    }

    #[tokio::test]
    async fn fts_index_handles_conversational_phrase_queries() {
        let index = FtsIndex::new().await.expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "target",
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-a",
                    channel_name: "Channel A",
                    video_title:
                        "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
                    published_at: "2026-01-10T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "target_summary_1_0".to_string(),
                    section_title: Some("One Good Thing".to_string()),
                    chunk_text: "One Good Thing closes the episode after the Sam Altman interview."
                        .to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("target source should be indexed");

        for (index_id, title, chunk_text) in [
            (
                0,
                "The talk that changed the web",
                "A broad discussion about web standards and browser politics.",
            ),
            (
                1,
                "We need to talk about founder mode",
                "A creator commentary video about management culture and startups.",
            ),
        ] {
            let video_id = format!("distractor-{index_id}");
            let chunk_id = format!("distractor-{index_id}_summary_1_0");
            index
                .upsert_source(
                    FtsSourceMeta {
                        video_id: &video_id,
                        source_kind: SearchSourceKind::Summary,
                        channel_id: "channel-b",
                        channel_name: "Channel B",
                        video_title: title,
                        published_at: "2026-01-01T00:00:00Z",
                    },
                    &[FtsChunk {
                        chunk_id,
                        section_title: None,
                        chunk_text: chunk_text.to_string(),
                        start_sec: None,
                    }],
                )
                .await
                .expect("distractor source should be indexed");
        }

        let results = index
            .search("video where they talk about one good thing", None, None, 5)
            .await;
        assert!(
            !results.is_empty(),
            "conversational phrase search should return results"
        );
        assert_eq!(results[0].video_id, "target");
    }

    #[tokio::test]
    async fn fts_index_delete_source_removes_documents() {
        let index = FtsIndex::new().await.expect("index should be created");

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
            .await
            .expect("source should be indexed");

        let before = index.search("deleted", None, None, 10).await;
        assert!(!before.is_empty());

        index
            .delete_source("video-del", SearchSourceKind::Transcript)
            .await
            .expect("delete should succeed");

        let after = index.search("deleted", None, None, 10).await;
        assert!(
            after.is_empty(),
            "deleted source should not appear in results"
        );
    }

    #[tokio::test]
    async fn fts_index_uses_persistent_local_database_file() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let index = FtsIndex::new_in_dir(temp_dir.path())
            .await
            .expect("index should be created");

        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: "video-persisted",
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-persisted",
                    channel_name: "Persisted Channel",
                    video_title: "Persistent FTS",
                    published_at: "2026-01-03T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: "video-persisted_summary_1_0".to_string(),
                    section_title: Some("Restore".to_string()),
                    chunk_text: "Persisted libsql search rows should survive reopen.".to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("source should be indexed");

        let reopened = FtsIndex::new_in_dir(temp_dir.path())
            .await
            .expect("index should reopen");
        let results = reopened.search("persisted reopen", None, None, 10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].video_id, "video-persisted");
    }
}
