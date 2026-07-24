use std::time::Duration;

use anyhow::{anyhow, Result};
use ureq::Agent;

const DEFAULT_BASE_URL: &str = "https://api.statistics.sk/rpo/v1";
const USER_AGENT_VALUE: &str = concat!("zrsr-cli/", env!("CARGO_PKG_VERSION"));

pub struct ZrsrClient {
    agent: Agent,
    base_url: String,
}

impl ZrsrClient {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("ZRSR_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        // ureq keeps a connection pool on the Agent -> follow-up requests to the
        // same host reuse the TCP+TLS connection.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Ok(Self { agent, base_url })
    }

    pub fn get(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        let req = self
            .agent
            .get(&url)
            .set("Accept", "application/json")
            .set("User-Agent", USER_AGENT_VALUE);
        send(req, &url)
    }

    pub fn get_with_query(&self, path: &str, query: &[(&str, String)]) -> Result<String> {
        let url = self.url_for(path);
        let mut req = self
            .agent
            .get(&url)
            .set("Accept", "application/json")
            .set("User-Agent", USER_AGENT_VALUE);
        for (k, v) in query {
            req = req.query(k, v);
        }
        send(req, &url)
    }

    fn url_for(&self, path: &str) -> String {
        resolve_url(&self.base_url, path)
    }
}

/// Sends a prepared GET request and reads its body, mapping a non-2xx status or
/// a transport error into an `anyhow` error carrying the status/URL/body.
fn send(req: ureq::Request, url: &str) -> Result<String> {
    match req.call() {
        Ok(resp) => Ok(resp.into_string()?),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(anyhow!("HTTP {code} on GET {url}\n{}", body.trim()))
        }
        Err(e) => Err(anyhow!("GET {url} failed: {e}")),
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
