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
