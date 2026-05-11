use super::chat::{CHAT_QUERY_LIMIT_TOTAL, ChatQueryIntent, ChatRetrievalPlan};

pub(super) fn preference_signal_score(text: &str, focus_terms: &[String]) -> f32 {
    let normalized = normalize_for_matching(text);
    let preference_hits = preference_signal_terms()
        .iter()
        .filter(|term| normalized.contains(**term))
        .count();
    if preference_hits == 0 {
        return 0.0;
    }

    let focus_hits = focus_terms
        .iter()
        .filter(|term| normalized.contains(term.as_str()))
        .count();
    if !focus_terms.is_empty() && focus_hits == 0 {
        return 0.0;
    }

    0.12 + (preference_hits.min(2) as f32 * 0.07) + (focus_hits.min(2) as f32 * 0.04)
}

pub(super) fn build_plan_label(
    intent: ChatQueryIntent,
    attributed_preference: bool,
) -> &'static str {
    if attributed_preference {
        "Recommendation lookup"
    } else {
        match intent {
            ChatQueryIntent::Fact => "Focused lookup",
            ChatQueryIntent::Synthesis => "Broader synthesis",
            ChatQueryIntent::Pattern => "Pattern scan",
            ChatQueryIntent::Comparison => "Comparison scan",
            ChatQueryIntent::RecentActivity => "Recent activity scan",
        }
    }
}

pub(super) fn is_attributed_preference_query(prompt: &str) -> bool {
    let normalized = normalize_for_matching(prompt);
    let has_attribution = normalized.contains("according to ")
        || normalized.starts_with("what does ")
        || normalized.starts_with("what do ")
        || normalized.contains(" does ")
            && (normalized.contains(" think ")
                || normalized.contains(" use ")
                || normalized.contains(" prefer ")
                || normalized.contains(" recommend "));
    let has_preference = preference_signal_terms()
        .iter()
        .any(|term| normalized.contains(term));
    has_attribution && has_preference
}

