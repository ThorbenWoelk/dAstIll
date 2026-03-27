use serde::Deserialize;

use crate::db;
use crate::models::{
    Channel, Highlight, HighlightChannelGroup, HighlightVideoGroup, Summary, Transcript, Video,
};
use crate::services::search::SearchSourceKind;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DbInspectToolInput {
    pub(crate) operation: Option<String>,
    pub(crate) resource: Option<String>,
    pub(crate) limit: Option<usize>,
    pub(crate) group_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct SearchLibraryToolInput {
    pub(crate) query: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct HighlightLookupToolInput {
    pub(crate) query: Option<String>,
    pub(crate) video_title: Option<String>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct RecentLibraryActivityToolInput {
    pub(crate) scope: Option<String>,
    pub(crate) channel_id: Option<String>,
    pub(crate) video_id: Option<String>,
    pub(crate) limit_videos: Option<usize>,
    pub(crate) include_summaries: Option<bool>,
    pub(crate) include_transcripts: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbInspectTarget {
    Summaries,
    Transcripts,
    Videos,
    Channels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbInspectOperation {
    Count,
    List,
    Breakdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbGroupBy {
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DbInspectQuery {
    pub(crate) operation: DbInspectOperation,
    pub(crate) target: DbInspectTarget,
    pub(crate) limit: usize,
    pub(crate) group_by: Option<DbGroupBy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbInspectResult {
    pub(crate) summary: String,
    pub(crate) output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchLibraryQuery {
    pub(crate) query: String,
    pub(crate) source_kind: Option<SearchSourceKind>,
    pub(crate) limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HighlightLookupQuery {
    pub(crate) query: Option<String>,
    pub(crate) video_title: Option<String>,
    pub(crate) limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HighlightLookupResult {
    pub(crate) summary: String,
    pub(crate) output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecentLibraryActivityScope {
    Channel,
    Video,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecentLibraryActivityQuery {
    pub(crate) scope: RecentLibraryActivityScope,
    pub(crate) channel_id: Option<String>,
    pub(crate) video_id: Option<String>,
    pub(crate) limit_videos: usize,
    pub(crate) include_summaries: bool,
    pub(crate) include_transcripts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MentionScope {
    pub(crate) cleaned_prompt: String,
    pub(crate) channel_focus_ids: Vec<String>,
    pub(crate) video_focus_ids: Vec<String>,
    pub(crate) channel_names: Vec<String>,
    pub(crate) video_titles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MentionToken {
    start: usize,
    end: usize,
    trigger: char,
    text: String,
}

impl DbInspectTarget {
    fn from_tool_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "summaries" | "summary" => Some(Self::Summaries),
            "transcripts" | "transcript" => Some(Self::Transcripts),
            "videos" | "video" => Some(Self::Videos),
            "channels" | "channel" => Some(Self::Channels),
            _ => None,
        }
    }

    fn singular(self) -> &'static str {
        match self {
            Self::Summaries => "summary",
            Self::Transcripts => "transcript",
            Self::Videos => "video",
            Self::Channels => "channel",
        }
    }

    fn plural(self) -> &'static str {
        match self {
            Self::Summaries => "summaries",
            Self::Transcripts => "transcripts",
            Self::Videos => "videos",
            Self::Channels => "channels",
        }
    }
}

impl DbInspectOperation {
    fn from_tool_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "count" => Some(Self::Count),
            "list" => Some(Self::List),
            "breakdown" => Some(Self::Breakdown),
            _ => None,
        }
    }
}

impl DbGroupBy {
    fn from_tool_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "channel" => Some(Self::Channel),
            _ => None,
        }
    }
}

fn parse_search_source_kind(value: &str) -> Option<Option<SearchSourceKind>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "all" => Some(None),
        "summary" | "summaries" => Some(Some(SearchSourceKind::Summary)),
        "transcript" | "transcripts" => Some(Some(SearchSourceKind::Transcript)),
        _ => None,
    }
}

pub(crate) async fn resolve_mention_scope(
    store: &db::Store,
    input: &str,
) -> Result<MentionScope, db::StoreError> {
    let channels = db::list_channels(store).await?;
    let videos = db::load_all_videos(store).await?;
    Ok(resolve_mention_scope_from_catalog(
        input, &channels, &videos,
    ))
}

pub(crate) fn resolve_mention_scope_from_catalog(
    input: &str,
    channels: &[Channel],
    videos: &[Video],
) -> MentionScope {
    let mentions = extract_mentions(input);
    let mut scope = MentionScope::default();
    for mention in &mentions {
        match mention.trigger {
            '+' => {
                if let Some(video) = resolve_video_mention(&mention.text, videos) {
                    push_unique(&mut scope.video_focus_ids, video.id.clone());
                    push_unique(&mut scope.video_titles, video.title.clone());
                    push_unique(&mut scope.channel_focus_ids, video.channel_id.clone());
                }
            }
            _ => {
                if let Some(channel) = resolve_channel_mention(&mention.text, channels) {
                    push_unique(&mut scope.channel_focus_ids, channel.id.clone());
                    push_unique(&mut scope.channel_names, channel.name.clone());
                    continue;
                }

                if let Some(video) = resolve_video_mention(&mention.text, videos) {
                    push_unique(&mut scope.video_focus_ids, video.id.clone());
                    push_unique(&mut scope.video_titles, video.title.clone());
                    push_unique(&mut scope.channel_focus_ids, video.channel_id.clone());
                }
            }
        }
    }

    scope.cleaned_prompt = if mentions.is_empty() {
        input.trim().to_string()
    } else {
        remove_mention_spans(input, &mentions)
    };
    let cleaned_prompt = scope.cleaned_prompt.clone();
    infer_plain_scope_from_text(&cleaned_prompt, channels, videos, &mut scope);
    scope
}

impl MentionScope {
    pub(crate) fn has_scope(&self) -> bool {
        !self.channel_focus_ids.is_empty() || !self.video_focus_ids.is_empty()
    }

    pub(crate) fn prompt_for_retrieval(&self, original: &str) -> String {
        let base = trim_to_option(&self.cleaned_prompt)
            .or_else(|| trim_to_option(original))
            .unwrap_or_default();
        self.scoped_query(&base)
    }

    pub(crate) fn prompt_for_planner(&self, original: &str) -> String {
        if !self.has_scope() {
            return original.trim().to_string();
        }

        let mut lines = vec![self.prompt_for_retrieval(original)];
        if let Some(detail) = self.scope_detail() {
            lines.push(format!("Scoped mentions: {detail}."));
        }
        lines.join("\n")
    }

    pub(crate) fn scoped_query(&self, base: &str) -> String {
        let mut parts = Vec::new();
        if let Some(value) = trim_to_option(base) {
            parts.push(value);
        }
        for title in &self.video_titles {
            parts.push(format!("\"{title}\""));
        }
        if parts.is_empty() {
            for name in &self.channel_names {
                parts.push(name.clone());
            }
        }
        if parts.is_empty() {
            base.trim().to_string()
        } else {
            parts.join(" ")
        }
    }

    pub(crate) fn scope_detail(&self) -> Option<String> {
        let mut parts = Vec::new();
        if !self.channel_names.is_empty() {
            parts.push(format!("channels: {}", self.channel_names.join(", ")));
        }
        if !self.video_titles.is_empty() {
            parts.push(format!("videos: {}", self.video_titles.join(", ")));
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }
}

pub(crate) fn build_db_inspect_query(
    tool_name: Option<&str>,
    input: Option<DbInspectToolInput>,
) -> Result<Option<DbInspectQuery>, String> {
    if tool_name.is_none() && input.is_none() {
        return Ok(None);
    }

    let tool_name = tool_name.ok_or_else(|| "missing tool name".to_string())?;
    if tool_name.trim() != "db_inspect" {
        return Err(format!("unsupported tool `{tool_name}`"));
    }

    let input = input.ok_or_else(|| "missing db_inspect input".to_string())?;
    let operation_value = input
        .operation
        .as_deref()
        .ok_or_else(|| "missing db_inspect operation".to_string())?;
    let resource_value = input
        .resource
        .as_deref()
        .ok_or_else(|| "missing db_inspect resource".to_string())?;

    let operation = DbInspectOperation::from_tool_value(operation_value)
        .ok_or_else(|| format!("unsupported db_inspect operation `{operation_value}`"))?;
    let target = DbInspectTarget::from_tool_value(resource_value)
        .ok_or_else(|| format!("unsupported db_inspect resource `{resource_value}`"))?;

    let group_by = match input.group_by.as_deref() {
        Some(value) => {
            let parsed = DbGroupBy::from_tool_value(value)
                .ok_or_else(|| format!("unsupported db_inspect group_by `{value}`"))?;
            Some(parsed)
        }
        None => None,
    };

    if operation == DbInspectOperation::Breakdown && group_by.is_none() {
        return Err("db_inspect breakdown requires group_by".to_string());
    }

    Ok(Some(DbInspectQuery {
        operation,
        target,
        limit: input.limit.unwrap_or(5).clamp(1, 10),
        group_by,
    }))
}

pub(crate) fn build_search_library_query(
    tool_name: Option<&str>,
    input: Option<SearchLibraryToolInput>,
) -> Result<Option<SearchLibraryQuery>, String> {
    if tool_name.is_none() && input.is_none() {
        return Ok(None);
    }

    let tool_name = tool_name.ok_or_else(|| "missing tool name".to_string())?;
    if tool_name.trim() != "search_library" {
        return Err(format!("unsupported tool `{tool_name}`"));
    }

    let input = input.ok_or_else(|| "missing search_library input".to_string())?;
    let query = input
        .query
        .as_deref()
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .ok_or_else(|| "missing search_library query".to_string())?;
    let source_kind = match input.source.as_deref() {
        Some(value) => parse_search_source_kind(value)
            .ok_or_else(|| format!("unsupported search_library source `{value}`"))?,
        None => None,
    };

    Ok(Some(SearchLibraryQuery {
        query,
        source_kind,
        limit: input.limit.unwrap_or(8).clamp(1, 24),
    }))
}

pub(crate) fn build_highlight_lookup_query(
    tool_name: Option<&str>,
    input: Option<HighlightLookupToolInput>,
) -> Result<Option<HighlightLookupQuery>, String> {
    if tool_name.is_none() && input.is_none() {
        return Ok(None);
    }

    let tool_name = tool_name.ok_or_else(|| "missing tool name".to_string())?;
    if tool_name.trim() != "highlight_lookup" {
        return Err(format!("unsupported tool `{tool_name}`"));
    }

    let input = input.ok_or_else(|| "missing highlight_lookup input".to_string())?;
    let query = input.query.and_then(|value| trim_to_option(&value));
    let video_title = input.video_title.and_then(|value| trim_to_option(&value));

    if query.is_none() && video_title.is_none() {
        return Err("highlight_lookup requires at least one of query or video_title".to_string());
    }

    Ok(Some(HighlightLookupQuery {
        query,
        video_title,
        limit: input.limit.unwrap_or(8).clamp(1, 20),
    }))
}

pub(crate) fn build_recent_library_activity_query(
    tool_name: Option<&str>,
    input: Option<RecentLibraryActivityToolInput>,
) -> Result<Option<RecentLibraryActivityQuery>, String> {
    if tool_name.is_none() && input.is_none() {
        return Ok(None);
    }

    let tool_name = tool_name.ok_or_else(|| "missing tool name".to_string())?;
    if tool_name.trim() != "recent_library_activity" {
        return Err(format!("unsupported tool `{tool_name}`"));
    }

    let input = input.ok_or_else(|| "missing recent_library_activity input".to_string())?;
    let scope = match input.scope.as_deref().map(str::trim) {
        None | Some("") | Some("channel") => RecentLibraryActivityScope::Channel,
        Some("video") => RecentLibraryActivityScope::Video,
        Some("library") => RecentLibraryActivityScope::Library,
        Some(value) => {
            return Err(format!(
                "unsupported recent_library_activity scope `{value}`"
            ));
        }
    };

    Ok(Some(RecentLibraryActivityQuery {
        scope,
        channel_id: input.channel_id.and_then(|value| trim_to_option(&value)),
        video_id: input.video_id.and_then(|value| trim_to_option(&value)),
        limit_videos: input.limit_videos.unwrap_or(6).clamp(3, 12),
        include_summaries: input.include_summaries.unwrap_or(true),
        include_transcripts: input.include_transcripts.unwrap_or(true),
    }))
}

pub(crate) async fn execute_db_inspect_query(
    store: &db::Store,
    query: DbInspectQuery,
) -> Result<DbInspectResult, db::StoreError> {
    match query.operation {
        DbInspectOperation::Count => {
            let count = match query.target {
                DbInspectTarget::Summaries => db::count_summaries(store).await,
                DbInspectTarget::Transcripts => db::count_transcripts(store).await,
                DbInspectTarget::Videos => db::count_videos(store).await,
                DbInspectTarget::Channels => db::count_channels(store).await,
            }?;
            let output = format_db_count_answer(query.target, count);
            Ok(DbInspectResult {
                summary: describe_db_inspect_query(query),
                output,
            })
        }
        DbInspectOperation::List => execute_list_query(store, query).await,
        DbInspectOperation::Breakdown => {
            let counts = match query.target {
                DbInspectTarget::Summaries => db::summaries_by_channel(store).await,
                DbInspectTarget::Transcripts => db::transcripts_by_channel(store).await,
                DbInspectTarget::Videos => db::videos_by_channel(store).await,
                DbInspectTarget::Channels => {
                    return Err(db::StoreError::Other(
                        "cannot break channels down by channel".to_string(),
                    ));
                }
            }?;
            let output = format_breakdown_by_channel_output(query.target, &counts);
            Ok(DbInspectResult {
                summary: describe_db_inspect_query(query),
                output,
            })
        }
    }
}

pub(crate) async fn execute_highlight_lookup_query(
    store: &db::Store,
    query: HighlightLookupQuery,
) -> Result<HighlightLookupResult, db::StoreError> {
    let groups = db::list_highlights_grouped(store).await?;
    let mut matches = flatten_highlight_groups(&groups)
        .into_iter()
        .filter(|candidate| matches_highlight_query(candidate, &query))
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        highlight_match_score(right, &query)
            .cmp(&highlight_match_score(left, &query))
            .then(right.highlight.created_at.cmp(&left.highlight.created_at))
            .then(right.highlight.id.cmp(&left.highlight.id))
    });
    matches.truncate(query.limit);

    let output = format_highlight_lookup_output(&query, &matches);
    Ok(HighlightLookupResult {
        summary: describe_highlight_lookup_query(&query),
        output,
    })
}

pub(crate) fn describe_db_inspect_query(query: DbInspectQuery) -> String {
    let target = match query.target {
        DbInspectTarget::Summaries => "summaries",
        DbInspectTarget::Transcripts => "transcripts",
        DbInspectTarget::Videos => "videos",
        DbInspectTarget::Channels => "channels",
    };
    match query.operation {
        DbInspectOperation::Count => format!("Count {target} in the database"),
        DbInspectOperation::List => {
            format!("List up to {} {target} from the database", query.limit)
        }
        DbInspectOperation::Breakdown => {
            format!(
                "{} breakdown by channel",
                target[0..1].to_uppercase() + &target[1..]
            )
        }
    }
}

fn format_db_count_answer(target: DbInspectTarget, count: usize) -> String {
    if count == 1 {
        return format!("There is 1 {} in the database.", target.singular());
    }
    format!("There are {count} {} in the database.", target.plural())
}

#[derive(Debug, Clone)]
struct HighlightCandidate<'a> {
    channel: &'a HighlightChannelGroup,
    video: &'a HighlightVideoGroup,
    highlight: &'a Highlight,
}

fn flatten_highlight_groups(groups: &[HighlightChannelGroup]) -> Vec<HighlightCandidate<'_>> {
    let mut candidates = Vec::new();
    for channel in groups {
        for video in &channel.videos {
            for highlight in &video.highlights {
                candidates.push(HighlightCandidate {
                    channel,
                    video,
                    highlight,
                });
            }
        }
    }
    candidates
}

fn matches_highlight_query(
    candidate: &HighlightCandidate<'_>,
    query: &HighlightLookupQuery,
) -> bool {
    let haystack = format!(
        "{} {} {} {} {}",
        candidate.channel.channel_name,
        candidate.video.title,
        candidate.highlight.text,
        candidate.highlight.prefix_context,
        candidate.highlight.suffix_context
    )
    .to_ascii_lowercase();

    let title_matches = query.video_title.as_ref().is_none_or(|value| {
        candidate
            .video
            .title
            .to_ascii_lowercase()
            .contains(&value.to_ascii_lowercase())
    });

    let query_matches = query.query.as_ref().is_none_or(|value| {
        tokenize_query(value)
            .iter()
            .all(|token| haystack.contains(token.as_str()))
    });

    title_matches && query_matches
}

fn highlight_match_score(
    candidate: &HighlightCandidate<'_>,
    query: &HighlightLookupQuery,
) -> usize {
    let haystack = format!(
        "{} {} {} {} {}",
        candidate.channel.channel_name,
        candidate.video.title,
        candidate.highlight.text,
        candidate.highlight.prefix_context,
        candidate.highlight.suffix_context
    )
    .to_ascii_lowercase();
    let mut score = 0;
    if let Some(value) = &query.video_title {
        let value = value.to_ascii_lowercase();
        if candidate.video.title.to_ascii_lowercase().contains(&value) {
            score += 6;
        }
    }
    if let Some(value) = &query.query {
        for token in tokenize_query(value) {
            if haystack.contains(&token) {
                score += 2;
                if candidate
                    .highlight
                    .text
                    .to_ascii_lowercase()
                    .contains(&token)
                {
                    score += 1;
                }
            }
        }
    }
    score
}

fn format_highlight_lookup_output(
    query: &HighlightLookupQuery,
    matches: &[HighlightCandidate<'_>],
) -> String {
    if matches.is_empty() {
        return format!(
            "No saved highlights matched {}.",
            describe_highlight_lookup_scope(query)
        );
    }

    let mut lines = vec![format!(
        "Saved highlights matching {}:",
        describe_highlight_lookup_scope(query)
    )];
    for (index, candidate) in matches.iter().enumerate() {
        let source = match candidate.highlight.source {
            crate::models::HighlightSource::Transcript => "transcript",
            crate::models::HighlightSource::Summary => "summary",
        };
        lines.push(format!(
            "{}. {} / {} / {} highlight: {}",
            index + 1,
            candidate.channel.channel_name,
            candidate.video.title,
            source,
            compact_highlight_text(&candidate.highlight.text)
        ));
    }
    lines.join("\n")
}

fn describe_highlight_lookup_query(query: &HighlightLookupQuery) -> String {
    format!(
        "Look up saved highlights for {}",
        describe_highlight_lookup_scope(query)
    )
}

fn describe_highlight_lookup_scope(query: &HighlightLookupQuery) -> String {
    match (&query.query, &query.video_title) {
        (Some(query_text), Some(video_title)) => {
            format!("query \"{query_text}\" in videos matching \"{video_title}\"")
        }
        (Some(query_text), None) => format!("query \"{query_text}\""),
        (None, Some(video_title)) => format!("videos matching \"{video_title}\""),
        (None, None) => "saved highlights".to_string(),
    }
}

fn compact_highlight_text(input: &str) -> String {
    const MAX_CHARS: usize = 220;
    let compact = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_CHARS {
        compact
    } else {
        let mut clipped = compact.chars().take(MAX_CHARS).collect::<String>();
        clipped.push_str("...");
        clipped
    }
}

fn extract_mentions(input: &str) -> Vec<MentionToken> {
    let mut mentions = Vec::new();
    let mut index = 0;

    while index < input.len() {
        let Some(ch) = input[index..].chars().next() else {
            break;
        };
        if ch != '@' && ch != '+' {
            index += ch.len_utf8();
            continue;
        }

        let parsed = match input[index + 1..].chars().next() {
            Some('"') => extract_quoted_mention(input, index),
            Some('{') => extract_braced_mention(input, index),
            Some(_) => extract_bare_mention(input, index),
            None => None,
        };

        if let Some(token) = parsed {
            index = token.end;
            mentions.push(token);
        } else {
            index += ch.len_utf8();
        }
    }

    mentions
}

fn extract_quoted_mention(input: &str, start: usize) -> Option<MentionToken> {
    let mut cursor = start + 2;
    while cursor < input.len() {
        let ch = input[cursor..].chars().next()?;
        if ch == '"' {
            let text = trim_to_option(&input[start + 2..cursor])?;
            return Some(MentionToken {
                start,
                end: cursor + 1,
                trigger: input[start..].chars().next().unwrap_or('@'),
                text,
            });
        }
        cursor += ch.len_utf8();
    }
    None
}

fn extract_braced_mention(input: &str, start: usize) -> Option<MentionToken> {
    let mut cursor = start + 2;
    while cursor < input.len() {
        let ch = input[cursor..].chars().next()?;
        if ch == '}' {
            let text = trim_to_option(&input[start + 2..cursor])?;
            return Some(MentionToken {
                start,
                end: cursor + 1,
                trigger: input[start..].chars().next().unwrap_or('@'),
                text,
            });
        }
        cursor += ch.len_utf8();
    }
    None
}

fn extract_bare_mention(input: &str, start: usize) -> Option<MentionToken> {
    let mut cursor = start + 1;
    while cursor < input.len() {
        let ch = input[cursor..].chars().next()?;
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')) {
            break;
        }
        cursor += ch.len_utf8();
    }

    let text = trim_to_option(&input[start + 1..cursor])?;
    Some(MentionToken {
        start,
        end: cursor,
        trigger: input[start..].chars().next().unwrap_or('@'),
        text,
    })
}

fn remove_mention_spans(input: &str, mentions: &[MentionToken]) -> String {
    let mut cleaned = String::with_capacity(input.len());
    let mut cursor = 0;
    for mention in mentions {
        if mention.start > cursor {
            cleaned.push_str(&input[cursor..mention.start]);
        }
        cursor = mention.end;
    }
    if cursor < input.len() {
        cleaned.push_str(&input[cursor..]);
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn resolve_channel_mention<'a>(token: &str, channels: &'a [Channel]) -> Option<&'a Channel> {
    resolve_unique_match(token, channels, |channel| {
        let mut haystacks = vec![normalize_lookup_key(&channel.name)];
        if let Some(handle) = &channel.handle {
            haystacks.push(normalize_lookup_key(handle.trim_start_matches('@')));
        }
        haystacks
    })
}

fn resolve_video_mention<'a>(token: &str, videos: &'a [Video]) -> Option<&'a Video> {
    resolve_unique_match(token, videos, |video| {
        vec![normalize_lookup_key(&video.title)]
    })
}

