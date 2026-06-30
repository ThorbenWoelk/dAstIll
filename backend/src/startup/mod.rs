use std::net::SocketAddr;

pub fn install_crypto_providers() {
    // Install crypto providers for all rustls versions in the dependency tree.
    // Dependency graph uses multiple rustls versions.
    // Installing both ensures TLS works across the entire tree.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls 0.23 crypto provider");
    // Note: rustls 0.22 (via libsql) uses a global default approach and will pick up
    // the ring crypto features via its dependency on rustls-webpki.
}

pub async fn bind_startup_listener() -> anyhow::Result<(u16, tokio::net::TcpListener)> {
    // Bind the port immediately so Cloud Run's TCP startup probe succeeds
    // before the rest of initialization runs. The OS queues incoming
    // connections in the backlog until axum::serve() processes them.
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("port {} bound - waiting for initialization", port);
    Ok((port, listener))
}
