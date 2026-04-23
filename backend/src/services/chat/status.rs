use super::*;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ChatRetrievalPlanVisibility {
    intent: ChatQueryIntent,
    label: String,
    budget: usize,
    max_per_video: usize,
    queries: Vec<String>,
    expansion_queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rationale: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    skip_retrieval: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    deep_research: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatToolStatusPayload {
    name: String,
    label: String,
    state: String,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}

impl ChatToolStatusPayload {
    pub(super) fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        state: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            state: state.into(),
            input: input.into(),
            output: None,
        }
    }

    pub(super) fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatStatusPayload {
    stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan: Option<ChatRetrievalPlanVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<ChatToolStatusPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget: Option<crate::models::ChatTurnBudgetSnapshot>,
}

impl ChatStatusPayload {
    pub(super) fn new(stage: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            label: Some(label.into()),
            detail: None,
            decision: None,
            plan: None,
            tool: None,
            budget: None,
        }
    }

    pub(super) fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub(super) fn with_decision(mut self, decision: impl Into<String>) -> Self {
        self.decision = Some(decision.into());
        self
    }

    pub(super) fn with_plan(mut self, plan: ChatRetrievalPlanVisibility) -> Self {
        self.plan = Some(plan);
        self
    }

    pub(super) fn with_tool(mut self, tool: ChatToolStatusPayload) -> Self {
        self.tool = Some(tool);
        self
    }

    pub(super) fn with_budget(mut self, budget: crate::models::ChatTurnBudgetSnapshot) -> Self {
        self.budget = Some(budget);
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChatRetrievalPlan {
    pub(crate) intent: ChatQueryIntent,
    pub(crate) label: String,
    pub(crate) budget: usize,
    pub(crate) max_per_video: usize,
    pub(crate) queries: Vec<String>,
    pub(crate) expansion_queries: Vec<String>,
    pub(crate) focus_terms: Vec<String>,
    pub(crate) channel_focus_ids: Vec<String>,
    pub(crate) video_focus_ids: Vec<String>,
    pub(crate) attributed_preference: bool,
    pub(crate) rationale: Option<String>,
    /// When true, retrieval is skipped and the model answers from conversation history only.
    pub(crate) skip_retrieval: bool,
    /// User requested maximum library coverage for this turn.
    pub(crate) deep_research: bool,
}

impl ChatRetrievalPlan {
    pub(super) fn fallback(prompt: &str, rationale: Option<String>) -> Self {
        let attributed_preference = is_attributed_preference_query(prompt);
        let recent_activity =
            is_recent_activity_query(prompt) && !is_explicit_realtime_status_query(prompt);
        let intent = if recent_activity {
            ChatQueryIntent::RecentActivity
        } else {
            ChatQueryIntent::Synthesis
        };
        let (budget, max_per_video) = if attributed_preference {
            (CHAT_RECOMMENDATION_SOURCE_LIMIT, 3)
        } else {
            match intent {
                ChatQueryIntent::RecentActivity => (CHAT_RECENT_ACTIVITY_SOURCE_LIMIT, 2),
                _ => (CHAT_SYNTHESIS_SOURCE_LIMIT, 4),
            }
        };
        Self {
            intent,
            label: build_plan_label(intent, attributed_preference).to_string(),
            budget,
            max_per_video,
            queries: vec![prompt.trim().to_string()],
            expansion_queries: Vec::new(),
            focus_terms: collect_focus_terms(prompt),
            channel_focus_ids: Vec::new(),
            video_focus_ids: Vec::new(),
            attributed_preference,
            rationale,
            skip_retrieval: false,
            deep_research: false,
        }
    }

    pub(super) fn from_response(prompt: &str, response: ChatQueryPlanResponse) -> Self {
        let planner_intent = response
            .intent
            .as_deref()
            .and_then(ChatQueryIntent::from_str)
            .unwrap_or(ChatQueryIntent::Synthesis);
        let mut intent = planner_intent;
        let attributed_preference = is_attributed_preference_query(prompt);
        let recent_activity =
            is_recent_activity_query(prompt) && !is_explicit_realtime_status_query(prompt);
        if attributed_preference && matches!(intent, ChatQueryIntent::Fact) {
            intent = ChatQueryIntent::Synthesis;
        }
        if recent_activity && !matches!(intent, ChatQueryIntent::Comparison) {
            intent = ChatQueryIntent::RecentActivity;
        }

        let skip_retrieval = response.needs_retrieval == Some(false);

        let (mut budget, mut max_per_video) = match intent {
            ChatQueryIntent::Fact => (CHAT_SOURCE_LIMIT, 3),
            ChatQueryIntent::Synthesis => (CHAT_SYNTHESIS_SOURCE_LIMIT, 4),
            ChatQueryIntent::Pattern => (CHAT_PATTERN_SOURCE_LIMIT, 3),
            ChatQueryIntent::Comparison => (CHAT_COMPARISON_SOURCE_LIMIT, 5),
            ChatQueryIntent::RecentActivity => (CHAT_RECENT_ACTIVITY_SOURCE_LIMIT, 2),
        };
        if attributed_preference && !matches!(intent, ChatQueryIntent::Comparison) {
            budget = budget.max(CHAT_RECOMMENDATION_SOURCE_LIMIT);
            max_per_video = max_per_video.min(3);
        }

        let mut queries = sanitize_queries(response.sub_queries.unwrap_or_default());
        let mut expansion_queries =
            sanitize_queries(response.expansion_queries.unwrap_or_default());

        if queries.is_empty() {
            queries.push(prompt.trim().to_string());
        }

        let heuristic_queries = if attributed_preference {
            recommendation_query_variants(prompt)
        } else {
            heuristic_query_variants(prompt, intent)
        };
        for query in heuristic_queries {
            if queries.len() < CHAT_QUERY_LIMIT_PER_PASS {
                push_unique_query(&mut queries, query);
            } else if expansion_queries.len() < CHAT_QUERY_LIMIT_TOTAL - queries.len() {
                push_unique_query(&mut expansion_queries, query);
            }
        }

        if matches!(intent, ChatQueryIntent::Fact) && !attributed_preference {
            expansion_queries.clear();
            queries.truncate(1);
        } else {
            queries.truncate(CHAT_QUERY_LIMIT_PER_PASS);
            expansion_queries.retain(|query| !queries.contains(query));
            expansion_queries.truncate(CHAT_QUERY_LIMIT_TOTAL.saturating_sub(queries.len()));
        }

        Self {
            intent,
            label: build_plan_label(intent, attributed_preference).to_string(),
            budget,
            max_per_video,
            queries,
            expansion_queries,
            focus_terms: collect_focus_terms(prompt),
            channel_focus_ids: Vec::new(),
            video_focus_ids: Vec::new(),
            attributed_preference,
            rationale: if attributed_preference && matches!(planner_intent, ChatQueryIntent::Fact) {
                Some(
                    "The wording asks for someone’s recommendation/opinion, so the search was widened beyond a narrow fact lookup."
                        .to_string(),
                )
            } else {
                response.rationale.and_then(|value| trim_to_option(&value))
            },
            skip_retrieval,
            deep_research: false,
        }
    }

    pub(super) fn visibility(&self) -> ChatRetrievalPlanVisibility {
        ChatRetrievalPlanVisibility {
            intent: self.intent,
            label: self.label.clone(),
            budget: self.budget,
            max_per_video: self.max_per_video,
            queries: self.queries.clone(),
            expansion_queries: self.expansion_queries.clone(),
            rationale: self.rationale.clone(),
            skip_retrieval: self.skip_retrieval,
            deep_research: self.deep_research,
        }
    }

    pub(super) fn apply_scope(&mut self, scope: &tools::MentionScope) {
        if !scope.has_scope() {
            return;
        }

        if matches!(self.intent, ChatQueryIntent::Comparison) && !scope.video_focus_ids.is_empty() {
            self.channel_focus_ids.clear();
            self.video_focus_ids.clear();

            if let Some(first_query) = self.queries.first_mut() {
                *first_query = scope.scoped_query(first_query);
            } else {
                let scoped = scope.prompt_for_retrieval(&scope.cleaned_prompt);
                if !scoped.trim().is_empty() {
                    self.queries.push(scoped);
                }
            }

            for query in self.queries.iter_mut().skip(1) {
                *query = remove_scoped_video_titles(query, &scope.video_titles);
            }
            for query in &mut self.expansion_queries {
                *query = remove_scoped_video_titles(query, &scope.video_titles);
            }
            self.queries.retain(|query| !query.trim().is_empty());
            self.expansion_queries
                .retain(|query| !query.trim().is_empty());

            if let Some(scope_detail) = scope.scope_detail() {
                let scope_note = format!(
                    "Start from {scope_detail}, but allow comparison searches to widen beyond the scoped video."
                );
                self.rationale = Some(match self.rationale.take() {
                    Some(existing) if !existing.is_empty() => format!("{scope_note} {existing}"),
                    _ => scope_note,
                });
            }
            return;
        }

        self.channel_focus_ids = scope.channel_focus_ids.clone();
        self.video_focus_ids = scope.video_focus_ids.clone();

        self.queries = self
            .queries
            .iter()
            .map(|query| scope.scoped_query(query))
            .filter(|query| !query.trim().is_empty())
            .collect();
        self.expansion_queries = self
            .expansion_queries
            .iter()
            .map(|query| scope.scoped_query(query))
            .filter(|query| !query.trim().is_empty())
            .collect();

        if self.queries.is_empty() {
            let scoped = scope.prompt_for_retrieval(&scope.cleaned_prompt);
            if !scoped.trim().is_empty() {
                self.queries.push(scoped);
            }
        }

        if let Some(scope_detail) = scope.scope_detail() {
            let scope_note = format!("Scoped to {scope_detail}.");
            self.rationale = Some(match self.rationale.take() {
                Some(existing) if !existing.is_empty() => format!("{scope_note} {existing}"),
                _ => scope_note,
            });
        }
    }

    fn queries_per_pass_cap(&self) -> usize {
        if self.deep_research {
            CHAT_DEEP_RESEARCH_QUERIES_PER_PASS
        } else {
            CHAT_QUERY_LIMIT_PER_PASS
        }
    }

    /// Widens retrieval after planner output so this turn uses the app’s maximum excerpt budget and richer query fan-out.
    pub(super) fn apply_deep_research(&mut self, prompt: &str) {
        let prompt = prompt.trim();
        self.deep_research = true;
        self.skip_retrieval = false;
        self.intent = ChatQueryIntent::Pattern;
        self.budget = CHAT_DEEP_RESEARCH_SOURCE_LIMIT;
        self.max_per_video = self.max_per_video.max(4);
        self.label = "Deep research".to_string();

        let mut combined: Vec<String> = Vec::new();
        push_unique_query(&mut combined, prompt.to_string());
        for query in self.queries.iter().chain(self.expansion_queries.iter()) {
            push_unique_query(&mut combined, query.clone());
        }
        let heur = if self.attributed_preference {
            recommendation_query_variants(prompt)
        } else {
            heuristic_query_variants(prompt, ChatQueryIntent::Pattern)
        };
        for query in heur {
            push_unique_query(&mut combined, query);
        }
        push_unique_query(&mut combined, format!("{prompt} themes"));
        push_unique_query(&mut combined, format!("{prompt} overview"));

        self.queries = combined
            .iter()
            .take(CHAT_DEEP_RESEARCH_PRIMARY_QUERIES)
            .cloned()
            .collect();
        self.expansion_queries = combined
            .iter()
            .skip(CHAT_DEEP_RESEARCH_PRIMARY_QUERIES)
            .take(CHAT_DEEP_RESEARCH_EXPANSION_QUERIES)
            .cloned()
            .collect();

        let note = "Deep research: using the maximum excerpt budget for this app and broader search passes.";
        self.rationale = Some(match self.rationale.take() {
            Some(previous) if !previous.is_empty() => format!("{note} {previous}"),
            _ => note.to_string(),
        });
    }

    pub(super) fn queries_for_pass(&self, pass: usize) -> Vec<String> {
        let cap = self.queries_per_pass_cap();
        match pass {
            1 => self.queries.clone(),
            2 => {
                let mut queries = self.expansion_queries.clone();
                if queries.is_empty() {
                    queries = heuristic_expansion_queries(self);
                }
                queries.truncate(cap);
                queries
            }
            3 => {
                let mut extra = heuristic_expansion_queries(self);
                let mut seen: HashSet<String> = self
                    .queries
                    .iter()
                    .chain(self.expansion_queries.iter())
                    .map(|q| q.trim().to_ascii_lowercase())
                    .collect();
                extra.retain(|q| {
                    let key = q.trim().to_ascii_lowercase();
                    if key.is_empty() || seen.contains(&key) {
                        return false;
                    }
                    seen.insert(key);
                    true
                });
                extra.truncate(cap);
                extra
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn supports_second_pass(&self) -> bool {
        !self.queries_for_pass(2).is_empty()
    }

    pub(super) fn supports_third_pass(&self) -> bool {
        self.budget >= 16
            && matches!(
                self.intent,
                ChatQueryIntent::Pattern | ChatQueryIntent::Comparison
            )
            && !self.queries_for_pass(3).is_empty()
    }

    pub(super) fn synthesis_video_cap(&self) -> usize {
        let scaled = (self.budget / 3).max(CHAT_SYNTHESIS_VIDEO_LIMIT);
        scaled.min(24)
    }
}

fn remove_scoped_video_titles(query: &str, video_titles: &[String]) -> String {
    let mut cleaned = query.to_string();
    for title in video_titles {
        cleaned = cleaned.replace(&format!("\"{title}\""), "");
        cleaned = cleaned.replace(title, "");
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Deserialize)]
pub(super) struct ChatQueryPlanResponse {
    pub(super) needs_retrieval: Option<bool>,
    pub(super) intent: Option<String>,
    pub(super) rationale: Option<String>,
    pub(super) sub_queries: Option<Vec<String>>,
    pub(super) expansion_queries: Option<Vec<String>>,
}

impl ChatQueryPlanResponse {
    pub(super) fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "needs_retrieval": { "type": "boolean" },
                "intent": {
                    "type": "string",
                    "enum": ["fact", "synthesis", "pattern", "comparison", "recent_activity"]
                },
                "rationale": { "type": "string" },
                "sub_queries": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "expansion_queries": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": [
                "needs_retrieval",
                "intent",
                "rationale",
                "sub_queries",
                "expansion_queries"
            ]
        })
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ChatToolLoopResponse {
    pub(super) action: Option<String>,
    pub(super) rationale: Option<String>,
    pub(super) tool_name: Option<String>,
    pub(super) search_library_input: Option<tools::SearchLibraryToolInput>,
    pub(super) db_inspect_input: Option<tools::DbInspectToolInput>,
    pub(super) highlight_lookup_input: Option<tools::HighlightLookupToolInput>,
    pub(super) recent_library_activity_input: Option<tools::RecentLibraryActivityToolInput>,
}

impl ChatToolLoopResponse {
    pub(super) fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "respond",
                        "tool_call",
                        "search_library",
                        "db_inspect",
                        "highlight_lookup",
                        "recent_library_activity"
                    ]
                },
                "rationale": { "type": "string" },
                "tool_name": {
                    "anyOf": [
                        {
                            "type": "string",
                            "enum": [
                                "search_library",
                                "db_inspect",
                                "highlight_lookup",
                                "recent_library_activity"
                            ]
                        },
                        { "type": "null" }
                    ]
                },
                "search_library_input": nullable_object_schema(object_schema(
                    serde_json::json!({
                        "query": { "type": "string" },
                        "source": {
                            "type": "string",
                            "enum": ["all", "summary", "transcript"]
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 24
                        }
                    }),
                    &["query", "source", "limit"],
                )),
                "db_inspect_input": nullable_object_schema(object_schema(
                    serde_json::json!({
                        "operation": {
                            "type": "string",
                            "enum": ["count", "list", "breakdown"]
                        },
                        "resource": {
                            "type": "string",
                            "enum": ["summaries", "transcripts", "videos", "channels"]
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 10
                        },
                        "group_by": {
                            "anyOf": [
                                {
                                    "type": "string",
                                    "enum": ["channel"]
                                },
                                { "type": "null" }
                            ]
                        }
                    }),
                    &["operation", "resource", "limit", "group_by"],
                )),
                "highlight_lookup_input": nullable_object_schema(object_schema(
                    serde_json::json!({
                        "query": {
                            "anyOf": [
                                { "type": "string" },
                                { "type": "null" }
                            ]
                        },
                        "video_title": {
                            "anyOf": [
                                { "type": "string" },
                                { "type": "null" }
                            ]
                        },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 20
                        }
                    }),
                    &["query", "video_title", "limit"],
                )),
                "recent_library_activity_input": nullable_object_schema(object_schema(
                    serde_json::json!({
                        "scope": {
                            "type": "string",
                            "enum": ["channel", "video", "library"]
                        },
                        "channel_id": {
                            "anyOf": [
                                { "type": "string" },
                                { "type": "null" }
                            ]
                        },
                        "video_id": {
                            "anyOf": [
                                { "type": "string" },
                                { "type": "null" }
                            ]
                        },
                        "limit_videos": {
                            "type": "integer",
                            "minimum": 3,
                            "maximum": 12
                        },
                        "include_summaries": { "type": "boolean" },
                        "include_transcripts": { "type": "boolean" }
                    }),
                    &[
                        "scope",
                        "channel_id",
                        "video_id",
                        "limit_videos",
                        "include_summaries",
                        "include_transcripts",
                    ],
                ))
            },
            "required": [
                "action",
                "rationale",
                "tool_name",
                "search_library_input",
                "db_inspect_input",
                "highlight_lookup_input",
                "recent_library_activity_input"
            ]
        })
    }
}