fn infer_plain_scope_from_text(
    input: &str,
    channels: &[Channel],
    videos: &[Video],
    scope: &mut MentionScope,
) {
    if scope.channel_focus_ids.is_empty()
        && let Some(channel) = resolve_plain_channel_reference(input, channels)
    {
        push_unique(&mut scope.channel_focus_ids, channel.id.clone());
        push_unique(&mut scope.channel_names, channel.name.clone());
    }

    if scope.video_focus_ids.is_empty()
        && let Some(video) = resolve_plain_video_reference(input, videos)
    {
        push_unique(&mut scope.video_focus_ids, video.id.clone());
        push_unique(&mut scope.video_titles, video.title.clone());
        push_unique(&mut scope.channel_focus_ids, video.channel_id.clone());
    }
}

fn resolve_plain_channel_reference<'a>(
    input: &str,
    channels: &'a [Channel],
) -> Option<&'a Channel> {
    resolve_unique_phrase_match(input, channels, |channel| {
        let mut haystacks = vec![normalize_lookup_key(&channel.name)];
        if let Some(handle) = &channel.handle {
            haystacks.push(normalize_lookup_key(handle.trim_start_matches('@')));
        }
        haystacks
    })
}

fn resolve_plain_video_reference<'a>(input: &str, videos: &'a [Video]) -> Option<&'a Video> {
    resolve_unique_phrase_match(input, videos, |video| {
        vec![normalize_lookup_key(&video.title)]
    })
}

