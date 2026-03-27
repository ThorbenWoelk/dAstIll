use serde::Deserialize;

use crate::db;
use crate::models::{Channel, Summary, Transcript, Video};
use crate::services::search::SearchSourceKind;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DbInspectToolInput {
    pub(crate) operation: Option<String>,
    pub(crate) resource: Option<String>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct SearchLibraryToolInput {
    pub(crate) query: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) limit: Option<usize>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DbInspectQuery {
    pub(crate) operation: DbInspectOperation,
    pub(crate) target: DbInspectTarget,
    pub(crate) limit: usize,
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

    Ok(Some(DbInspectQuery {
        operation,
        target,
        limit: input.limit.unwrap_or(5).clamp(1, 10),
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
    }
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
    }
}

fn format_db_count_answer(target: DbInspectTarget, count: usize) -> String {
    if count == 1 {
        return format!("There is 1 {} in the database.", target.singular());
    }
    format!("There are {count} {} in the database.", target.plural())
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
        DbInspectOperation, DbInspectQuery, DbInspectTarget, DbInspectToolInput,
        SearchLibraryToolInput, build_db_inspect_query, build_search_library_query,
        describe_db_inspect_query,
    };
    use crate::services::search::SearchSourceKind;

    #[test]
    fn builds_count_query_from_valid_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
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
        });
        assert_eq!(description, "List up to 5 channels from the database");
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
}
