//! Curated Ollama **cloud** model tags for chat UI (`:cloud` or `*-cloud` suffixes).
//! IDs align with published tags on https://ollama.com/search?c=cloud (verified March 2026).

pub struct ChatCloudModelEntry {
    pub id: &'static str,
    pub label: &'static str,
}

/// Popular / capable cloud options first (speed, reasoning, coding), then breadth.
pub const CHAT_CLOUD_MODEL_CHOICES: &[ChatCloudModelEntry] = &[
    ChatCloudModelEntry {
        id: "gemini-3-flash-preview:cloud",
        label: "Gemini 3 Flash",
    },
    ChatCloudModelEntry {
        id: "glm-5:cloud",
        label: "GLM 5",
    },
    ChatCloudModelEntry {
        id: "qwen3.5:397b-cloud",
        label: "Qwen 3.5 397B",
    },
    ChatCloudModelEntry {
        id: "qwen3.5:cloud",
        label: "Qwen 3.5 Cloud",
    },
    ChatCloudModelEntry {
        id: "deepseek-v3.2:cloud",
        label: "DeepSeek V3.2",
    },
    ChatCloudModelEntry {
        id: "minimax-m2.7:cloud",
        label: "MiniMax M2.7",
    },
    ChatCloudModelEntry {
        id: "kimi-k2.5:cloud",
        label: "Kimi K2.5",
    },
    ChatCloudModelEntry {
        id: "nemotron-3-super:cloud",
        label: "Nemotron 3 Super",
    },
    ChatCloudModelEntry {
        id: "qwen3-coder-next:cloud",
        label: "Qwen3 Coder Next",
    },
    ChatCloudModelEntry {
        id: "qwen3-next:80b-cloud",
        label: "Qwen3 Next 80B",
    },
    ChatCloudModelEntry {
        id: "devstral-2:123b-cloud",
        label: "Devstral 2 123B",
    },
];

pub fn is_chat_cloud_model_choice(id: &str) -> bool {
    CHAT_CLOUD_MODEL_CHOICES.iter().any(|entry| entry.id == id)
}

pub fn default_chat_cloud_model_id(configured_server_model: &str) -> String {
    if is_chat_cloud_model_choice(configured_server_model) {
        configured_server_model.to_string()
    } else {
        CHAT_CLOUD_MODEL_CHOICES[0].id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_cloud_tagged() {
        use crate::services::http::is_cloud_model;
        for entry in CHAT_CLOUD_MODEL_CHOICES {
            assert!(
                is_cloud_model(entry.id),
                "expected cloud tag, got {}",
                entry.id
            );
        }
    }

    #[test]
    fn default_prefers_configured_when_allowed() {
        assert_eq!(default_chat_cloud_model_id("glm-5:cloud"), "glm-5:cloud");
        assert_eq!(
            default_chat_cloud_model_id("qwen3:8b"),
            CHAT_CLOUD_MODEL_CHOICES[0].id
        );
    }
}