fn resolve_unique_match<'a, T, F>(token: &str, items: &'a [T], haystacks: F) -> Option<&'a T>
where
    F: Fn(&T) -> Vec<String>,
{
    let needle = normalize_lookup_key(token);
    if needle.is_empty() {
        return None;
    }

    let exact = items
        .iter()
        .filter(|item| haystacks(item).iter().any(|candidate| candidate == &needle))
        .collect::<Vec<_>>();
    if exact.len() == 1 {
        return exact.into_iter().next();
    }
    if !exact.is_empty() {
        return None;
    }

    let fuzzy = items
        .iter()
        .filter(|item| {
            haystacks(item)
                .iter()
                .any(|candidate| candidate.contains(&needle) || needle.contains(candidate))
        })
        .collect::<Vec<_>>();
    (fuzzy.len() == 1).then(|| fuzzy[0])
}

fn resolve_unique_phrase_match<'a, T, F>(input: &str, items: &'a [T], haystacks: F) -> Option<&'a T>
where
    F: Fn(&T) -> Vec<String>,
{
    let normalized_input = normalize_lookup_key(input);
    if normalized_input.is_empty() {
        return None;
    }

    let matches = items
        .iter()
        .filter(|item| {
            haystacks(item)
                .iter()
                .any(|candidate| lookup_phrase_exists(&normalized_input, candidate))
        })
        .collect::<Vec<_>>();
    (matches.len() == 1).then(|| matches[0])
}

