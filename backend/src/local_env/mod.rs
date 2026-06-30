use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn shared_local_dir() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(config_home).join("dastill"));
    }

    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".config").join("dastill"))
}

fn shared_local_env_path() -> Option<PathBuf> {
    shared_local_dir().map(|dir| dir.join("backend.env"))
}

fn load_dotenv_file(path: &Path, shell_env_keys: &HashSet<OsString>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            if key.is_empty()
                || key.starts_with('#')
                || shell_env_keys.contains(&OsString::from(key))
            {
                continue;
            }

            // Process-level env should win over file-based env so local shell overrides can
            // unblock startup without editing ignored files. Later files override earlier files.
            unsafe { env::set_var(key, value) };
        }
    }
}

pub fn load_envs() {
    let shell_env_keys: HashSet<OsString> = env::vars_os().map(|(key, _)| key).collect();

    if let Some(shared_path) = shared_local_env_path() {
        load_dotenv_file(&shared_path, &shell_env_keys);
    }

    load_dotenv_file(Path::new(".env"), &shell_env_keys);
}

pub fn clear_missing_google_application_credentials() -> bool {
    let Some(path) = env::var_os("GOOGLE_APPLICATION_CREDENTIALS") else {
        return false;
    };
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty() || path.exists() {
        return false;
    }

    unsafe { env::remove_var("GOOGLE_APPLICATION_CREDENTIALS") };
    true
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
