use axum::http::StatusCode;

use super::resolve_delete_highlight_result;

#[test]
fn delete_highlight_result_maps_missing_rows_to_not_found() {
    assert_eq!(
        resolve_delete_highlight_result(true).unwrap(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        resolve_delete_highlight_result(false).unwrap_err().0,
        StatusCode::NOT_FOUND
    );
}
