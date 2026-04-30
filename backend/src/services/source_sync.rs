use chrono::Utc;

use crate::db::{self, SourceProfileRecord, StoreError};
use crate::models::{
    Channel, ContentSource, ContentStatus, TranscriptRenderMode, Video, VideoInfo,
};
use crate::state::AppState;

use super::{OpenAlexPublicationMaterial, PodcastEpisodeMaterial};

fn compatibility_channel_from_source(source: &ContentSource) -> Channel {
    let now = Utc::now();
    Channel {
        id: source.id.clone(),
        handle: source.handle.clone().or(source.subtitle.clone()),
        name: source.title.clone(),
        thumbnail_url: source.thumbnail_url.clone(),
        added_at: now,
        earliest_sync_date: Some(db::default_earliest_sync_date_floor(now)),
        earliest_sync_date_user_set: false,
    }
}

fn upsert_compat_video<'a>(
    store: &'a db::Store,
    video: &'a Video,
) -> impl std::future::Future<Output = Result<(), StoreError>> + 'a {
    async move {
        let _ = db::insert_video(store, video).await?;
        Ok(())
    }
}

fn abstract_video(source: &ContentSource, material: &OpenAlexPublicationMaterial) -> Video {
    Video {
        id: material.item.id.clone(),
        channel_id: source.id.clone(),
        title: material.item.title.clone(),
        thumbnail_url: None,
        published_at: material.item.published_at.unwrap_or_else(Utc::now),
        is_short: false,
        transcript_status: if material.abstract_text.is_some() {
            ContentStatus::Ready
        } else {
            ContentStatus::Pending
        },
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

fn podcast_video(source: &ContentSource, material: &PodcastEpisodeMaterial) -> Video {
    Video {
        id: material.item.id.clone(),
        channel_id: source.id.clone(),
        title: material.item.title.clone(),
        thumbnail_url: material.item.thumbnail_url.clone(),
        published_at: material.item.published_at.unwrap_or_else(Utc::now),
        is_short: false,
        transcript_status: if material.transcript_text.is_some() {
            ContentStatus::Ready
        } else {
            ContentStatus::Pending
        },
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

fn website_video(material: &crate::services::website::WebsitePageMaterial) -> Video {
    Video {
        id: material.item.id.clone(),
        channel_id: material.source.id.clone(),
        title: material.item.title.clone(),
        thumbnail_url: None,
        published_at: material.item.published_at.unwrap_or(material.published_at),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

async fn upsert_video_info(store: &db::Store, info: &VideoInfo) -> Result<(), StoreError> {
    db::upsert_video_info(store, info).await
}

fn normalized_text_for_match(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn transcript_matches_show_notes(transcript: &crate::models::Transcript, show_notes: &str) -> bool {
    let expected = normalized_text_for_match(show_notes);
    if expected.is_empty() {
        return false;
    }

    [
        transcript.raw_text.as_deref(),
        transcript.formatted_markdown.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|content| normalized_text_for_match(content) == expected)
}

async fn remove_stale_podcast_show_notes_transcript(
    state: &AppState,
    channel_id: &str,
    video_id: &str,
    show_notes: Option<&str>,
) -> Result<(), StoreError> {
    let Some(show_notes) = show_notes else {
        return Ok(());
    };

    let Some(transcript) = db::get_transcript(&state.db, video_id).await? else {
        return Ok(());
    };

    if !transcript_matches_show_notes(&transcript, show_notes) {
        return Ok(());
    }

    tracing::warn!(
        video_id = %video_id,
        "removing podcast show notes cached as transcript"
    );
    db::delete_transcript(&state.db, video_id).await?;
    db::delete_summary(&state.db, video_id).await?;
    db::reset_summary_auto_regen_attempts(&state.db, video_id).await?;
    db::update_video_transcript_status(&state.db, video_id, ContentStatus::Pending).await?;
    db::update_video_summary_status(&state.db, video_id, ContentStatus::Pending).await?;
    state.read_cache.evict_channel(channel_id).await;
    Ok(())
}

pub async fn persist_source_profile_and_channel(
    store: &db::Store,
    profile: &SourceProfileRecord,
) -> Result<Channel, StoreError> {
    let channel = compatibility_channel_from_source(&profile.source);
    db::insert_channel(store, &channel).await?;
    db::put_source_profile(store, profile).await?;
    Ok(channel)
}

pub async fn sync_source_profile(
    state: &AppState,
    profile: &SourceProfileRecord,
) -> Result<usize, String> {
    match profile.source.provider {
        crate::models::ProviderKind::OpenAlex => {
            let query = profile
                .openalex_query
                .clone()
                .ok_or_else(|| "openalex source missing structured query".to_string())?;
            let materials = state
                .openalex
                .sync_query_source_materials(&profile.source, &query)
                .await
                .map_err(|err| err.to_string())?;
            for material in &materials {
                let video = abstract_video(&profile.source, material);
                upsert_compat_video(&state.db, &video)
                    .await
                    .map_err(|err| err.to_string())?;
                if let Some(abstract_text) = material.abstract_text.as_deref() {
                    db::save_manual_transcript(
                        &state.db,
                        &video.id,
                        abstract_text,
                        TranscriptRenderMode::PlainText,
                    )
                    .await
                    .map_err(|err| err.to_string())?;
                }
                upsert_video_info(
                    &state.db,
                    &VideoInfo {
                        video_id: video.id.clone(),
                        watch_url: material.watch_url.clone(),
                        title: video.title.clone(),
                        description: material.description.clone(),
                        thumbnail_url: None,
                        channel_name: Some(profile.source.title.clone()),
                        channel_id: Some(profile.source.id.clone()),
                        published_at: Some(video.published_at),
                        duration_iso8601: None,
                        duration_seconds: None,
                        view_count: None,
                    },
                )
                .await
                .map_err(|err| err.to_string())?;
            }
            Ok(materials.len())
        }
        crate::models::ProviderKind::PodcastRss => {
            let materials = state
                .podcast_feed
                .sync_feed_source_materials(&profile.source)
                .await
                .map_err(|err| err.to_string())?;
            for material in &materials {
                let video = podcast_video(&profile.source, material);
                upsert_compat_video(&state.db, &video)
                    .await
                    .map_err(|err| err.to_string())?;
                if let Some(audio_asset) = material.audio_asset.as_ref() {
                    db::upsert_media_asset(&state.db, audio_asset)
                        .await
                        .map_err(|err| err.to_string())?;
                }
                remove_stale_podcast_show_notes_transcript(
                    state,
                    &video.channel_id,
                    &video.id,
                    material.show_notes.as_deref(),
                )
                .await
                .map_err(|err| err.to_string())?;
                if let Some(transcript_text) = material.transcript_text.as_deref() {
                    db::save_manual_transcript(
                        &state.db,
                        &video.id,
                        transcript_text,
                        TranscriptRenderMode::PlainText,
                    )
                    .await
                    .map_err(|err| err.to_string())?;
                }
                upsert_video_info(
                    &state.db,
                    &VideoInfo {
                        video_id: video.id.clone(),
                        watch_url: material.watch_url.clone(),
                        title: video.title.clone(),
                        description: material.description.clone(),
                        thumbnail_url: video.thumbnail_url.clone(),
                        channel_name: Some(profile.source.title.clone()),
                        channel_id: Some(profile.source.id.clone()),
                        published_at: Some(video.published_at),
                        duration_iso8601: None,
                        duration_seconds: None,
                        view_count: None,
                    },
                )
                .await
                .map_err(|err| err.to_string())?;
            }
            Ok(materials.len())
        }
        crate::models::ProviderKind::Website => {
            let page_url = profile
                .source
                .subtitle
                .as_deref()
                .ok_or_else(|| "website source missing page URL".to_string())?;
            let material = state
                .website
                .resolve_page(page_url)
                .await
                .map_err(|err| err.to_string())?;
            let video = website_video(&material);
            upsert_compat_video(&state.db, &video)
                .await
                .map_err(|err| err.to_string())?;
            db::save_manual_transcript(
                &state.db,
                &video.id,
                &material.text_content,
                TranscriptRenderMode::PlainText,
            )
            .await
            .map_err(|err| err.to_string())?;
            upsert_video_info(
                &state.db,
                &VideoInfo {
                    video_id: video.id.clone(),
                    watch_url: material.page_url.clone(),
                    title: material.title.clone(),
                    description: material.excerpt.clone(),
                    thumbnail_url: None,
                    channel_name: Some(profile.source.title.clone()),
                    channel_id: Some(profile.source.id.clone()),
                    published_at: Some(video.published_at),
                    duration_iso8601: None,
                    duration_seconds: None,
                    view_count: None,
                },
            )
            .await
            .map_err(|err| err.to_string())?;
            Ok(1)
        }
        crate::models::ProviderKind::YouTube => Ok(0),
        other => Err(format!("provider {other:?} sync not implemented")),
    }
}

#[cfg(test)]
#[path = "source_sync_tests.rs"]
mod source_sync_tests;
