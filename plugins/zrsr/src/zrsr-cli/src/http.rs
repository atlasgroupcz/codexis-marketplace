use std::time::Duration;

use anyhow::{Error, Result};
use cdx_common::http_client::call_with_retry;
use ureq::Agent;

const DEFAULT_BASE_URL: &str = "https://api.statistics.sk/rpo/v1";
const USER_AGENT_VALUE: &str = concat!("zrsr-cli/", env!("CARGO_PKG_VERSION"));

pub struct ZrsrClient {
    agent: Agent,
    base_url: String,
}

impl ZrsrClient {
    pub fn new() -> Result<Self> {
        let base_url = cdx_common::http::base_url_from_env("ZRSR_BASE_URL", DEFAULT_BASE_URL);
        let agent = cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(10),
            Duration::from_secs(30),
        );
        Ok(Self { agent, base_url })
    }

    pub fn get(&self, path: &str) -> Result<String> {
        self.get_with_query(path, &[])
    }

    pub fn get_with_query(&self, path: &str, query: &[(&str, String)]) -> Result<String> {
        let url = self.url_for(path);
        call_with_retry(|| {
            self.agent
                .get(&url)
                .header("Accept", "application/json")
                .query_pairs(query.iter().map(|(k, v)| (*k, v.as_str())))
                .call()
        })
        .into_text("GET", &url)
        .map_err(Error::msg)
    }

    fn url_for(&self, path: &str) -> String {
        cdx_common::http::resolve_url(&self.base_url, path)
    }
}
