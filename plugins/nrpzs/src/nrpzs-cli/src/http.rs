use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::Value;
use ureq::Agent;

const DEFAULT_BASE_URL: &str = "https://nrpzs-api.uzis.cz/api/v1";
const USER_AGENT_VALUE: &str = concat!("nrpzs-cli/", env!("CARGO_PKG_VERSION"));
const PAGE: usize = 100;

pub struct NrpzsClient {
    agent: Agent,
    base_url: String,
}

impl NrpzsClient {
    pub fn new() -> Result<Self> {
        let base_url =
            std::env::var("NRPZS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        // ureq keeps a connection pool on the Agent -> requests to the same host
        // (e.g. the fetch_all pagination loop) reuse the TCP+TLS connection.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Ok(Self { agent, base_url })
    }

    pub fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = cdx_common::http::resolve_url(&self.base_url, path);
        let mut req = self
            .agent
            .get(&url)
            .set("Accept", "application/json")
            .set("User-Agent", USER_AGENT_VALUE);
        for &(k, v) in params {
            req = req.query(k, v);
        }
        match req.call() {
            Ok(resp) => Ok(resp.into_string()?),
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(anyhow!("HTTP {code} on GET {url}\n{}", body.trim()))
            }
            Err(e) => Err(anyhow!("GET {url} failed: {e}")),
        }
    }

    pub fn get_raw_allow_empty(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = cdx_common::http::resolve_url(&self.base_url, path);
        let mut req = self
            .agent
            .get(&url)
            .set("Accept", "application/json")
            .set("User-Agent", USER_AGENT_VALUE);
        for &(k, v) in params {
            req = req.query(k, v);
        }
        match req.call() {
            Ok(resp) => Ok(resp.into_string()?),
            Err(ureq::Error::Status(404, _)) => Ok("[]".to_string()),
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(anyhow!("HTTP {code} on GET {url}\n{}", body.trim()))
            }
            Err(e) => Err(anyhow!("GET {url} failed: {e}")),
        }
    }

    pub fn get_array(&self, path: &str, params: &[(&str, &str)]) -> Result<Vec<Value>> {
        let body = self.get_raw(path, params)?;
        let v: Value =
            serde_json::from_str(&body).map_err(|e| anyhow!("response is not JSON: {e}"))?;
        match v {
            Value::Array(a) => Ok(a),
            other => Err(anyhow!("expected a JSON array, got: {other}")),
        }
    }

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
}

