use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

const LOCAL_LIBSQL_DIR_PREFIX: &str = "dastill-search-index";

pub fn local_libsql_dir(base_temp_dir: &Path, port: u16) -> PathBuf {
    if let Some(explicit_dir) = std::env::var_os("DASTILL_LIBSQL_DIR") {
        return PathBuf::from(explicit_dir);
    }

    let cwd = std::env::current_dir()
        .ok()
        .and_then(|path| path.canonicalize().ok().or(Some(path)))
        .unwrap_or_else(|| PathBuf::from("."));

    local_libsql_dir_for_scope(base_temp_dir, &cwd, port)
}

fn local_libsql_dir_for_scope(base_temp_dir: &Path, cwd: &Path, port: u16) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    port.hash(&mut hasher);
    let scope_hash = hasher.finish();
    base_temp_dir.join(format!("{LOCAL_LIBSQL_DIR_PREFIX}-{scope_hash:016x}"))
}

#[cfg(test)]
#[path = "runtime_paths_tests.rs"]
mod runtime_paths_tests;
