use super::{extract_readable_text, extract_title};
use scraper::Html;

#[test]
fn extraction_prefers_article_text() {
    let document = Html::parse_document(
        "<html><head><title>Example</title></head><body><article><p>Hello</p><p>world</p></article></body></html>",
    );
    assert_eq!(extract_title(&document).as_deref(), Some("Example"));
    assert_eq!(extract_readable_text(&document), "Hello world");
}
