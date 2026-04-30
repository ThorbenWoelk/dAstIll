use super::{ReadCache, Store, turso_schema};

impl Store {
    pub async fn for_test() -> Store {
        let config = crate::aws_auth::load_aws_sdk_config("us-east-1".to_string())
            .await
            .expect("failed to build AWS SDK config for tests");
        let s3 = aws_sdk_s3::Client::new(&config);
        let s3v = aws_sdk_s3vectors::Client::new(&config);
        let turso_db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create in-memory libSQL database for tests");
        let turso_conn = turso_db
            .connect()
            .expect("failed to connect to test libSQL database");
        turso_schema::initialize_turso_schema(&turso_conn)
            .await
            .expect("failed to initialize libSQL schema for tests");
        Store {
            s3,
            s3v,
            turso: turso_conn,
            data_bucket: std::env::var("S3_DATA_BUCKET")
                .unwrap_or_else(|_| "dastill-test".to_string()),
            vector_bucket: std::env::var("S3_VECTOR_BUCKET")
                .unwrap_or_else(|_| "dastill-vectors-test".to_string()),
            vector_index: std::env::var("S3_VECTOR_INDEX")
                .unwrap_or_else(|_| "search-chunks".to_string()),
            read_cache: ReadCache::default(),
            snapshot_publisher: None,
        }
    }
}
