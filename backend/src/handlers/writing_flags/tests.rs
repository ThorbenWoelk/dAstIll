use axum::http::StatusCode;

use super::{MAX_FLAG_COMMENT_CHARS, MAX_FLAG_TEXT_CHARS, validate_writing_flag};
use crate::models::CreateWritingFlagRequest;

fn request(text: &str, comment: &str) -> CreateWritingFlagRequest {
    CreateWritingFlagRequest {
        text: text.to_string(),
        comment: comment.to_string(),
        prefix_context: String::new(),
        suffix_context: String::new(),
    }
}

#[test]
fn a_flag_keeps_its_trimmed_text_and_note() {
    let payload = request("  the half worth fearing more ", " invented ranking ");
    assert_eq!(
        validate_writing_flag(&payload).unwrap(),
        ("the half worth fearing more", "invented ranking")
    );
}

#[test]
fn a_flag_needs_text_and_a_note() {
    for payload in [request(" ", "note"), request("text", "  ")] {
        let (status, _) = validate_writing_flag(&payload).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[test]
fn a_flag_rejects_overlong_text_or_notes() {
    let long_text = request(&"a".repeat(MAX_FLAG_TEXT_CHARS + 1), "note");
    let long_note = request("text", &"a".repeat(MAX_FLAG_COMMENT_CHARS + 1));
    for payload in [long_text, long_note] {
        let (status, _) = validate_writing_flag(&payload).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
