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
