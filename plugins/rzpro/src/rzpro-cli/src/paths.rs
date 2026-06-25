use std::path::PathBuf;

fn home_dir() -> PathBuf {
    if let Ok(h) = std::env::var("CODEXIS_PUBLIC_USER_HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn app_dir() -> PathBuf {
    home_dir().join(".cdx").join("apps").join("rzpro")
}

pub fn data_dir() -> PathBuf {
    app_dir().join("data")
}

/// Local path of an extracted dataset CSV (e.g. "RZPRO_ZP" -> .../data/RZPRO_ZP.csv).
pub fn csv_path(dataset: &str) -> PathBuf {
    data_dir().join(format!("{dataset}.csv"))
}

pub fn manifest_path() -> PathBuf {
    app_dir().join("manifest.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn app_dir_honors_codexis_home() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", "/tmp/fakehome");
        assert_eq!(app_dir(), PathBuf::from("/tmp/fakehome/.cdx/apps/rzpro"));
        assert_eq!(
            csv_path("RZPRO_ZP"),
            PathBuf::from("/tmp/fakehome/.cdx/apps/rzpro/data/RZPRO_ZP.csv")
        );
        std::env::remove_var("CODEXIS_PUBLIC_USER_HOME");
    }
}
