use super::{extract_readable_text, extract_title, website_url_identity};
use scraper::Html;

#[test]
fn extraction_prefers_article_text() {
    let document = Html::parse_document(
        "<html><head><title>Example</title></head><body><article><p>Hello</p><p>world</p></article></body></html>",
    );
    assert_eq!(extract_title(&document).as_deref(), Some("Example"));
    assert_eq!(extract_readable_text(&document), "Hello world");
}

#[test]
fn website_url_identity_distinguishes_slug_collisions() {
    let underscored = website_url_identity("https://example.com/foo_bar");
    let hyphenated = website_url_identity("https://example.com/foo-bar");
    let trailing_slash = website_url_identity("https://example.com/path/");
    let no_slash = website_url_identity("https://example.com/path");

    assert_ne!(underscored, hyphenated);
    assert_ne!(trailing_slash, no_slash);
    assert!(underscored.starts_with("https-example-com-foo-bar:"));
    assert!(hyphenated.starts_with("https-example-com-foo-bar:"));
}

#[test]
fn website_url_identity_is_stable_for_exact_url() {
    let first = website_url_identity("https://example.com/docs/guide");
    let second = website_url_identity("https://example.com/docs/guide");
    assert_eq!(first, second);
}
