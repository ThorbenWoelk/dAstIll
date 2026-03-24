use crate::models::{ChatConversation, ChatRole};

use super::chat::{
    CHAT_HISTORY_LIMIT, CHAT_SYSTEM_PROMPT, CHAT_SYSTEM_PROMPT_CONVERSATION_TURN, ChatRetrievalPlan,
    OllamaRequestMessage, RetrievedChatSource, VideoObservation,
};

const GROUNDING_CITATION_FOOTER: &str = "\n---\nInline citations: Use [1], [2], … in your answer for the same [Source N] as above (one chunk per index). Place brackets right after the phrase they support.";

pub(super) fn synthesis_raw_limit_for_plan(plan: &ChatRetrievalPlan) -> usize {
    plan.budget.min(48).max(8)
}

pub(super) fn build_ollama_messages(
    conversation: &ChatConversation,
    grounding_context: String,
    conversation_only: bool,
) -> Vec<OllamaRequestMessage> {
    let system_primary = if conversation_only {
        CHAT_SYSTEM_PROMPT_CONVERSATION_TURN
    } else {
        CHAT_SYSTEM_PROMPT
    };
    let mut messages = vec![
        OllamaRequestMessage {
            role: "system".to_string(),
            content: system_primary.to_string(),
        },
        OllamaRequestMessage {
            role: "system".to_string(),
            content: grounding_context,
        },
    ];

    let history = conversation
        .messages
        .iter()
        .rev()
        .take(CHAT_HISTORY_LIMIT)
        .cloned()
        .collect::<Vec<_>>();

    for message in history.into_iter().rev() {
        messages.push(OllamaRequestMessage {
            role: match message.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
            }
            .to_string(),
            content: message.content,
        });
    }

    messages
}

pub(super) fn build_grounding_context(retrieved_sources: &[RetrievedChatSource]) -> String {
    let mut context = String::from("Ground-truth excerpts for the next answer only:\n\n");
    for (index, source) in retrieved_sources.iter().enumerate() {
        let source_number = index + 1;
        context.push_str(&format!(
            "[Source {source_number}] Video: {}\nChannel: {}\nType: {}\n",
            source.source.video_title,
            source.source.channel_name,
            source.source.source_kind.as_str(),
        ));
        if let Some(section_title) = &source.source.section_title {
            context.push_str(&format!("Section: {section_title}\n"));
        }
        context.push_str(&format!("Excerpt:\n{}\n\n", source.context_text));
    }
    context.push_str("If these excerpts are not enough, explicitly say so.");
    context.push_str(GROUNDING_CITATION_FOOTER);
    context
}

pub(super) fn build_conversation_only_grounding() -> String {
    "No new library excerpts are attached for this turn. Answer using the conversation history only. If the question clearly requires fresh evidence from the indexed library, say that briefly and suggest the user ask in a way that triggers a library search.".to_string()
}

pub(super) fn build_synthesis_grounding_context(
    prompt: &str,
    plan: &ChatRetrievalPlan,
    retrieved_sources: &[RetrievedChatSource],
    observations: &[VideoObservation],
    raw_excerpt_limit: usize,
) -> String {
    let mut context = format!(
        "Question type: {}\nRetrieval budget: {} excerpts (max {} per video)\nOriginal question: {}\n\n",
        plan.intent.label(),
        plan.budget,
        plan.max_per_video,
        prompt.trim(),
    );
    context.push_str(
        "Intermediate synthesis notes derived only from the raw excerpts below. Treat the raw excerpts as the source of truth.\n\n",
    );

    for (index, observation) in observations.iter().enumerate() {
        let number = index + 1;
        context.push_str(&format!(
            "[Video note {number}] Video: {}\nChannel: {}\n{}\n\n",
            observation.video_title,
            observation.channel_name,
            observation.summary.trim(),
        ));
    }

    context.push_str("Supporting raw excerpts:\n\n");
    for (index, source) in retrieved_sources
        .iter()
        .take(raw_excerpt_limit)
        .enumerate()
    {
        let source_number = index + 1;
        context.push_str(&format!(
            "[Source {source_number}] Video: {}\nChannel: {}\nType: {}\n",
            source.source.video_title,
            source.source.channel_name,
            source.source.source_kind.as_str(),
        ));
        if let Some(section_title) = &source.source.section_title {
            context.push_str(&format!("Section: {section_title}\n"));
        }
        context.push_str(&format!("Excerpt:\n{}\n\n", source.context_text));
    }

    context.push_str(
        "If the notes and excerpts do not fully support an answer, explain the limitation explicitly.",
    );
    context.push_str(GROUNDING_CITATION_FOOTER);
    context
}
