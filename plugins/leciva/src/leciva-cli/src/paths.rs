use std::path::PathBuf;

fn home_dir() -> PathBuf {
    if let Ok(h) = std::env::var("CODEXIS_PUBLIC_USER_HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

pub fn app_dir() -> PathBuf {
    home_dir().join(".cdx").join("apps").join("leciva")
}

pub fn index_path() -> PathBuf {
    app_dir().join("index.tsv")
}

pub fn manifest_path() -> PathBuf {
    app_dir().join("manifest.json")
}

#[allow(dead_code)]
pub fn last_check_path() -> PathBuf {
    app_dir().join("last_check")
}

pub fn cis_dir() -> PathBuf {
    app_dir().join("cis")
}

pub fn cis_path(name: &str) -> PathBuf {
    cis_dir().join(name)
}

pub fn latka_drugs_path() -> PathBuf {
    app_dir().join("latka-drugs.tsv")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Both tests mutate the same process-global env var; serialize them so they
    // don't race under the parallel test runner (cargo test is multi-threaded).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn app_dir_honors_codexis_home() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", "/tmp/fakehome");
        assert_eq!(app_dir(), PathBuf::from("/tmp/fakehome/.cdx/apps/leciva"));
        assert_eq!(index_path(), PathBuf::from("/tmp/fakehome/.cdx/apps/leciva/index.tsv"));
        std::env::remove_var("CODEXIS_PUBLIC_USER_HOME");
    }

    #[test]
    fn cis_paths_under_app_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", "/tmp/fh2");
        assert_eq!(cis_dir(), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/cis"));
        assert_eq!(cis_path("latky.tsv"), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/cis/latky.tsv"));
        assert_eq!(latka_drugs_path(), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/latka-drugs.tsv"));
        std::env::remove_var("CODEXIS_PUBLIC_USER_HOME");
    }
}
