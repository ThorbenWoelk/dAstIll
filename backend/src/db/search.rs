use aws_smithy_types::Document;

use crate::models::{ContentStatus, Summary, Transcript, Video};
use crate::services::search::{SearchCandidate, SearchIndexChunk, SearchSourceKind};

use super::{
    SearchMaterial, SearchProgressMaterial, SearchSourceCounts, SearchSourceRecord,
    SearchSourceState, Store, StoreError, format_aws_error,
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
    store
        .delete_key(&search_source_key(video_id, source_kind))
        .await
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
        if r.index_status != "pending" {
            continue;
        }
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
    channel_id: &str,
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

    #[derive(serde::Serialize)]
    struct ChunkData<'a> {
        video_id: &'a str,
        source_kind: &'a str,
        section_title: Option<&'a str>,
        chunk_text: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_sec: Option<f32>,
    }

    let mut put_batch: Vec<aws_sdk_s3vectors::types::PutInputVector> = Vec::new();

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
        store
            .put_json(
                &format!("search-chunks/{vkey}.json"),
                &ChunkData {
                    video_id,
                    source_kind: source_kind.as_str(),
                    section_title: chunk.section_title.as_deref(),
                    chunk_text: &chunk.chunk_text,
                    start_sec: chunk.start_sec,
                },
            )
            .await?;

        let chunk_text_clamped: String = chunk.chunk_text.chars().take(30_000).collect();
        let mut meta_entries: Vec<(&str, Document)> = vec![
            ("video_id", Document::String(video_id.to_string())),
            ("channel_id", Document::String(channel_id.to_string())),
            (
                "source_kind",
                Document::String(source_kind.as_str().to_string()),
            ),
            ("chunk_text", Document::String(chunk_text_clamped)),
            (
                "source_generation",
                Document::Number(aws_smithy_types::Number::Float(
                    current.source_generation as f64,
                )),
            ),
            (
                "chunk_index",
                Document::Number(aws_smithy_types::Number::Float(chunk.chunk_index as f64)),
            ),
        ];
        if let Some(ref title) = chunk.section_title {
            meta_entries.push(("section_title", Document::String(title.clone())));
        }
        if let Some(sec) = chunk.start_sec {
            meta_entries.push((
                "start_sec",
                Document::Number(aws_smithy_types::Number::Float(sec as f64)),
            ));
        }

        let put_vector = aws_sdk_s3vectors::types::PutInputVector::builder()
            .key(vkey)
            .data(aws_sdk_s3vectors::types::VectorData::Float32(embedding))
            .metadata(build_metadata(meta_entries))
            .build()
            .map_err(|e| StoreError::S3Vectors(e.to_string()))?;

        put_batch.push(put_vector);

        // Flush in batches of 500 (API limit)
        if put_batch.len() >= 500 {
            store
                .s3v
                .put_vectors()
                .vector_bucket_name(&store.vector_bucket)
                .index_name(&store.vector_index)
                .set_vectors(Some(std::mem::take(&mut put_batch)))
                .send()
                .await
                .map_err(|e| StoreError::S3Vectors(format_aws_error(&e)))?;
        }
    }

    // Flush remaining vectors
    if !put_batch.is_empty() {
        store
            .s3v
            .put_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index)
            .set_vectors(Some(put_batch))
            .send()
            .await
            .map_err(|e| StoreError::S3Vectors(format_aws_error(&e)))?;
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
    // Derive vector keys from S3 chunk objects (avoids scanning entire vector index)
    let prefix = format!("search-chunks/{video_id}_{}_", source_kind.as_str());
    let chunk_keys = store.list_keys(&prefix).await?;

    let vector_keys: Vec<String> = chunk_keys
        .iter()
        .filter_map(|k| {
            k.strip_prefix("search-chunks/")
                .and_then(|s| s.strip_suffix(".json"))
                .map(|s| s.to_string())
        })
        .collect();

    // Batch delete vectors (up to 500 per call)
    for batch in vector_keys.chunks(500) {
        let mut req = store
            .s3v
            .delete_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index);
        for key in batch {
            req = req.keys(key);
        }
        req.send().await.ok();
    }

    // Clean up S3 chunk objects
    for key in &chunk_keys {
        store.delete_key(key).await.ok();
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
    let Some(video) = super::videos::get_video(store, video_id, false).await? else {
        return Ok(None);
    };
    let channel_name = store
        .get_json::<crate::models::Channel>(&format!("channels/{}.json", video.channel_id))
        .await?
        .map(|c| c.name)
        .unwrap_or_default();

    let (content, timed_segments) = match source_kind {
        SearchSourceKind::Transcript if video.transcript_status == ContentStatus::Ready => {
            let transcript = store
                .get_json::<Transcript>(&format!("transcripts/{video_id}.json"))
                .await?;
            let text = transcript
                .as_ref()
                .and_then(|t| t.raw_text.clone().or_else(|| t.formatted_markdown.clone()))
                .unwrap_or_default();
            let timed = transcript.and_then(|t| t.timed_text);
            (text, timed)
        }
        SearchSourceKind::Summary if video.summary_status == ContentStatus::Ready => {
            let content = store
                .get_json::<Summary>(&format!("summaries/{video_id}.json"))
                .await?
                .map(|s| s.content)
                .unwrap_or_default();
            (content, None)
        }
        _ => return Ok(None),
    };

    let content = content.trim().to_string();
    if content.is_empty() {
        return Ok(None);
    }

    Ok(Some(SearchMaterial {
        video_id: video_id.to_string(),
        channel_id: video.channel_id.clone(),
        channel_name,
        video_title: video.title,
        published_at: video.published_at.to_rfc3339(),
        source_kind,
        content,
        timed_segments,
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

    let all_videos: Vec<Video> = super::videos::load_all_videos(store).await?;
    let mut materials = Vec::new();

    for video in &all_videos {
        if materials.len() >= limit {
            break;
        }
        if video.summary_status == ContentStatus::Ready
            && !indexed.contains(&(video.id.clone(), "summary".to_string()))
        {
            if let Some(mat) =
                load_search_material(store, &video.id, SearchSourceKind::Summary).await?
            {
                materials.push(mat);
            }
        }
    }
    for video in &all_videos {
        if materials.len() >= limit {
            break;
        }
        if video.transcript_status == ContentStatus::Ready
            && !indexed.contains(&(video.id.clone(), "transcript".to_string()))
        {
            if let Some(mat) =
                load_search_material(store, &video.id, SearchSourceKind::Transcript).await?
            {
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

    for source in all_sources
        .iter()
        .filter(|s| s.index_status == "ready" || s.index_status == "failed")
    {
        if materials.len() >= limit {
            break;
        }
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
    let all_videos: Vec<Video> = super::videos::load_all_videos(store).await?;
    let all_sources: Vec<SearchSourceRecord> = store.load_all("search-sources/").await?;
    let source_map: std::collections::HashMap<(String, String), &SearchSourceRecord> = all_sources
        .iter()
        .map(|s| ((s.video_id.clone(), s.source_kind.clone()), s))
        .collect();

    let mut materials = Vec::new();
    for video in &all_videos {
        if video.summary_status == ContentStatus::Ready {
            if let Some(summary) = store
                .get_json::<Summary>(&format!("summaries/{}.json", video.id))
                .await?
            {
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
            if let Some(transcript) = store
                .get_json::<Transcript>(&format!("transcripts/{}.json", video.id))
                .await?
            {
                let content = transcript
                    .raw_text
                    .or(transcript.formatted_markdown)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
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
    if embedding.is_empty() {
        return Ok(Vec::new());
    }

    let top_k = limit.clamp(1, 100);

    let mut req = store
        .s3v
        .query_vectors()
        .vector_bucket_name(&store.vector_bucket)
        .index_name(&store.vector_index)
        .query_vector(aws_sdk_s3vectors::types::VectorData::Float32(embedding))
        .top_k(top_k as i32)
        .return_metadata(true);

    // Server-side filters on filterable metadata fields.
    // channel_id was added to metadata in a later release - old vectors without it
    // will be excluded by this filter until re-indexed (acceptable transition behaviour).
    let mut filter_map = std::collections::HashMap::new();
    if let Some(kind) = source_kind {
        filter_map.insert(
            "source_kind".to_string(),
            Document::String(kind.as_str().to_string()),
        );
    }
    if let Some(cid) = channel_id {
        filter_map.insert("channel_id".to_string(), Document::String(cid.to_string()));
    }
    if !filter_map.is_empty() {
        req = req.filter(Document::Object(filter_map));
    }

    let vectors = match req.send().await {
        Ok(output) => output.vectors,
        Err(err) => {
            tracing::debug!(error = %format_aws_error(&err), "vector search unavailable");
            return Ok(Vec::new());
        }
    };

    // Collect unique video IDs from results, then fetch only those
    let video_ids: std::collections::HashSet<String> = vectors
        .iter()
        .filter_map(|v| {
            v.metadata
                .as_ref()
                .and_then(|m| get_doc_string(m, "video_id"))
        })
        .collect();

    let mut video_map: std::collections::HashMap<String, Video> = std::collections::HashMap::new();
    let mut channel_map: std::collections::HashMap<String, crate::models::Channel> =
        std::collections::HashMap::new();
    for vid in &video_ids {
        if let Some(video) = super::videos::get_video(store, vid, false).await? {
            if !channel_map.contains_key(&video.channel_id) {
                if let Some(ch) = store
                    .get_json::<crate::models::Channel>(&format!(
                        "channels/{}.json",
                        video.channel_id
                    ))
                    .await?
                {
                    channel_map.insert(ch.id.clone(), ch);
                }
            }
            video_map.insert(vid.clone(), video);
        }
    }

    let mut candidates = Vec::new();
    for v in vectors {
        let empty_doc = Document::Object(Default::default());
        let meta = v.metadata.as_ref().unwrap_or(&empty_doc);

        let vid = get_doc_string(meta, "video_id").unwrap_or_default();
        let sk = get_doc_string(meta, "source_kind").unwrap_or_default();

        let Some(video) = video_map.get(&vid) else {
            continue;
        };
        let ch = channel_map.get(&video.channel_id);

        let start_sec = match meta {
            Document::Object(map) => map.get("start_sec").and_then(|d| match d {
                Document::Number(n) => Some(n.to_f64_lossy() as f32),
                _ => None,
            }),
            _ => None,
        };

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
            start_sec,
        });

        if candidates.len() >= limit {
            break;
        }
    }
    Ok(candidates)
}

pub async fn search_exact_global_candidates(
    store: &Store,
    query_embedding: &str,
    embedding_model: &str,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, StoreError> {
    search_vector_candidates(
        store,
        query_embedding,
        embedding_model,
        source_kind,
        channel_id,
        limit,
    )
    .await
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
    let all_videos: Vec<Video> = super::videos::load_all_videos(store).await?;
    let total_sources: usize = all_videos
        .iter()
        .map(|v| {
            (v.transcript_status == ContentStatus::Ready) as usize
                + (v.summary_status == ContentStatus::Ready) as usize
        })
        .sum();

    Ok(SearchSourceCounts {
        pending,
        indexing,
        ready,
        failed,
        total_sources,
    })
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
    for batch in all_keys.chunks(500) {
        let mut req = store
            .s3v
            .delete_vectors()
            .vector_bucket_name(&store.vector_bucket)
            .index_name(&store.vector_index);
        for key in batch {
            req = req.keys(key);
        }
        req.send().await.ok();
    }
    Ok(())
}
