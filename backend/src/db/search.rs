use aws_smithy_types::Document;

use crate::models::{ContentStatus, Summary, Transcript, Video};
use crate::services::search::{SearchCandidate, SearchIndexChunk, SearchSourceKind};

use super::{
    SearchMaterial, SearchProgressMaterial, SearchSourceCounts, SearchSourceRecord,
    SearchSourceState, Store, StoreError,
};

fn search_source_key(video_id: &str, source_kind: SearchSourceKind) -> String {
    format!("search-sources/{video_id}/{}.json", source_kind.as_str())
}

fn vector_key(video_id: &str, source_kind: &str, generation: i64, chunk_index: usize) -> String {
    format!("{video_id}_{source_kind}_{generation}_{chunk_index}")
}

fn source_id_from_video_kind(video_id: &str, kind: &str) -> i64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    video_id.hash(&mut hasher);
    kind.hash(&mut hasher);
    (hasher.finish() & 0x7FFFFFFFFFFFFFFF) as i64
}

fn build_metadata(entries: Vec<(&str, Document)>) -> Document {
    let map: std::collections::HashMap<String, Document> = entries
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    Document::Object(map)
}

fn get_doc_string(doc: &Document, key: &str) -> Option<String> {
    match doc {
        Document::Object(map) => match map.get(key) {
            Some(Document::String(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

pub async fn mark_search_source_pending(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
) -> Result<(), StoreError> {
    let key = search_source_key(video_id, source_kind);
    let existing = store.get_json::<SearchSourceRecord>(&key).await?;
    let (id, generation) = match existing {
        Some(r) => (r.id, r.source_generation + 1),
        None => (source_id_from_video_kind(video_id, source_kind.as_str()), 1),
    };
    let record = SearchSourceRecord {
        id,
        source_generation: generation,
        video_id: video_id.to_string(),
        source_kind: source_kind.as_str().to_string(),
        content_hash: content_hash.to_string(),
        embedding_model: None,
        index_status: "pending".to_string(),
        last_indexed_at: None,
        last_error: None,
    };
    store.put_json(&key, &record).await
}

pub async fn clear_search_source(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), StoreError> {
    delete_vectors_for_source(store, video_id, source_kind).await?;
    store.delete_key(&search_source_key(video_id, source_kind)).await
}

pub async fn get_search_source_state(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<Option<SearchSourceState>, StoreError> {
    let record: Option<SearchSourceRecord> = store
        .get_json(&search_source_key(video_id, source_kind))
        .await?;
    Ok(record.map(SearchSourceState::from))
}

pub async fn list_pending_search_sources(
    store: &Store,
    limit: usize,
) -> Result<Vec<SearchSourceState>, StoreError> {
    let all: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let mut summaries = Vec::new();
    let mut transcripts = Vec::new();

    for r in all {
        if r.index_status != "pending" { continue; }
        let state = SearchSourceState::from(r);
        match state.source_kind {
            SearchSourceKind::Summary => summaries.push(state),
            SearchSourceKind::Transcript => transcripts.push(state),
        }
    }

    summaries.truncate(limit);
    if summaries.len() < limit {
        transcripts.truncate(limit - summaries.len());
        summaries.extend(transcripts);
    }
    Ok(summaries)
}

pub async fn mark_search_source_indexing(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
) -> Result<bool, StoreError> {
    let key = search_source_key(video_id, source_kind);
    let Some(mut record) = store.get_json::<SearchSourceRecord>(&key).await? else {
        return Ok(false);
    };
    if record.content_hash != content_hash || record.index_status != "pending" {
        return Ok(false);
    }
    record.index_status = "indexing".to_string();
    record.last_error = None;
    store.put_json(&key, &record).await?;
    Ok(true)
}

pub async fn mark_search_source_failed(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
    error: &str,
) -> Result<(), StoreError> {
    let key = search_source_key(video_id, source_kind);
    if let Some(mut record) = store.get_json::<SearchSourceRecord>(&key).await? {
        if record.content_hash == content_hash {
            record.index_status = "failed".to_string();
            record.last_error = Some(error.to_string());
            store.put_json(&key, &record).await?;
        }
    }
    Ok(())
}

pub async fn replace_search_chunks(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
    embedding_model: Option<&str>,
    chunks: &[SearchIndexChunk],
) -> Result<bool, StoreError> {
    let key = search_source_key(video_id, source_kind);
    let Some(current) = store.get_json::<SearchSourceRecord>(&key).await? else {
        return Ok(false);
    };
    if current.content_hash != content_hash || current.index_status != "indexing" {
        return Ok(false);
    }

    delete_vectors_for_source(store, video_id, source_kind).await?;

    for chunk in chunks {
        let embedding = chunk
            .embedding_json
            .as_deref()
            .and_then(|json| serde_json::from_str::<Vec<f32>>(json).ok());
        let Some(embedding) = embedding else { continue };

        let vkey = vector_key(
            video_id,
            source_kind.as_str(),
            current.source_generation,
            chunk.chunk_index,
        );

        // Store full chunk text in S3 for FTS and retrieval
        let chunk_s3_key = format!("search-chunks/{vkey}.json");
        #[derive(serde::Serialize)]
        struct ChunkData<'a> {
            video_id: &'a str,
            source_kind: &'a str,
            section_title: Option<&'a str>,
            chunk_text: &'a str,
        }
        store
            .put_json(
                &chunk_s3_key,
                &ChunkData {
                    video_id,
                    source_kind: source_kind.as_str(),
                    section_title: chunk.section_title.as_deref(),
                    chunk_text: &chunk.chunk_text,
                },
            )
            .await?;

        // Metadata: keep small (filterable <= 2 KB, total <= 40 KB)
        // chunk_text + section_title are declared non-filterable in the index
        let chunk_text_clamped: String = chunk.chunk_text.chars().take(30_000).collect();
        let mut meta_entries: Vec<(&str, Document)> = vec![
            ("video_id", Document::String(video_id.to_string())),
            ("source_kind", Document::String(source_kind.as_str().to_string())),
            ("chunk_text", Document::String(chunk_text_clamped)),
            ("source_generation", Document::Number(aws_smithy_types::Number::Float(current.source_generation as f64))),
            ("chunk_index", Document::Number(aws_smithy_types::Number::Float(chunk.chunk_index as f64))),
        ];
        if let Some(ref title) = chunk.section_title {
            meta_entries.push(("section_title", Document::String(title.clone())));
        }

        let vector_data = aws_sdk_s3vectors::types::VectorData::Float32(embedding);
        let put_vector = aws_sdk_s3vectors::types::PutInputVector::builder()
            .key(vkey)
            .data(vector_data)
            .metadata(build_metadata(meta_entries))
            .build()
            .map_err(|e| StoreError::S3Vectors(e.to_string()))?;

        store
            .s3v
            .put_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index)
            .vectors(put_vector)
            .send()
            .await
            .map_err(|e| StoreError::S3Vectors(format!("{e:#}")))?;
    }

    let mut updated = current;
    updated.embedding_model = embedding_model.map(ToOwned::to_owned);
    updated.index_status = "ready".to_string();
    updated.last_indexed_at = Some(chrono::Utc::now().to_rfc3339());
    updated.last_error = None;
    store.put_json(&key, &updated).await?;
    Ok(true)
}

async fn list_all_vector_keys(store: &Store) -> Vec<String> {
    let mut all_keys = Vec::new();
    let mut next_token: Option<String> = None;
    loop {
        let mut req = store
            .s3v
            .list_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index);
        if let Some(token) = next_token.take() {
            req = req.next_token(token);
        }
        match req.send().await {
            Ok(output) => {
                for v in &output.vectors {
                    all_keys.push(v.key.clone());
                }
                next_token = output.next_token;
                if next_token.is_none() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    all_keys
}

async fn delete_vectors_for_source(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), StoreError> {
    let prefix = format!("{video_id}_{}_", source_kind.as_str());
    let all_keys = list_all_vector_keys(store).await;

    let keys_to_delete: Vec<&String> = all_keys
        .iter()
        .filter(|k| k.starts_with(&prefix))
        .collect();

    for key in &keys_to_delete {
        store
            .s3v
            .delete_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index)
            .keys(key.as_str())
            .send()
            .await
            .ok();
        // Also clean up the S3 chunk text
        store.delete_key(&format!("search-chunks/{key}.json")).await.ok();
    }
    Ok(())
}

pub(crate) async fn delete_vectors_for_video(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    delete_vectors_for_source(store, video_id, SearchSourceKind::Summary).await?;
    delete_vectors_for_source(store, video_id, SearchSourceKind::Transcript).await
}

pub async fn load_search_material(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<Option<SearchMaterial>, StoreError> {
    let Some(video) = store.get_json::<Video>(&format!("videos/{video_id}.json")).await? else {
        return Ok(None);
    };
    let channel_name = store
        .get_json::<crate::models::Channel>(&format!("channels/{}.json", video.channel_id))
        .await?
        .map(|c| c.name)
        .unwrap_or_default();

    let content = match source_kind {
        SearchSourceKind::Transcript if video.transcript_status == ContentStatus::Ready => {
            store
                .get_json::<Transcript>(&format!("transcripts/{video_id}.json"))
                .await?
                .and_then(|t| t.raw_text.or(t.formatted_markdown))
                .unwrap_or_default()
        }
        SearchSourceKind::Summary if video.summary_status == ContentStatus::Ready => {
            store
                .get_json::<Summary>(&format!("summaries/{video_id}.json"))
                .await?
                .map(|s| s.content)
                .unwrap_or_default()
        }
        _ => return Ok(None),
    };

    let content = content.trim().to_string();
    if content.is_empty() { return Ok(None); }

    Ok(Some(SearchMaterial {
        video_id: video_id.to_string(),
        channel_name,
        video_title: video.title,
        source_kind,
        content,
    }))
}

pub async fn list_search_backfill_materials(
    store: &Store,
    limit: usize,
) -> Result<Vec<SearchMaterial>, StoreError> {
    let all_sources: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let indexed: std::collections::HashSet<(String, String)> = all_sources
        .iter()
        .map(|s| (s.video_id.clone(), s.source_kind.clone()))
        .collect();

    let all_videos: Vec<Video> = store.load_all("videos/").await?;
    let mut materials = Vec::new();

    for video in &all_videos {
        if materials.len() >= limit { break; }
        if video.summary_status == ContentStatus::Ready
            && !indexed.contains(&(video.id.clone(), "summary".to_string()))
        {
            if let Some(mat) = load_search_material(store, &video.id, SearchSourceKind::Summary).await? {
                materials.push(mat);
            }
        }
    }
    for video in &all_videos {
        if materials.len() >= limit { break; }
        if video.transcript_status == ContentStatus::Ready
            && !indexed.contains(&(video.id.clone(), "transcript".to_string()))
        {
            if let Some(mat) = load_search_material(store, &video.id, SearchSourceKind::Transcript).await? {
                materials.push(mat);
            }
        }
    }
    Ok(materials)
}

pub async fn list_search_reconciliation_materials(
    store: &Store,
    limit: usize,
) -> Result<Vec<SearchMaterial>, StoreError> {
    let all_sources: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let mut materials = Vec::new();

    for source in all_sources.iter().filter(|s| s.index_status == "ready" || s.index_status == "failed") {
        if materials.len() >= limit { break; }
        let kind = SearchSourceKind::from_db_value(&source.source_kind);
        if let Some(mat) = load_search_material(store, &source.video_id, kind).await? {
            materials.push(mat);
        }
    }
    Ok(materials)
}

pub async fn list_search_progress_materials(
    store: &Store,
) -> Result<Vec<SearchProgressMaterial>, StoreError> {
    let all_videos: Vec<Video> = store.load_all("videos/").await?;
    let all_sources: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let source_map: std::collections::HashMap<(String, String), &SearchSourceRecord> = all_sources
        .iter()
        .map(|s| ((s.video_id.clone(), s.source_kind.clone()), s))
        .collect();

    let mut materials = Vec::new();
    for video in &all_videos {
        if video.summary_status == ContentStatus::Ready {
            if let Some(summary) = store.get_json::<Summary>(&format!("summaries/{}.json", video.id)).await? {
                let content = summary.content.trim().to_string();
                if !content.is_empty() {
                    let source = source_map.get(&(video.id.clone(), "summary".to_string()));
                    materials.push(SearchProgressMaterial {
                        video_id: video.id.clone(),
                        source_kind: SearchSourceKind::Summary,
                        content,
                        index_status: source.map(|s| s.index_status.clone()),
                        embedding_model: source.and_then(|s| s.embedding_model.clone()),
                    });
                }
            }
        }
        if video.transcript_status == ContentStatus::Ready {
            if let Some(transcript) = store.get_json::<Transcript>(&format!("transcripts/{}.json", video.id)).await? {
                let content = transcript.raw_text.or(transcript.formatted_markdown).unwrap_or_default().trim().to_string();
                if !content.is_empty() {
                    let source = source_map.get(&(video.id.clone(), "transcript".to_string()));
                    materials.push(SearchProgressMaterial {
                        video_id: video.id.clone(),
                        source_kind: SearchSourceKind::Transcript,
                        content,
                        index_status: source.map(|s| s.index_status.clone()),
                        embedding_model: source.and_then(|s| s.embedding_model.clone()),
                    });
                }
            }
        }
    }
    Ok(materials)
}

pub async fn search_vector_candidates(
    store: &Store,
    query_embedding: &str,
    _embedding_model: &str,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, StoreError> {
    let embedding: Vec<f32> = serde_json::from_str(query_embedding).unwrap_or_default();
    if embedding.is_empty() { return Ok(Vec::new()); }

    let result = store
        .s3v
        .query_vectors()
        .vector_bucket_name(&store.vector_bucket)
        .index_name(&store.vector_index)
        .query_vector(aws_sdk_s3vectors::types::VectorData::Float32(embedding))
        .top_k(limit as i32)
        .return_metadata(true)
        .send()
        .await;

    let vectors = match result {
        Ok(output) => output.vectors,
        Err(err) => {
            tracing::debug!(error = %err, "vector search unavailable");
            return Ok(Vec::new());
        }
    };

    let all_videos: Vec<Video> = store.load_all("videos/").await?;
    let video_map: std::collections::HashMap<&str, &Video> =
        all_videos.iter().map(|v| (v.id.as_str(), v)).collect();
    let all_channels: Vec<crate::models::Channel> = store.load_all("channels/").await?;
    let channel_map: std::collections::HashMap<&str, &crate::models::Channel> =
        all_channels.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut candidates = Vec::new();
    for v in vectors {
        let metadata = v.metadata.as_ref();
        let empty_doc = Document::Object(Default::default());
        let meta = metadata.unwrap_or(&empty_doc);

        let vid = get_doc_string(meta, "video_id").unwrap_or_default();
        let sk = get_doc_string(meta, "source_kind").unwrap_or_default();

        if source_kind.is_some_and(|f| sk != f.as_str()) { continue; }

        let video = video_map.get(vid.as_str());
        if channel_id.is_some_and(|f| video.is_none_or(|v| v.channel_id != f)) { continue; }

        let Some(video) = video else { continue };
        let ch = channel_map.get(video.channel_id.as_str());

        candidates.push(SearchCandidate {
            chunk_id: v.key.clone(),
            video_id: vid,
            channel_id: video.channel_id.clone(),
            channel_name: ch.map(|c| c.name.clone()).unwrap_or_default(),
            video_title: video.title.clone(),
            source_kind: SearchSourceKind::from_db_value(&sk),
            section_title: get_doc_string(meta, "section_title"),
            chunk_text: get_doc_string(meta, "chunk_text").unwrap_or_default(),
            published_at: video.published_at.to_rfc3339(),
        });
    }
    Ok(candidates)
}

pub async fn search_fts_candidates(
    store: &Store,
    query: &str,
    _embedding_model: Option<&str>,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, StoreError> {
    let query_tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| t.trim_matches('"').to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if query_tokens.is_empty() { return Ok(Vec::new()); }

    let all_videos: Vec<Video> = store.load_all("videos/").await?;
    let video_map: std::collections::HashMap<&str, &Video> =
        all_videos.iter().map(|v| (v.id.as_str(), v)).collect();
    let all_channels: Vec<crate::models::Channel> = store.load_all("channels/").await?;
    let channel_map: std::collections::HashMap<&str, &crate::models::Channel> =
        all_channels.iter().map(|c| (c.id.as_str(), c)).collect();

    // Scan chunk text from S3 objects (stored during indexing)
    #[derive(serde::Deserialize)]
    struct ChunkData {
        video_id: String,
        source_kind: String,
        section_title: Option<String>,
        chunk_text: String,
    }

    let chunk_keys = store.list_keys("search-chunks/").await?;
    let mut scored: Vec<(SearchCandidate, usize)> = Vec::new();

    for chunk_key in chunk_keys {
        let Some(chunk) = store.get_json::<ChunkData>(&chunk_key).await? else { continue };

        if source_kind.is_some_and(|f| chunk.source_kind != f.as_str()) { continue; }

        let video = video_map.get(chunk.video_id.as_str());
        if channel_id.is_some_and(|f| video.is_none_or(|v| v.channel_id != f)) { continue; }

        let text_lower = chunk.chunk_text.to_lowercase();
        let match_count = query_tokens.iter().filter(|t| text_lower.contains(t.as_str())).count();
        if match_count == 0 { continue; }

        let Some(video) = video else { continue };
        let ch = channel_map.get(video.channel_id.as_str());

        let chunk_id = chunk_key
            .strip_prefix("search-chunks/")
            .and_then(|s| s.strip_suffix(".json"))
            .unwrap_or(&chunk_key)
            .to_string();

        scored.push((
            SearchCandidate {
                chunk_id,
                video_id: chunk.video_id,
                channel_id: video.channel_id.clone(),
                channel_name: ch.map(|c| c.name.clone()).unwrap_or_default(),
                video_title: video.title.clone(),
                source_kind: SearchSourceKind::from_db_value(&chunk.source_kind),
                section_title: chunk.section_title,
                chunk_text: chunk.chunk_text,
                published_at: video.published_at.to_rfc3339(),
            },
            match_count,
        ));
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(limit);
    Ok(scored.into_iter().map(|(c, _)| c).collect())
}

pub async fn search_exact_candidates(
    _store: &Store,
    _query_embedding: &str,
    _embedding_model: &str,
    _candidate_ids: &[i64],
    _limit: usize,
) -> Result<Vec<SearchCandidate>, StoreError> {
    Ok(Vec::new())
}

pub async fn search_exact_global_candidates(
    store: &Store,
    query_embedding: &str,
    embedding_model: &str,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, StoreError> {
    search_vector_candidates(store, query_embedding, embedding_model, source_kind, channel_id, limit).await
}

pub async fn get_search_source_counts(store: &Store) -> Result<SearchSourceCounts, StoreError> {
    let all_sources: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let (mut pending, mut indexing, mut ready, mut failed) = (0, 0, 0, 0);
    for s in &all_sources {
        match s.index_status.as_str() {
            "pending" => pending += 1,
            "indexing" => indexing += 1,
            "ready" => ready += 1,
            "failed" => failed += 1,
            _ => {}
        }
    }
    let all_videos: Vec<Video> = store.load_all("videos/").await?;
    let total_sources: usize = all_videos
        .iter()
        .map(|v| {
            (v.transcript_status == ContentStatus::Ready) as usize
                + (v.summary_status == ContentStatus::Ready) as usize
        })
        .sum();

    Ok(SearchSourceCounts { pending, indexing, ready, failed, total_sources })
}

pub async fn prune_stale_search_rows(_store: &Store, _limit: usize) -> Result<usize, StoreError> {
    Ok(0)
}

pub async fn has_vector_index(_store: &Store) -> Result<bool, StoreError> {
    Ok(true)
}

pub async fn ensure_vector_index(_store: &Store) -> Result<(), StoreError> {
    Ok(())
}

pub async fn reset_search_projection(store: &Store) -> Result<(), StoreError> {
    store.delete_prefix("search-sources/").await?;
    store.delete_prefix("search-chunks/").await?;

    let all_keys = list_all_vector_keys(store).await;
    for key in all_keys {
        store
            .s3v
            .delete_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index)
            .keys(&key)
            .send()
            .await
            .ok();
    }
    Ok(())
}