pub(super) fn is_comparison_query(prompt: &str) -> bool {
    let normalized = normalize_for_matching(prompt);
    [
        "compare",
        "comparison",
        "different angle",
        "different angles",
        "same topic",
        "same subject",
        "same subjects",
        "disagree",
        "disagreement",
        "aligned",
        "alignment",
        "similar",
        "closest in theme",
        "counterargument",
        "challenges",
        "challenge this topic",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

pub(super) fn is_creator_stance_query(prompt: &str) -> bool {
    let normalized = normalize_for_matching(prompt);
    normalized.contains("what does this creator think about")
        || normalized.contains("what does this channel think about")
        || normalized.contains("what do they think about")
        || normalized.contains("what changed in this creator s position")
        || normalized.contains("how does this creator feel about")
        || normalized.contains("how does this channel feel about")
}

pub(super) fn recommendation_query_variants(prompt: &str) -> Vec<String> {
    let subject = extract_subject_phrase(prompt);
    let focus = collect_focus_terms(prompt).join(" ");
    let mut queries = Vec::new();

    if !subject.is_empty() && !focus.is_empty() {
        queries.push(format!("{subject} {focus} recommendation"));
        queries.push(format!("{subject} favorite {focus}"));
        queries.push(format!("{subject} preferred {focus}"));
        queries.push(format!("{subject} {focus} opinion"));
        queries.push(format!("{subject} {focus} use"));
    } else if !focus.is_empty() {
        queries.push(format!("{focus} recommendation"));
        queries.push(format!("best {focus}"));
        queries.push(format!("favorite {focus}"));
    }

    queries
}

pub(super) fn collect_focus_terms(prompt: &str) -> Vec<String> {
    let subject_terms = tokenize_for_matching(&extract_subject_phrase(prompt));
    tokenize_for_matching(prompt)
        .into_iter()
        .filter(|token| token.len() > 2)
        .filter(|token| !is_query_stopword(token))
        .filter(|token| !subject_terms.contains(token))
        .take(4)
        .collect()
}

pub(super) fn sanitize_queries(queries: Vec<String>) -> Vec<String> {
    let mut sanitized = Vec::new();
    for query in queries {
        push_unique_query(&mut sanitized, query);
        if sanitized.len() >= CHAT_QUERY_LIMIT_TOTAL {
            break;
        }
    }
    sanitized
}

pub(super) fn heuristic_query_variants(prompt: &str, intent: ChatQueryIntent) -> Vec<String> {
    let prompt = prompt.trim();
    if let Some(queries) = alignment_query_variants(prompt) {
        return queries;
    }
    if let Some(queries) = creator_stance_query_variants(prompt) {
        return queries;
    }
    if let Some(queries) = deep_dive_query_variants(prompt) {
        return queries;
    }
    if let Some(queries) = counterargument_query_variants(prompt) {
        return queries;
    }
    if let Some(queries) = procedural_query_variants(prompt) {
        return queries;
    }

    match intent {
        ChatQueryIntent::Fact => Vec::new(),
        ChatQueryIntent::Synthesis => vec![format!("{prompt} overview")],
        ChatQueryIntent::Pattern => vec![
            format!("{prompt} speaking style"),
            format!("{prompt} rhetoric examples"),
            format!("{prompt} tone and phrasing"),
        ],
        ChatQueryIntent::Comparison => vec![
            format!("{prompt} differences"),
            format!("{prompt} similarities"),
            format!("{prompt} contrasting viewpoints"),
        ],
        ChatQueryIntent::RecentActivity => {
            let subject = extract_subject_phrase(prompt);
            if subject.is_empty() {
                vec![format!("{prompt} latest videos")]
            } else {
                vec![
                    format!("{subject} latest videos"),
                    format!("{subject} recent uploads"),
                ]
            }
        }
    }
}

fn alignment_query_variants(prompt: &str) -> Option<Vec<String>> {
    let normalized = normalize_for_matching(prompt);
    let is_alignment = normalized.contains("summary")
        && normalized.contains("transcript")
        && (normalized.contains("supports")
            || normalized.contains("miss")
            || normalized.contains("reliable")
            || normalized.contains("evidence"));
    if !is_alignment {
        return None;
    }

    Some(vec![
        "summary transcript evidence support".to_string(),
        "summary transcript omission mismatch".to_string(),
        "summary reliability transcript evidence".to_string(),
    ])
}

fn creator_stance_query_variants(prompt: &str) -> Option<Vec<String>> {
    if !is_creator_stance_query(prompt) {
        return None;
    }

    let focus = collect_focus_terms(prompt).join(" ");
    if focus.is_empty() {
        return Some(vec![
            "opinion stance reaction".to_string(),
            "support criticism concern".to_string(),
        ]);
    }

    Some(vec![
        format!("{focus} opinion"),
        format!("{focus} stance reaction"),
        format!("{focus} criticism support"),
    ])
}

fn deep_dive_query_variants(prompt: &str) -> Option<Vec<String>> {
    let normalized = normalize_for_matching(prompt);
    let asks_for_depth = normalized.contains("deepest dive")
        || normalized.contains("deep dive")
        || normalized.contains("deeply")
        || normalized.contains("in depth")
        || normalized.contains("comprehensive");
    if !asks_for_depth {
        return None;
    }

    Some(vec![
        "deep dive comprehensive detailed explanation".to_string(),
        "in depth technical details thorough overview".to_string(),
        "comprehensive analysis examples details".to_string(),
    ])
}

fn counterargument_query_variants(prompt: &str) -> Option<Vec<String>> {
    let normalized = normalize_for_matching(prompt);
    let is_counterargument = normalized.contains("counterargument")
        || normalized.contains("challenge")
        || normalized.contains("challenges")
        || normalized.contains("disagree")
        || normalized.contains("disagreement")
        || normalized.contains("skeptical");
    if !is_counterargument {
        return None;
    }

    Some(vec![
        "counterargument skeptical disagreement".to_string(),
        "opposing argument criticism limitation".to_string(),
        "concern downside tradeoff challenge".to_string(),
    ])
}

fn procedural_query_variants(prompt: &str) -> Option<Vec<String>> {
    let normalized = normalize_for_matching(prompt);
    let is_procedural = normalized.contains("step by step")
        || normalized.contains("instructions")
        || normalized.contains("instruction")
        || normalized.contains("procedure")
        || normalized.contains("procedural")
        || normalized.contains("walkthrough")
        || normalized.contains("tutorial");
    if !is_procedural {
        return None;
    }

    Some(vec![
        "step instructions tutorial setup".to_string(),
        "walkthrough guide how to implementation".to_string(),
        "procedure steps practical setup".to_string(),
    ])
}

pub(super) fn heuristic_expansion_queries(plan: &ChatRetrievalPlan) -> Vec<String> {
    let base_queries = if plan.attributed_preference {
        recommendation_query_variants(plan.queries.first().map(String::as_str).unwrap_or_default())
    } else {
        heuristic_query_variants(
            plan.queries.first().map(String::as_str).unwrap_or_default(),
            plan.intent,
        )
    };

    base_queries
        .into_iter()
        .filter(|query| {
            !plan
                .queries
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(query))
        })
        .collect()
}

fn extract_subject_phrase(prompt: &str) -> String {
    let tokens = tokenize_for_matching(prompt);

    if let Some(index) = tokens.iter().position(|token| token == "according")
        && tokens.get(index + 1).is_some_and(|token| token == "to")
    {
        let subject = tokens
            .iter()
            .skip(index + 2)
            .take_while(|token| !is_boundary_token(token))
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    if tokens.starts_with(&["what".to_string(), "does".to_string()]) {
        let subject = tokens
            .iter()
            .skip(2)
            .take_while(|token| {
                !matches!(
                    token.as_str(),
                    "think" | "recommend" | "prefer" | "use" | "say"
                )
            })
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    if tokens.starts_with(&["what".to_string(), "is".to_string()]) {
        let subject = tokens
            .iter()
            .skip(2)
            .take_while(|token| {
                !matches!(
                    token.as_str(),
                    "doing" | "focused" | "working" | "talking" | "saying" | "covering"
                )
            })
            .take(6)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    if tokens.starts_with(&["what".to_string(), "has".to_string()]) {
        let subject = tokens
            .iter()
            .skip(2)
            .take_while(|token| !matches!(token.as_str(), "been" | "recently" | "lately"))
            .take(6)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    String::new()
}

fn tokenize_for_matching(input: &str) -> Vec<String> {
    normalize_for_matching(input)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn normalize_for_matching(input: &str) -> String {
    input
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char.is_whitespace() {
                char.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect()
}

fn is_boundary_token(token: &str) -> bool {
    matches!(
        token,
        "about" | "for" | "on" | "in" | "with" | "and" | "or" | "vs" | "versus"
    )
}

fn is_query_stopword(token: &str) -> bool {
    matches!(
        token,
        "what"
            | "which"
            | "who"
            | "does"
            | "do"
            | "did"
            | "is"
            | "are"
            | "the"
            | "a"
            | "an"
            | "according"
            | "to"
            | "think"
            | "opinion"
            | "best"
            | "favorite"
            | "favourite"
            | "prefer"
            | "preferred"
            | "recommend"
            | "recommendation"
            | "use"
            | "uses"
            | "should"
            | "recent"
            | "recently"
            | "lately"
            | "latest"
            | "currently"
            | "nowadays"
            | "days"
            | "i"
            | "we"
            | "me"
    )
}

fn preference_signal_terms() -> &'static [&'static str] {
    &[
        " best ",
        " favorite ",
        " favourite ",
        " prefer ",
        " preferred ",
        " recommend ",
        " recommendation ",
        " would use ",
        " i use ",
        " i choose ",
        " go with ",
    ]
}

pub(super) fn push_unique_query(queries: &mut Vec<String>, query: impl Into<String>) {
    let query = query.into();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }
    if queries
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(trimmed))
    {
        return;
    }
    queries.push(trimmed.to_string());
}
