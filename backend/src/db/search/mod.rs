use crate::models::{ContentStatus, Summary, Transcript, Video};
use crate::search::{SearchCandidate, SearchIndexChunk, SearchSourceKind};

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

pub fn should_refresh_search_source(
    current: Option<&SearchSourceState>,
    content_hash: &str,
    semantic_enabled: bool,
    runtime_embedding_model: Option<&str>,
) -> bool {
    let Some(current) = current else {
        return true;
    };

    current.content_hash != content_hash
        || current.index_status == "failed"
        || (semantic_enabled && current.embedding_model.as_deref() != runtime_embedding_model)
}

pub async fn clear_search_source(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), StoreError> {
    delete_search_artifacts_for_source(store, video_id, source_kind).await?;
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
    _channel_id: &str,
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

    delete_search_artifacts_for_source(store, video_id, source_kind).await?;

    #[derive(serde::Serialize)]
    struct ChunkData<'a> {
        video_id: &'a str,
        source_kind: &'a str,
        section_title: Option<&'a str>,
        chunk_text: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_sec: Option<f32>,
    }

    let mut bundle_data = Vec::new();

    for chunk in chunks {
        let vkey = vector_key(
            video_id,
            source_kind.as_str(),
            current.source_generation,
            chunk.chunk_index,
        );

        let chunk_item = ChunkData {
            video_id,
            source_kind: source_kind.as_str(),
            section_title: chunk.section_title.as_deref(),
            chunk_text: &chunk.chunk_text,
            start_sec: chunk.start_sec,
        };

        // 1. Maintain individual chunks for now (to avoid breaking current search results/FTS)
        store
            .put_json(&format!("search-chunks/{vkey}.json"), &chunk_item)
            .await?;

        // 2. Add to bundle for future optimized hydration
        bundle_data.push(chunk_item);
    }

    // 3. Write the consolidated bundle (compressed)
    if !bundle_data.is_empty() {
        let bundle_key = format!(
            "search-bundles/{}_{}_{}.json.gz",
            video_id,
            source_kind.as_str(),
            current.source_generation
        );
        store.put_json_gz(&bundle_key, &bundle_data).await?;
    }

    let mut updated = current;
    updated.embedding_model = embedding_model.map(ToOwned::to_owned);
    updated.index_status = "ready".to_string();
    updated.last_indexed_at = Some(chrono::Utc::now().to_rfc3339());
    updated.last_error = None;
    store.put_json(&key, &updated).await?;
    Ok(true)
}

async fn delete_search_artifacts_for_source(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), StoreError> {
    let prefix = format!("search-chunks/{video_id}_{}_", source_kind.as_str());
    let chunk_keys = store.list_keys(&prefix).await?;

    for key in &chunk_keys {
        store.delete_key(key).await.ok();
    }

    let bundle_prefix = format!("search-bundles/{}_{}_", video_id, source_kind.as_str());
    let bundle_keys = store.list_keys(&bundle_prefix).await?;
    for key in &bundle_keys {
        store.delete_key(key).await.ok();
    }

    Ok(())
}

pub(crate) async fn delete_search_artifacts_for_video(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    delete_search_artifacts_for_source(store, video_id, SearchSourceKind::Summary).await?;
    delete_search_artifacts_for_source(store, video_id, SearchSourceKind::Transcript).await
}

pub async fn load_search_material(
    store: &Store,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<Option<SearchMaterial>, StoreError> {
    let Some(video) = super::videos::get_video(store, video_id, false).await? else {
        return Ok(None);
    };
    let channel_name = super::channels::get_canonical_channel(store, &video.channel_id)
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
    _store: &Store,
    _query_embedding: &str,
    _embedding_model: &str,
    _source_kind: Option<SearchSourceKind>,
    _channel_id: Option<&str>,
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
    Ok(false)
}

pub async fn ensure_vector_index(_store: &Store) -> Result<(), StoreError> {
    Err(StoreError::Other(
        "vector search is disabled in the GCS-only runtime".to_string(),
    ))
}

pub async fn reset_search_projection(store: &Store) -> Result<(), StoreError> {
    store.delete_prefix("search-sources/").await?;
    store.delete_prefix("search-chunks/").await?;
    store.delete_prefix("search-bundles/").await?;
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
