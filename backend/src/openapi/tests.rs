use super::document;

#[test]
fn openapi_document_includes_live_debug_paths() {
    let document = document();
    assert!(document.paths.paths.contains_key("/api/openapi.json"));
    assert!(
        document
            .paths
            .paths
            .contains_key("/api/workspace/bootstrap")
    );
    assert!(document.paths.paths.contains_key("/api/chat/conversations"));
}
