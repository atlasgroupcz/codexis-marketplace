use std::time::Duration;

use anyhow::{anyhow, Error, Result};
use cdx_common::http_client::{call_with_retry, Outcome};
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
        let base_url = cdx_common::http::base_url_from_env("NRPZS_BASE_URL", DEFAULT_BASE_URL);
        let agent = cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(10),
            Duration::from_secs(30),
        );
        Ok(Self { agent, base_url })
    }

    /// Issues the GET. The status is left unjudged: `get_raw_allow_empty` reads
    /// a 404 as an empty result, `get_raw` reads it as an error.
    fn send(&self, url: &str, params: &[(&str, &str)]) -> Outcome {
        call_with_retry(|| {
            self.agent
                .get(url)
                .header("Accept", "application/json")
                .query_pairs(params.iter().copied())
                .call()
        })
    }

    pub fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = cdx_common::http::resolve_url(&self.base_url, path);
        self.send(&url, params)
            .into_text("GET", &url)
            .map_err(Error::msg)
    }

    /// This endpoint answers "no such register entry" with a 404, which is an
    /// answer rather than a failure - every other status is judged as usual.
    pub fn get_raw_allow_empty(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let url = cdx_common::http::resolve_url(&self.base_url, path);
        match self.send(&url, params) {
            Outcome::Status(404, _) => Ok("[]".to_string()),
            other => other.into_text("GET", &url).map_err(Error::msg),
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
