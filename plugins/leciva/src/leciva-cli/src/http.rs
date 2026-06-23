use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use serde::Deserialize;

const DEFAULT_DLP_BASE: &str = "https://prehledy.sukl.cz/dlp/v1";
const DEFAULT_OPENDATA_BASE: &str = "https://opendata.sukl.cz";
const USER_AGENT_VALUE: &str = concat!("leciva-cli/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Davka {
    pub typ: String,
    pub datum_vydani: String,
    pub platnost_od: String,
}

pub fn opendata_base() -> String {
    std::env::var("SUKL_OPENDATA_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_OPENDATA_BASE.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Vybere DLPO dávku ze seznamu /aktualni-davky.
pub fn pick_dlpo(davky: &[Davka]) -> Result<Davka> {
    davky
        .iter()
        .find(|d| d.typ.eq_ignore_ascii_case("DLPO"))
        .cloned()
        .ok_or_else(|| anyhow!("no DLPO davka in /aktualni-davky"))
}

pub struct SuklClient {
    client: Client,
    base_url: String,
}

impl SuklClient {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("SUKL_DLP_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_DLP_BASE.to_string())
            .trim_end_matches('/')
            .to_string();
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self { client, base_url })
    }

    fn url_for(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    pub fn get(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        let resp = self.client.get(&url).header(ACCEPT, "application/json").send()?;
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("HTTP {status} on GET {url}\n{}", body.trim()))
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Jako `get`, ale vrátí tělo i při ne-2xx. Některé endpointy (cau-scau,
    /// dokumenty-metadata) signalizují „bez záznamu" přes HTTP 404 + chybový JSON.
    pub fn get_lenient(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        let resp = self.client.get(&url).header(ACCEPT, "application/json").send()?;
        Ok(resp.text().unwrap_or_default())
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.client.get(url).send()?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} on GET {url}"));
        }
        Ok(resp.bytes()?.to_vec())
    }

    pub fn aktualni_dlpo(&self) -> Result<Davka> {
        let body = self.get("/aktualni-davky")?;
        let davky: Vec<Davka> =
            serde_json::from_str(&body).map_err(|e| anyhow!("cannot parse /aktualni-davky: {e}"))?;
        pick_dlpo(&davky)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_dlpo_over_scau() {
        let json = r#"[
            {"typ":"SCAU","datumVydani":"2026-05-27T00:00:00","platnostOd":"2026-06-01T00:00:00","verze":1,"zmeny":""},
            {"typ":"DLPO","datumVydani":"2026-05-27T00:00:00","platnostOd":"2026-06-01T00:00:00","verze":1,"zmeny":""}
        ]"#;
        let davky: Vec<Davka> = serde_json::from_str(json).unwrap();
        let dlpo = pick_dlpo(&davky).unwrap();
        assert_eq!(dlpo.typ, "DLPO");
        assert_eq!(dlpo.datum_vydani, "2026-05-27T00:00:00");
    }

    #[test]
    fn pick_dlpo_errors_when_absent() {
        let davky: Vec<Davka> = vec![];
        assert!(pick_dlpo(&davky).is_err());
    }
}