fn normalize_lookup_key(input: &str) -> String {
    input
        .trim()
        .trim_start_matches('@')
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn lookup_phrase_exists(input: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let haystack = format!(" {input} ");
    let needle = format!(" {needle} ");
    haystack.contains(&needle)
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn tokenize_query(input: &str) -> Vec<String> {
    input
        .to_ascii_lowercase()
        .split(|char: char| !char.is_ascii_alphanumeric())
        .filter(|token| token.len() > 1)
        .map(ToString::to_string)
        .collect()
}

fn trim_to_option(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

async fn execute_list_query(
    store: &db::Store,
    query: DbInspectQuery,
) -> Result<DbInspectResult, db::StoreError> {
    let output = match query.target {
        DbInspectTarget::Summaries => {
            let mut items: Vec<Summary> = store.load_all("summaries/").await?;
            items.sort_by(|left, right| left.video_id.cmp(&right.video_id));
            let rows = items
                .into_iter()
                .take(query.limit)
                .map(|summary| format!("- {}", summary.video_id))
                .collect::<Vec<_>>();
            format_list_output("summary video ids", rows)
        }
        DbInspectTarget::Transcripts => {
            let mut items: Vec<Transcript> = store.load_all("transcripts/").await?;
            items.sort_by(|left, right| left.video_id.cmp(&right.video_id));
            let rows = items
                .into_iter()
                .take(query.limit)
                .map(|transcript| format!("- {}", transcript.video_id))
                .collect::<Vec<_>>();
            format_list_output("transcript video ids", rows)
        }
        DbInspectTarget::Videos => {
            let mut items: Vec<Video> = store.load_all("videos/").await?;
            items.sort_by(|left, right| right.published_at.cmp(&left.published_at));
            let rows = items
                .into_iter()
                .take(query.limit)
                .map(|video| format!("- {} - {}", video.id, video.title))
                .collect::<Vec<_>>();
            format_list_output("videos", rows)
        }
        DbInspectTarget::Channels => {
            let mut items: Vec<Channel> = store.load_all("channels/").await?;
            items.sort_by(|left, right| left.name.cmp(&right.name));
            let rows = items
                .into_iter()
                .take(query.limit)
                .map(|channel| format!("- {} - {}", channel.id, channel.name))
                .collect::<Vec<_>>();
            format_list_output("channels", rows)
        }
    };

    Ok(DbInspectResult {
        summary: describe_db_inspect_query(query),
        output,
    })
}

fn format_breakdown_by_channel_output(
    target: DbInspectTarget,
    counts: &[(String, usize)],
) -> String {
    if counts.is_empty() {
        return format!("No {} found in the database.", target.plural());
    }
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    let rows = counts
        .iter()
        .map(|(name, count)| format!("- {name}: {count}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{} breakdown by channel (total {total}):\n{rows}",
        target.plural()[0..1].to_uppercase() + &target.plural()[1..]
    )
}

fn format_list_output(label: &str, rows: Vec<String>) -> String {
    if rows.is_empty() {
        return format!("No {label} found in the database.");
    }
    format!(
        "Here are the first {} {label} in the database:\n{}",
        rows.len(),
        rows.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        DbGroupBy, DbInspectOperation, DbInspectQuery, DbInspectTarget, DbInspectToolInput,
        HighlightLookupQuery, HighlightLookupToolInput, RecentLibraryActivityScope,
        RecentLibraryActivityToolInput, SearchLibraryToolInput, build_db_inspect_query,
        build_highlight_lookup_query, build_recent_library_activity_query,
        build_search_library_query, describe_db_inspect_query, describe_highlight_lookup_query,
        resolve_mention_scope_from_catalog,
    };
    use crate::models::{Channel, ContentStatus, Video};
    use crate::services::search::SearchSourceKind;

    #[test]
    fn builds_count_query_from_valid_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::Count);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(query.limit, 5);
    }

    #[test]
    fn clamps_list_limit_from_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("list".to_string()),
                resource: Some("videos".to_string()),
                limit: Some(99),
                group_by: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::List);
        assert_eq!(query.target, DbInspectTarget::Videos);
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn rejects_unknown_tool_name() {
        let error = build_db_inspect_query(
            Some("search"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("unknown tool should be rejected");

        assert!(error.contains("unsupported tool"));
    }

    #[test]
    fn rejects_unknown_resource() {
        let error = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("search_sources".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("unknown resource should be rejected");

        assert!(error.contains("unsupported db_inspect resource"));
    }

    #[test]
    fn describe_query_is_human_readable() {
        let description = describe_db_inspect_query(DbInspectQuery {
            operation: DbInspectOperation::List,
            target: DbInspectTarget::Channels,
            limit: 5,
            group_by: None,
        });
        assert_eq!(description, "List up to 5 channels from the database");
    }

    #[test]
    fn builds_breakdown_query_from_valid_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("breakdown".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: Some("channel".to_string()),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::Breakdown);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(query.group_by, Some(DbGroupBy::Channel));
    }

    #[test]
    fn rejects_breakdown_without_group_by() {
        let error = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("breakdown".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("breakdown without group_by should be rejected");

        assert!(error.contains("requires group_by"));
    }

    #[test]
    fn builds_search_library_query_from_valid_tool_request() {
        let query = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("ownership model".to_string()),
                source: Some("summary".to_string()),
                limit: Some(4),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.query, "ownership model");
        assert_eq!(query.source_kind, Some(SearchSourceKind::Summary));
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn search_library_defaults_to_all_sources_and_clamps_limit() {
        let query = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("rust vector search".to_string()),
                source: None,
                limit: Some(99),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.source_kind, None);
        assert_eq!(query.limit, 24);
    }

    #[test]
    fn search_library_rejects_unknown_source() {
        let error = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("rust vector search".to_string()),
                source: Some("video".to_string()),
                limit: None,
            }),
        )
        .expect_err("invalid source should be rejected");

        assert!(error.contains("unsupported search_library source"));
    }

    #[test]
    fn builds_highlight_lookup_query_from_valid_request() {
        let query = build_highlight_lookup_query(
            Some("highlight_lookup"),
            Some(HighlightLookupToolInput {
                query: Some("prototype-first".to_string()),
                video_title: Some("Theo".to_string()),
                limit: Some(4),
            }),
        )
        .expect("valid request")
        .expect("query should be built");

        assert_eq!(query.query.as_deref(), Some("prototype-first"));
        assert_eq!(query.video_title.as_deref(), Some("Theo"));
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn highlight_lookup_requires_query_or_video_title() {
        let error = build_highlight_lookup_query(
            Some("highlight_lookup"),
            Some(HighlightLookupToolInput {
                query: Some("   ".to_string()),
                video_title: None,
                limit: None,
            }),
        )
        .expect_err("empty highlight lookup request should fail");

        assert!(error.contains("requires at least one of query or video_title"));
    }

    #[test]
    fn highlight_lookup_description_is_human_readable() {
        let description = describe_highlight_lookup_query(&HighlightLookupQuery {
            query: Some("agent".to_string()),
            video_title: None,
            limit: 5,
        });

        assert_eq!(description, "Look up saved highlights for query \"agent\"");
    }

    #[test]
    fn resolves_bare_channel_mentions_into_scope() {
        let channels = vec![sample_channel("chan_1", "Theo", Some("@theo"))];
        let videos = vec![sample_video("vid_1", "chan_1", "Vector Search Guide")];

        let scope = resolve_mention_scope_from_catalog(
            "What does @theo recommend for databases?",
            &channels,
            &videos,
        );

        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(scope.channel_names, vec!["Theo".to_string()]);
        assert_eq!(scope.cleaned_prompt, "What does recommend for databases?");
    }

    #[test]
    fn resolves_quoted_video_mentions_into_scope() {
        let channels = vec![sample_channel("chan_1", "Theo", Some("@theo"))];
        let videos = vec![sample_video("vid_1", "chan_1", "Rust Search Deep Dive")];

        let scope = resolve_mention_scope_from_catalog(
            "Summarize @\"Rust Search Deep Dive\" in three bullets",
            &channels,
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(
            scope.video_titles,
            vec!["Rust Search Deep Dive".to_string()]
        );
        assert_eq!(
            scope.prompt_for_retrieval("Summarize @\"Rust Search Deep Dive\" in three bullets"),
            "Summarize in three bullets \"Rust Search Deep Dive\""
        );
    }

    #[test]
    fn plus_mentions_scope_videos_only() {
        let channels = vec![sample_channel(
            "chan_1",
            "HealthyGamerGG",
            Some("@healthygamergg"),
        )];
        let videos = vec![sample_video(
            "vid_1",
            "chan_1",
            "Why Effort Alone Doesn’t Lead to Change",
        )];

        let scope = resolve_mention_scope_from_catalog(
            "Summarize +{Why Effort Alone Doesn’t Lead to Change}",
            &channels,
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert!(scope.channel_names.is_empty());
        assert_eq!(
            scope.prompt_for_retrieval("Summarize +{Why Effort Alone Doesn’t Lead to Change}"),
            "Summarize \"Why Effort Alone Doesn’t Lead to Change\""
        );
    }

    #[test]
    fn plain_channel_reference_resolves_scope_when_unambiguous() {
        let channels = vec![
            sample_channel("chan_1", "HealthyGamerGG", Some("@healthygamergg")),
            sample_channel("chan_2", "Theo", Some("@theo")),
        ];
        let scope = resolve_mention_scope_from_catalog(
            "What is HealthyGamerGG doing lately?",
            &channels,
            &[],
        );

        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(scope.channel_names, vec!["HealthyGamerGG".to_string()]);
        assert_eq!(
            scope.cleaned_prompt,
            "What is HealthyGamerGG doing lately?".to_string()
        );
    }

    #[test]
    fn plain_video_reference_resolves_scope_when_unambiguous() {
        let videos = vec![sample_video(
            "vid_1",
            "chan_1",
            "Why Effort Alone Doesn’t Lead to Change",
        )];
        let scope = resolve_mention_scope_from_catalog(
            "Summarize Why Effort Alone Doesn’t Lead to Change",
            &[],
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert_eq!(
            scope.video_titles,
            vec!["Why Effort Alone Doesn’t Lead to Change".to_string()]
        );
    }

    #[test]
    fn builds_recent_library_activity_query_with_defaults() {
        let query = build_recent_library_activity_query(
            Some("recent_library_activity"),
            Some(RecentLibraryActivityToolInput {
                scope: Some("channel".to_string()),
                channel_id: None,
                video_id: None,
                limit_videos: None,
                include_summaries: None,
                include_transcripts: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.scope, RecentLibraryActivityScope::Channel);
        assert_eq!(query.limit_videos, 6);
        assert!(query.include_summaries);
        assert!(query.include_transcripts);
    }

    fn sample_channel(id: &str, name: &str, handle: Option<&str>) -> Channel {
        Channel {
            id: id.to_string(),
            handle: handle.map(str::to_string),
            name: name.to_string(),
            thumbnail_url: None,
            added_at: chrono::Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        }
    }

    fn sample_video(id: &str, channel_id: &str, title: &str) -> Video {
        Video {
            id: id.to_string(),
            channel_id: channel_id.to_string(),
            title: title.to_string(),
            thumbnail_url: None,
            published_at: chrono::Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }
}
