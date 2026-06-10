use std::io::Read;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;

const DEFAULT_RPO_BASE_URL: &str = "https://api.statistics.sk/rpo/v1";
const DEFAULT_RUZ_BASE_URL: &str = "https://www.registeruz.sk/cruz-public/api";
const USER_AGENT_VALUE: &str = concat!("orsr-cli/", env!("CARGO_PKG_VERSION"));

pub struct RegistryClient {
    client: Client,
    rpo_base_url: String,
    ruz_base_url: String,
}

impl RegistryClient {
    pub fn new() -> Result<Self> {
        let rpo_base_url = base_url_from_env("RPO_BASE_URL", DEFAULT_RPO_BASE_URL);
        let ruz_base_url = base_url_from_env("RUZ_BASE_URL", DEFAULT_RUZ_BASE_URL);
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            rpo_base_url,
            ruz_base_url,
        })
    }

    pub fn rpo_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&join_url(&self.rpo_base_url, path), query)
    }

    pub fn ruz_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&join_url(&self.ruz_base_url, path), query)
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let mut resp = self.client.get(url).send()?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {} on GET {url}", format_status(status)));
        }
        let mut body = Vec::new();
        // TODO: drop once orsr.sk sends TLS close_notify — its legacy server
        // closes EOF-delimited responses abruptly, which strict rustls reads
        // report as an error even though the body already arrived.
        if let Err(e) = resp.read_to_end(&mut body) {
            if body.is_empty() {
                return Err(anyhow!("error reading GET {url}: {e}"));
            }
        }
        Ok(body)
    }

    fn get_json(&self, url: &str, query: &[(&str, &str)]) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .query(query)
            .header(ACCEPT, "application/json")
            .send()?;
        let status = resp.status();
        let final_url = resp.url().to_string();
        let body = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "HTTP {status} on GET {final_url}\n{body}",
                status = format_status(status),
                body = body.trim()
            ));
        }
        if body.trim_start().starts_with('<') {
            return Err(anyhow!(
                "GET {final_url} returned HTML instead of JSON — the service is unavailable or the request was blocked"
            ));
        }
        Ok(body)
    }
}

fn base_url_from_env(var: &str, default: &str) -> String {
    std::env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn join_url(base: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

fn format_status(status: StatusCode) -> String {
    match status.canonical_reason() {
        Some(reason) => format!("{} {}", status.as_u16(), reason),
        None => status.as_u16().to_string(),
    }
}
