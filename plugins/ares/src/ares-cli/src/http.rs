use std::time::Duration;

use anyhow::{Error, Result};
use cdx_common::http_client::call_with_retry;
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
        let base_url = cdx_common::http::base_url_from_env("ARES_BASE_URL", DEFAULT_BASE_URL);
        let agent = cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(10),
            Duration::from_secs(30),
        );
        Ok(Self { agent, base_url })
    }

    pub fn get(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        call_with_retry(|| {
            self.agent
                .get(&url)
                .header("Accept", "application/json")
                .call()
        })
        .into_text("GET", &url)
        .map_err(Error::msg)
    }

    /// The only POST ARES takes is `vyhledat`, a search. It changes nothing on
    /// the server, so it retries like the GETs do.
    pub fn post_json(&self, path: &str, body: &Value) -> Result<String> {
        let url = self.url_for(path);
        call_with_retry(|| {
            self.agent
                .post(&url)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .send_json(body)
        })
        .into_text("POST", &url)
        .map_err(Error::msg)
    }

    fn url_for(&self, path: &str) -> String {
        cdx_common::http::resolve_url(&self.base_url, path)
    }
}
