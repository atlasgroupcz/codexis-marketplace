use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// Publication date from the OpenData page, normalised to YYYY-MM-DD.
    pub datum: String,
    pub downloaded_at: u64,
    /// Row count per dataset (excluding header).
    pub counts: BTreeMap<String, usize>,
}

impl Manifest {
    /// ISO dates compare correctly as plain strings (lexicographic).
    pub fn is_older_than(&self, remote_datum: &str) -> bool {
        self.datum.as_str() < remote_datum
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::manifest_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn load() -> Option<Manifest> {
    let data = std::fs::read_to_string(paths::manifest_path()).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(datum: &str) -> Manifest {
        Manifest {
            datum: datum.into(),
            downloaded_at: 0,
            counts: BTreeMap::new(),
        }
    }

    #[test]
    fn staleness_by_date() {
        assert!(m("2026-06-23").is_older_than("2026-06-24"));
        assert!(!m("2026-06-24").is_older_than("2026-06-24"));
        assert!(!m("2026-06-24").is_older_than("2026-06-23"));
    }

    #[test]
    fn roundtrip_camelcase() {
        let json = serde_json::to_string(&m("2026-06-24")).unwrap();
        assert!(json.contains("\"downloadedAt\""));
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.datum, "2026-06-24");
    }
}
