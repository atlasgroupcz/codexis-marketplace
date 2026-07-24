use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::Value;
use ureq::Agent;

const DEFAULT_BASE_URL: &str = "https://ares.gov.cz/ekonomicke-subjekty-v-be/rest";
const USER_AGENT_VALUE: &str = concat!("ares-cli/", env!("CARGO_PKG_VERSION"));

pub struct AresClient {
    agent: Agent,
    base_url: String,
}

impl AresClient {
    pub fn new() -> Result<Self> {
        let base_url =
            std::env::var("ARES_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        // ureq keeps a connection pool on the Agent -> repeated lookups to the
        // same host reuse the TCP+TLS connection.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Ok(Self { agent, base_url })
    }

    pub fn get(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        let result = self
            .agent
            .get(&url)
            .set("Accept", "application/json")
            .set("User-Agent", USER_AGENT_VALUE)
            .call();
        read_response(result, &url, "GET")
    }

    pub fn post_json(&self, path: &str, body: &Value) -> Result<String> {
        let url = self.url_for(path);
        let result = self
            .agent
            .post(&url)
            .set("Accept", "application/json")
            .set("Content-Type", "application/json")
            .set("User-Agent", USER_AGENT_VALUE)
            .send_json(body);
        read_response(result, &url, "POST")
    }

    fn url_for(&self, path: &str) -> String {
        resolve_url(&self.base_url, path)
    }
}

/// Reads a ureq response, returning the body on success or an error carrying
/// the status, method, URL and body on failure (was cdx_common::http::read_body).
fn read_response(
    result: Result<ureq::Response, ureq::Error>,
    url: &str,
    method: &str,
) -> Result<String> {
    match result {
        Ok(resp) => Ok(resp.into_string()?),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(anyhow!("HTTP {code} on {method} {url}\n{}", body.trim()))
        }
        Err(ureq::Error::Transport(t)) => Err(anyhow!("{method} {url} failed: {t}")),
    }
}

/// Resolves a request path against a base URL (was cdx_common::http::resolve_url).
fn resolve_url(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}
