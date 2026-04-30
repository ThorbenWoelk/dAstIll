use super::planner_access_constraints;
use crate::security::{AccessContext, AccessRole, AuthState};

#[test]
fn planner_constraints_allow_db_inspect_for_anonymous_scope() {
    let constraints = planner_access_constraints(&AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: vec!["channel-a".to_string()],
        allowed_other_video_ids: Vec::new(),
    });

    assert!(constraints.contains("Only use `db_inspect` for read-only library queries."));
    assert!(!constraints.contains("Do not use `db_inspect` unless the caller is signed in."));
    assert!(!constraints.contains("session role is `operator`"));
}
