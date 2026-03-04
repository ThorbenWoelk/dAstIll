use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub fn build_http_client() -> Client {
    ClientBuilder::new()
        .user_agent("dastill/0.1")
        .timeout(Duration::from_secs(20))
        .build()
        .expect("http client build")
}

/// Detect rate-limit (HTTP 429) errors from the rig completion error chain.
pub fn is_rate_limited(err: &rig::completion::PromptError) -> bool {
    let msg = err.to_string();
    msg.contains("429") && msg.contains("Too Many Requests")
}
