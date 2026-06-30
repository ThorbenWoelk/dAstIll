use std::sync::Arc;

use super::{ReadCache, Store, sql_schema};
use crate::object_store::memory::MemoryObjectStore;

impl Store {
    pub async fn for_test() -> Store {
        let sql_db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create in-memory libSQL database for tests");
        let sql_conn = sql_db
            .connect()
            .expect("failed to connect to test libSQL database");
        sql_schema::initialize_sql_schema(&sql_conn)
            .await
            .expect("failed to initialize libSQL schema for tests");
        Store {
            objects: Arc::new(MemoryObjectStore::new()),
            sql: sql_conn,
            read_cache: ReadCache::default(),
            source_generation_tracker: None,
            snapshot_publisher: None,
        }
    }
}
