use super::*;

pub(super) fn format_search_library_tool_output(
    query: &tools::SearchLibraryQuery,
    sources: &[RetrievedChatSource],
) -> String {
    if sources.is_empty() {
        return format!("No grounded excerpts found for \"{}\".", query.query);
    }

    let rows = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            format!(
                "{}. {} / {} / {} - {}",
                index + 1,
                source.source.channel_name,
                source.source.video_title,
                source.source.source_kind.as_str(),
                source.source.snippet
            )
        })
        .collect::<Vec<_>>();

    format!(
        "Found {} excerpt{} for \"{}\":\n{}",
        sources.len(),
        if sources.len() == 1 { "" } else { "s" },
        query.query,
        rows.join("\n")
    )
}

pub(super) fn merge_retrieved_sources(
    existing: &mut Vec<RetrievedChatSource>,
    new_sources: impl IntoIterator<Item = RetrievedChatSource>,
) {
    let mut seen = existing
        .iter()
        .map(|source| source.source.chunk_id.clone())
        .collect::<HashSet<_>>();

    for source in new_sources {
        if seen.insert(source.source.chunk_id.clone()) {
            existing.push(source);
        }
    }
}

#[cfg(test)]
#[path = "utilities_tests.rs"]
mod utilities_tests;
