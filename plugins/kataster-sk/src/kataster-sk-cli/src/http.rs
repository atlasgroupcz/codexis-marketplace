use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use ureq::Agent;

const DEFAULT_ZBGIS_BASE_URL: &str = "https://zbgis.skgeodesy.sk/mapka/api/search/kataster";
const DEFAULT_ESKN_BASE_URL: &str =
    "https://kataster.skgeodesy.sk/eskn/rest/services/VRM";
// The cadastre WAF rejects non-browser user agents, so impersonate one.
const USER_AGENT_VALUE: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";
const MAX_ATTEMPTS: usize = 3;

pub struct KatasterClient {
    agent: Agent,
    zbgis_base_url: String,
    eskn_base_url: String,
}

impl KatasterClient {
    pub fn new() -> Result<Self> {
        let zbgis_base_url = base_url_from_env("ZBGIS_BASE_URL", DEFAULT_ZBGIS_BASE_URL);
        let eskn_base_url = base_url_from_env("ESKN_BASE_URL", DEFAULT_ESKN_BASE_URL);
        // ureq keeps a connection pool on the Agent -> repeated requests to the
        // same cadastre host reuse the TCP+TLS connection.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Ok(Self {
            agent,
            zbgis_base_url,
            eskn_base_url,
        })
    }

    pub fn zbgis_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&join_url(&self.zbgis_base_url, path), query)
    }

    pub fn eskn_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&join_url(&self.eskn_base_url, path), query)
    }

    fn get_json(&self, url: &str, query: &[(&str, &str)]) -> Result<String> {
        for attempt in 0..MAX_ATTEMPTS {
            let mut req = self
                .agent
                .get(url)
                .set("Accept", "application/json")
                .set("User-Agent", USER_AGENT_VALUE);
            for &(k, v) in query {
                req = req.query(k, v);
            }
            match req.call() {
                Ok(resp) => {
                    let body = resp.into_string()?;
                    // A WAF block arrives as HTTP 200 + HTML - never retried.
                    if body.trim_start().starts_with('<') {
                        return Err(anyhow!(
                            "GET {url} returned HTML instead of JSON - the cadastre WAF blocked the request or the service is unavailable"
                        ));
                    }
                    return Ok(body);
                }
                Err(ureq::Error::Status(code, resp)) => {
                    if retryable_status(code) && attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    let reason = resp.status_text().to_string();
                    let body = resp.into_string().unwrap_or_default();
                    return Err(anyhow!(
                        "HTTP {status} on GET {url}\n{body}",
                        status = format_status(code, &reason),
                        url = url,
                        body = body.trim()
                    ));
                }
                Err(error) => {
                    // Transport / network error.
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(anyhow!("GET {url} failed: {error}"));
                }
            }
        }
        Err(anyhow!("GET {url} failed after {MAX_ATTEMPTS} attempts"))
    }
}

/// `"<code> <reason>"`, or just the code when there is no reason phrase.
/// (was cdx_common::http::format_status, adapted to ureq's numeric status)
fn format_status(code: u16, reason: &str) -> String {
    if reason.is_empty() {
        code.to_string()
    } else {
        format!("{code} {reason}")
    }
}

/// Reads an env var (trimmed of a trailing slash), falling back to `default`.
/// (was cdx_common::http::base_url_from_env)
fn base_url_from_env(var: &str, default: &str) -> String {
    std::env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Joins a base URL and a path; an empty path yields the base unchanged.
/// (was cdx_common::http::join_url)
fn join_url(base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

/// Whether a status is worth retrying (429 or any 5xx).
/// (was cdx_common::http::retryable_status)
fn retryable_status(code: u16) -> bool {
    code == 429 || (500..=599).contains(&code)
}

/// Exponential backoff: 250ms · 2^attempt.
/// (was cdx_common::http::backoff)
fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(250 * 2_u64.pow(attempt as u32))
}
