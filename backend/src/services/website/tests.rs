use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use super::{
    MAX_WEBSITE_HTML_BYTES, WebsiteService, extract_readable_text, extract_title,
    read_response_text_limited,
};
use scraper::Html;

#[test]
fn extraction_prefers_article_text() {
    let document = Html::parse_document(
        "<html><head><title>Example</title></head><body><article><p>Hello</p><p>world</p></article></body></html>",
    );
    assert_eq!(extract_title(&document).as_deref(), Some("Example"));
    assert_eq!(extract_readable_text(&document), "Hello world");
}

fn serve_once(status_line: &str, headers: &str, body: &[u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener should have addr");
    let status_line = status_line.to_string();
    let headers = headers.to_string();
    let body = body.to_vec();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let response = [
            status_line.as_bytes(),
            b"\r\n",
            headers.as_bytes(),
            b"\r\n\r\n",
            body.as_slice(),
        ]
        .concat();
        stream.write_all(&response).expect("response should write");
    });
    format!("http://{addr}/")
}

#[tokio::test]
async fn read_response_text_limited_rejects_content_length_over_cap() {
    let url = serve_once(
        "HTTP/1.1 200 OK",
        &format!("Content-Length: {}", MAX_WEBSITE_HTML_BYTES + 1),
        b"x",
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_text_limited(response, MAX_WEBSITE_HTML_BYTES)
        .await
        .expect_err("oversized content-length should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn read_response_text_limited_rejects_streamed_body_over_cap() {
    let oversized = vec![b'a'; (MAX_WEBSITE_HTML_BYTES as usize) + 8];
    let url = serve_once(
        "HTTP/1.1 200 OK",
        "Content-Type: text/html",
        &oversized,
    );
    let response = reqwest::get(&url).await.expect("request should send");
    let err = read_response_text_limited(response, MAX_WEBSITE_HTML_BYTES)
        .await
        .expect_err("oversized body should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn resolve_page_rejects_oversized_html_without_buffering_full_body() {
    let oversized = vec![b'a'; (MAX_WEBSITE_HTML_BYTES as usize) + 32];
    let url = serve_once(
        "HTTP/1.1 200 OK",
        "Content-Type: text/html",
        &oversized,
    );
    let service = WebsiteService::new();
    let err = service
        .resolve_page(&url)
        .await
        .expect_err("oversized website HTML should fail");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn resolve_page_accepts_small_html() {
    let body = b"<html><head><title>Ok</title></head><body><article><p>Hello site</p></article></body></html>";
    let url = serve_once(
        "HTTP/1.1 200 OK",
        &format!("Content-Type: text/html\r\nContent-Length: {}", body.len()),
        body,
    );
    let service = WebsiteService::new();
    let page = service
        .resolve_page(&url)
        .await
        .expect("small website HTML should resolve");
    assert_eq!(page.title, "Ok");
    assert!(page.text_content.contains("Hello site"));
}
