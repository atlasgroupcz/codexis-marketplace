use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://nrpzs-api.uzis.cz/api/v1";
const USER_AGENT_VALUE: &str = concat!("nrpzs-cli/", env!("CARGO_PKG_VERSION"));
const PAGE: usize = 100;

pub struct NrpzsClient {
    client: Client,
    base_url: String,
}

impl NrpzsClient {
    pub fn new() -> Result<Self> {
        let base_url =
            std::env::var("NRPZS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client, base_url })
    }

    /// Raw GET, body returned verbatim. `params` are URL query pairs.
    pub fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = self.url_for(path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .header(ACCEPT, "application/json")
            .send()?;
        read_body(resp, &url)
    }

    /// Like get_raw, but maps HTTP 404 (NRPZS "Empty result") to an empty JSON array.
    pub fn get_raw_allow_empty(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = self.url_for(path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .header(ACCEPT, "application/json")
            .send()?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok("[]".to_string());
        }
        read_body(resp, &url)
    }

    /// GET parsed as a JSON array.
    pub fn get_array(&self, path: &str, params: &[(&str, &str)]) -> Result<Vec<Value>> {
        let body = self.get_raw(path, params)?;
        let v: Value =
            serde_json::from_str(&body).map_err(|e| anyhow!("response is not JSON: {e}"))?;
        match v {
            Value::Array(a) => Ok(a),
            other => Err(anyhow!("expected a JSON array, got: {other}")),
        }
    }

    /// Page through all results for `params` (adds limit/offset), max 100/page.
    pub fn fetch_all(&self, path: &str, params: &[(&str, &str)]) -> Result<Vec<Value>> {
        let mut all = Vec::new();
        let mut offset = 0usize;
        loop {
            let off = offset.to_string();
            let lim = PAGE.to_string();
            let mut p: Vec<(&str, &str)> = params.to_vec();
            p.push(("limit", &lim));
            p.push(("offset", &off));
            let page = self.get_array(path, &p)?;
            let n = page.len();
            all.extend(page);
            if n < PAGE || offset > 100_000 {
                break;
            }
            offset += PAGE;
        }
        Ok(all)
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
}

fn read_body(resp: reqwest::blocking::Response, url: &str) -> Result<String> {
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if status.is_success() {
        Ok(body)
    } else {
        Err(anyhow!(
            "HTTP {status} on GET {url}\n{body}",
            status = format_status(status),
            url = url,
            body = body.trim()
        ))
    }
}

fn format_status(status: StatusCode) -> String {
    match status.canonical_reason() {
        Some(reason) => format!("{} {}", status.as_u16(), reason),
        None => status.as_u16().to_string(),
    }
}
