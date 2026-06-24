use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub batch: String,
    pub platnost_od: String,
    pub downloaded_at: u64,
    pub row_count: usize,
}

impl Manifest {
    /// ISO dates (YYYY-MM-DD…) compare correctly as plain strings (lexicographic).
    pub fn is_older_than(&self, remote_platnost_od: &str) -> bool {
        self.platnost_od.as_str() < remote_platnost_od
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::manifest_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

pub fn load() -> Option<Manifest> {
    let data = std::fs::read_to_string(paths::manifest_path()).ok()?;
    serde_json::from_str(&data).ok()
}

/// Stale = no local manifest, or the local one is older than the remote batch.
pub fn is_stale(remote_platnost_od: &str) -> bool {
    match load() {
        None => true,
        Some(m) => m.is_older_than(remote_platnost_od),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(platnost: &str) -> Manifest {
        Manifest {
            batch: "2026.06".into(),
            platnost_od: platnost.into(),
            downloaded_at: 0,
            row_count: 69147,
        }
    }

    #[test]
    fn older_when_remote_newer() {
        assert!(m("2026-06-01T00:00:00").is_older_than("2026-07-01T00:00:00"));
    }

    #[test]
    fn not_older_when_same() {
        assert!(!m("2026-06-01T00:00:00").is_older_than("2026-06-01T00:00:00"));
    }

    #[test]
    fn not_older_when_remote_older() {
        assert!(!m("2026-06-01T00:00:00").is_older_than("2026-05-01T00:00:00"));
    }

    #[test]
    fn roundtrip_json_camelcase() {
        let json = serde_json::to_string(&m("2026-06-01T00:00:00")).unwrap();
        assert!(json.contains("\"platnostOd\""));
        assert!(json.contains("\"rowCount\""));
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.batch, "2026.06");
    }
}
