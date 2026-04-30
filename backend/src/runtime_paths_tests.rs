use super::local_libsql_dir_for_scope;
use std::path::Path;

#[test]
fn local_libsql_dir_is_stable_for_same_scope() {
    let base = Path::new("/tmp");
    let cwd = Path::new("/repo/worktree-a");

    let left = local_libsql_dir_for_scope(base, cwd, 3001);
    let right = local_libsql_dir_for_scope(base, cwd, 3001);

    assert_eq!(left, right);
}

#[test]
fn local_libsql_dir_changes_with_worktree() {
    let base = Path::new("/tmp");

    let left = local_libsql_dir_for_scope(base, Path::new("/repo/worktree-a"), 3001);
    let right = local_libsql_dir_for_scope(base, Path::new("/repo/worktree-b"), 3001);

    assert_ne!(left, right);
}

#[test]
fn local_libsql_dir_changes_with_port() {
    let base = Path::new("/tmp");
    let cwd = Path::new("/repo/worktree-a");

    let left = local_libsql_dir_for_scope(base, cwd, 3001);
    let right = local_libsql_dir_for_scope(base, cwd, 3544);

    assert_ne!(left, right);
}
