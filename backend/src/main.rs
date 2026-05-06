use dastill::local_env::load_envs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dastill::startup::install_crypto_providers();
    load_envs();
    let _logfire_guard = dastill::logging::init_tracing()?;

    let (port, listener) = dastill::startup::bind_startup_listener().await?;
    let runtime = dastill::startup::build_runtime(port).await?;
    dastill::startup::spawn_runtime_workers(runtime.state.clone(), runtime.fts_dir).await;
    let app = dastill::routes::build_app(runtime.state, runtime.security.as_ref())?;

    tracing::info!(
        "initialization complete — serving on {}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}
