use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;

use super::{clear_missing_google_application_credentials, load_envs};

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn load_dotenv_preserves_existing_env_vars() {
    let _guard = TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let temp = tempdir().expect("tempdir");
    std::fs::write(
        temp.path().join(".env"),
        "KEEP_ME=from-dotenv\nSET_ME=from-dotenv\n",
    )
    .expect("write dotenv");
    let _reset = TestReset::capture(&["KEEP_ME", "SET_ME"], temp.path());
    unsafe { env::set_var("KEEP_ME", "from-shell") };
    unsafe { env::remove_var("SET_ME") };

    load_envs();

    assert_eq!(env::var("KEEP_ME").as_deref(), Ok("from-shell"));
    assert_eq!(env::var("SET_ME").as_deref(), Ok("from-dotenv"));
}

#[test]
fn load_dotenv_uses_shared_env_when_local_file_is_absent() {
    let _guard = TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let temp = tempdir().expect("tempdir");
    let config_home = temp.path().join("config-home");
    let shared_dir = config_home.join("dastill");
    std::fs::create_dir_all(&shared_dir).expect("create shared env dir");
    std::fs::write(shared_dir.join("backend.env"), "SET_ME=from-shared\n")
        .expect("write shared dotenv");
    let _reset = TestReset::capture_with_env(
        &["SET_ME", "XDG_CONFIG_HOME", "HOME"],
        temp.path(),
        &[("XDG_CONFIG_HOME", Some(config_home.as_os_str()))],
    );
    unsafe { env::remove_var("SET_ME") };

    load_envs();

    assert_eq!(env::var("SET_ME").as_deref(), Ok("from-shared"));
}

#[test]
fn load_dotenv_prefers_local_env_over_shared_file() {
    let _guard = TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let temp = tempdir().expect("tempdir");
    let config_home = temp.path().join("config-home");
    let shared_dir = config_home.join("dastill");
    std::fs::create_dir_all(&shared_dir).expect("create shared env dir");
    std::fs::write(shared_dir.join("backend.env"), "SET_ME=from-shared\n")
        .expect("write shared dotenv");
    std::fs::write(temp.path().join(".env"), "SET_ME=from-local\n").expect("write dotenv");
    let _reset = TestReset::capture_with_env(
        &["SET_ME", "XDG_CONFIG_HOME", "HOME"],
        temp.path(),
        &[("XDG_CONFIG_HOME", Some(config_home.as_os_str()))],
    );
    unsafe { env::remove_var("SET_ME") };

    load_envs();

    assert_eq!(env::var("SET_ME").as_deref(), Ok("from-local"));
}

#[test]
fn clear_missing_google_application_credentials_unsets_missing_path() {
    let _guard = TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let temp = tempdir().expect("tempdir");
    let missing_path = temp.path().join("missing-creds.json");
    let _reset = TestReset::capture_current_env(&["GOOGLE_APPLICATION_CREDENTIALS"]);
    unsafe { env::set_var("GOOGLE_APPLICATION_CREDENTIALS", &missing_path) };

    assert!(clear_missing_google_application_credentials());
    assert!(env::var_os("GOOGLE_APPLICATION_CREDENTIALS").is_none());
}

#[test]
fn clear_missing_google_application_credentials_keeps_existing_path() {
    let _guard = TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let temp = tempdir().expect("tempdir");
    let creds_path = temp.path().join("creds.json");
    std::fs::write(&creds_path, "{}").expect("write creds file");
    let _reset = TestReset::capture_current_env(&["GOOGLE_APPLICATION_CREDENTIALS"]);
    unsafe { env::set_var("GOOGLE_APPLICATION_CREDENTIALS", &creds_path) };

    assert!(!clear_missing_google_application_credentials());
    assert_eq!(
        env::var_os("GOOGLE_APPLICATION_CREDENTIALS"),
        Some(creds_path.into_os_string())
    );
}

struct TestReset {
    saved_dir: PathBuf,
    saved_env: Vec<(String, Option<std::ffi::OsString>)>,
}

impl TestReset {
    fn capture(keys: &[&str], dir: &std::path::Path) -> Self {
        let reset = Self::capture_current_env(keys);
        env::set_current_dir(dir).expect("set current dir");
        reset
    }

    fn capture_current_env(keys: &[&str]) -> Self {
        let saved_dir = env::current_dir().expect("current dir");
        let saved_env = keys
            .iter()
            .map(|key| ((*key).to_string(), env::var_os(key)))
            .collect();
        Self {
            saved_dir,
            saved_env,
        }
    }

    fn capture_with_env(
        keys: &[&str],
        dir: &std::path::Path,
        env_overrides: &[(&str, Option<&std::ffi::OsStr>)],
    ) -> Self {
        let reset = Self::capture_current_env(keys);
        env::set_current_dir(dir).expect("set current dir");
        for (key, value) in env_overrides {
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
        }
        reset
    }
}

impl Drop for TestReset {
    fn drop(&mut self) {
        env::set_current_dir(&self.saved_dir).expect("restore current dir");
        for (key, value) in &self.saved_env {
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
        }
    }
}
