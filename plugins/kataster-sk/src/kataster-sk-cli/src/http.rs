use std::time::Duration;

use anyhow::{anyhow, Error, Result};
use cdx_common::http_client::call_with_retry;
use ureq::Agent;

const DEFAULT_ZBGIS_BASE_URL: &str = "https://zbgis.skgeodesy.sk/mapka/api/search/kataster";
const DEFAULT_ESKN_BASE_URL: &str = "https://kataster.skgeodesy.sk/eskn/rest/services/VRM";
// The cadastre WAF rejects non-browser user agents, so impersonate one.
const USER_AGENT_VALUE: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

pub struct KatasterClient {
    agent: Agent,
    zbgis_base_url: String,
    eskn_base_url: String,
}

impl KatasterClient {
    pub fn new() -> Result<Self> {
        let zbgis_base_url =
            cdx_common::http::base_url_from_env("ZBGIS_BASE_URL", DEFAULT_ZBGIS_BASE_URL);
        let eskn_base_url =
            cdx_common::http::base_url_from_env("ESKN_BASE_URL", DEFAULT_ESKN_BASE_URL);
        let agent = cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(10),
            Duration::from_secs(30),
        );
        Ok(Self {
            agent,
            zbgis_base_url,
            eskn_base_url,
        })
    }

    pub fn zbgis_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(
            &cdx_common::http::join_url(&self.zbgis_base_url, path),
            query,
        )
    }

    pub fn eskn_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(
            &cdx_common::http::join_url(&self.eskn_base_url, path),
            query,
        )
    }

    fn get_json(&self, url: &str, query: &[(&str, &str)]) -> Result<String> {
        let body = call_with_retry(|| {
            self.agent
                .get(url)
                .header("Accept", "application/json")
                .query_pairs(query.iter().copied())
                .call()
        })
        .into_text("GET", url)
        .map_err(Error::msg)?;

        // A WAF block arrives as HTTP 200 + HTML - never retried, and it is the
        // one failure this service reports with a success status.
        if cdx_common::http::looks_like_html(&body) {
            return Err(anyhow!(
                "GET {url} returned HTML instead of JSON - the cadastre WAF blocked the request or the service is unavailable"
            ));
        }
        Ok(body)
    }
}
