use super::*;

impl ChatService {
    pub(super) async fn generate_and_store_title(
        &self,
        state: AppState,
        conversation_scope_id: String,
        conversation_id: String,
        prompt: String,
    ) {
        let span = logfire::span!(
            "chat.title",
            conversation.id = conversation_id.clone(),
            prompt.chars = prompt.chars().count(),
        );

        async move {
            let started = Instant::now();
            let (generated_title, model_used) = match self.generate_title(&prompt).await {
                Ok(title) => title,
                Err(error) => {
                    tracing::warn!(conversation_id = %conversation_id, error = %error, "chat title generation failed");
                    let _ = finalize_title_generation(
                        &state,
                        &conversation_scope_id,
                        &conversation_id,
                        None,
                    )
                    .await;
                    return;
                }
            };

            if let Err(error) = finalize_title_generation(
                &state,
                &conversation_scope_id,
                &conversation_id,
                Some(generated_title.clone()),
            )
            .await
            {
                tracing::warn!(conversation_id = %conversation_id, error = %error, "failed to persist generated title");
                return;
            }

            tracing::info!(
                conversation_id = %conversation_id,
                model = %model_used,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "chat title generated"
            );
        }
        .instrument(span)
        .await;
    }

    async fn generate_title(&self, prompt: &str) -> Result<(String, String), String> {
        let (response, model_used) = self
            .core
            .prompt_with_fallback(
                "chat_title",
                "Generate a short conversation title in 3 to 5 words. Return only the title text without quotes or punctuation at the end.",
                prompt.trim(),
                crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
            )
            .await
            .map_err(|error| format!("{error:?}"))?;
        let title = Some(sanitize_generated_title(&response))
            .filter(|title| !title.is_empty())
            .ok_or_else(|| "Ollama title response was empty".to_string())?;
        Ok((title, model_used))
    }
}
