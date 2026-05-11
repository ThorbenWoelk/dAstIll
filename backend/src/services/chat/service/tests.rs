use super::*;

#[test]
fn assistant_messages_strip_emojis() {
    let service = ChatService::new(OllamaCore::new("http://localhost:11434", "test-model"));

    let message = service.build_assistant_message(
        "Done ✅ with evidence.[1]".to_string(),
        Vec::new(),
        ChatMessageStatus::Completed,
        None,
    );

    assert_eq!(message.content, "Done  with evidence.[1]");
}
