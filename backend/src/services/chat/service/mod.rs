use super::*;

impl ChatService {
    pub fn new(core: OllamaCore) -> Self {
        Self {
            core,
            multi_pass_enabled: true,
        }
    }

    pub fn with_multi_pass_enabled(mut self, enabled: bool) -> Self {
        self.multi_pass_enabled = enabled;
        self
    }

    pub fn model(&self) -> &str {
        self.core.model()
    }

    pub fn chat_client_config(&self) -> crate::models::ChatClientConfig {
        let default_model = cloud_models::default_chat_cloud_model_id(self.model());
        let models = cloud_models::CHAT_CLOUD_MODEL_CHOICES
            .iter()
            .map(|entry| crate::models::ChatModelOption {
                id: entry.id.to_string(),
                label: entry.label.to_string(),
            })
            .collect();
        crate::models::ChatClientConfig {
            default_model,
            models,
        }
    }

    pub async fn is_available(&self) -> bool {
        self.core.is_available().await
    }

    pub fn create_conversation(&self, title: Option<String>) -> ChatConversation {
        let now = Utc::now();
        ChatConversation {
            id: generate_chat_id("conv"),
            title: title.and_then(|value| trim_to_option(&value)),
            title_status: ChatTitleStatus::Idle,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }

    pub fn build_user_message(&self, content: &str) -> ChatMessage {
        ChatMessage {
            id: generate_chat_id("msg"),
            role: ChatRole::User,
            content: content.trim().to_string(),
            sources: Vec::new(),
            status: ChatMessageStatus::Completed,
            created_at: Utc::now(),
            model: None,
            prompt_tokens: None,
            completion_tokens: None,
            total_duration_ns: None,
            turn_trace: None,
        }
    }

    pub(crate) fn build_assistant_message(
        &self,
        content: String,
        sources: Vec<ChatSource>,
        status: ChatMessageStatus,
        generation: Option<GenerationMeta>,
    ) -> ChatMessage {
        self.build_assistant_message_with_trace(content, sources, status, generation, None)
    }

    pub(crate) fn build_assistant_message_with_trace(
        &self,
        content: String,
        sources: Vec<ChatSource>,
        status: ChatMessageStatus,
        generation: Option<GenerationMeta>,
        turn_trace: Option<crate::models::ChatTurnTrace>,
    ) -> ChatMessage {
        let content = strip_emoji(&content);
        ChatMessage {
            id: generate_chat_id("msg"),
            role: ChatRole::Assistant,
            content,
            sources,
            status,
            created_at: Utc::now(),
            model: generation.as_ref().map(|meta| meta.model.clone()),
            prompt_tokens: generation.as_ref().and_then(|meta| meta.prompt_tokens),
            completion_tokens: generation.as_ref().and_then(|meta| meta.completion_tokens),
            total_duration_ns: generation.as_ref().and_then(|meta| meta.total_duration_ns),
            turn_trace,
        }
    }

    pub(super) fn assistant_generation_meta(
        &self,
        reply_model: &str,
        terminal: Option<OllamaStreamStats>,
    ) -> GenerationMeta {
        GenerationMeta {
            model: reply_model.to_string(),
            prompt_tokens: terminal.as_ref().and_then(|s| s.prompt_eval_count),
            completion_tokens: terminal.as_ref().and_then(|s| s.eval_count),
            total_duration_ns: terminal.as_ref().and_then(|s| s.total_duration_ns),
        }
    }

    pub fn build_provisional_title(&self, content: &str) -> Option<String> {
        trim_to_option(content).map(|value| limit_text(&value, CHAT_TITLE_MAX_CHARS))
    }

    pub fn start_reply_workflow(&self, request: ReplyWorkflowRequest) {
        let service = self.clone();
        let ReplyWorkflowRequest {
            state,
            conversation,
            access_context,
            conversation_scope_id,
            active_reply_key,
            prompt,
            should_auto_name,
            deep_research,
            reply_model,
            active_reply,
            persist_to_store,
        } = request;
        tokio::spawn(async move {
            if persist_to_store && should_auto_name {
                let naming_service = service.clone();
                let naming_state = state.clone();
                let naming_conversation_scope_id = conversation_scope_id.clone();
                let naming_conversation_id = conversation.id.clone();
                let naming_prompt = prompt.clone();
                tokio::spawn(async move {
                    naming_service
                        .generate_and_store_title(
                            naming_state,
                            naming_conversation_scope_id,
                            naming_conversation_id,
                            naming_prompt,
                        )
                        .await;
                });
            }

            service
                .run_reply_workflow(
                    state,
                    conversation,
                    access_context,
                    conversation_scope_id,
                    active_reply_key,
                    prompt,
                    deep_research,
                    reply_model,
                    active_reply,
                    persist_to_store,
                )
                .await;
        });
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
