use libsql::Connection;

pub async fn initialize_turso_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS videos (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            title TEXT NOT NULL,
            thumbnail_url TEXT,
            published_at TEXT NOT NULL,
            is_short INTEGER NOT NULL DEFAULT 0,
            transcript_status TEXT NOT NULL DEFAULT 'pending',
            summary_status TEXT NOT NULL DEFAULT 'pending',
            acknowledged INTEGER NOT NULL DEFAULT 0,
            retry_count INTEGER NOT NULL DEFAULT 0,
            quality_score INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_videos_channel_published
            ON videos(channel_id, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_videos_transcript_status
            ON videos(transcript_status);
        CREATE INDEX IF NOT EXISTS idx_videos_summary_status
            ON videos(summary_status);

        CREATE TABLE IF NOT EXISTS preferences (
            user_id TEXT PRIMARY KEY,
            data TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tts_stats (
            id TEXT PRIMARY KEY DEFAULT 'global',
            sample_count INTEGER NOT NULL DEFAULT 0,
            total_words INTEGER NOT NULL DEFAULT 0,
            total_duration_secs REAL NOT NULL DEFAULT 0.0
        );
        "#,
    )
    .await
    .map_err(|err| format!("failed to initialize Turso schema: {err}"))?;
    Ok(())
}