fn nullable_object_schema(properties_and_required: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "additionalProperties": false,
                "properties": properties_and_required["properties"].clone(),
                "required": properties_and_required["required"].clone()
            },
            { "type": "null" }
        ]
    })
}

fn object_schema(properties: serde_json::Value, required: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "properties": properties,
        "required": required
    })
}

#[derive(Debug)]
pub(super) struct ToolLoopStepOutcome {
    pub(super) action: ToolLoopAction,
    pub(super) rationale: Option<String>,
}

#[derive(Debug)]
pub(super) enum ToolLoopAction {
    Respond,
    ToolCall(PlannedChatToolCall),
}

#[derive(Debug, Clone)]
pub(super) enum PlannedChatToolCall {
    SearchLibrary(tools::SearchLibraryQuery),
    DbInspect(tools::DbInspectQuery),
    HighlightLookup(tools::HighlightLookupQuery),
    RecentLibraryActivity(tools::RecentLibraryActivityQuery),
}

impl PlannedChatToolCall {
    pub(super) fn tool_name(&self) -> &'static str {
        match self {
            Self::SearchLibrary(_) => "search_library",
            Self::DbInspect(_) => "db_inspect",
            Self::HighlightLookup(_) => "highlight_lookup",
            Self::RecentLibraryActivity(_) => "recent_library_activity",
        }
    }

    pub(super) fn label(&self) -> &'static str {
        match self {
            Self::SearchLibrary(_) => "Library search",
            Self::DbInspect(_) => "Database lookup",
            Self::HighlightLookup(_) => "Saved highlights lookup",
            Self::RecentLibraryActivity(_) => "Recent library activity",
        }
    }

    pub(super) fn input_summary(&self) -> String {
        match self {
            Self::SearchLibrary(query) => describe_search_library_query(query.clone()),
            Self::DbInspect(query) => tools::describe_db_inspect_query(*query),
            Self::HighlightLookup(query) => describe_highlight_lookup_query(query.clone()),
            Self::RecentLibraryActivity(query) => {
                describe_recent_library_activity_query(query.clone())
            }
        }
    }
}
