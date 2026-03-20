use crate::models::{ChatConversation, ChatRole};

use super::chat::{
    CHAT_HISTORY_LIMIT, CHAT_SYNTHESIS_RAW_SOURCE_LIMIT, CHAT_SYSTEM_PROMPT, ChatRetrievalPlan,
    OllamaRequestMessage, RetrievedChatSource, VideoObservation,
};

pub(super) fn build_ollama_messages(
    conversation: &ChatConversation,
    grounding_context: String,
) -> Vec<OllamaRequestMessage> {
    let mut messages = vec![
        OllamaRequestMessage {
            role: "system".to_string(),
            content: CHAT_SYSTEM_PROMPT.to_string(),
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
    context
}

pub(super) fn build_synthesis_grounding_context(
    prompt: &str,
    plan: &ChatRetrievalPlan,
    retrieved_sources: &[RetrievedChatSource],
    observations: &[VideoObservation],
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
        .take(CHAT_SYNTHESIS_RAW_SOURCE_LIMIT)
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
    context
}
