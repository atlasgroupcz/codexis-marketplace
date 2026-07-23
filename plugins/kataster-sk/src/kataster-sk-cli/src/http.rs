use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use cdx_common::http::{backoff, base_url_from_env, format_status, join_url, retryable_status};

const DEFAULT_ZBGIS_BASE_URL: &str = "https://zbgis.skgeodesy.sk/mapka/api/search/kataster";
const DEFAULT_ESKN_BASE_URL: &str =
    "https://kataster.skgeodesy.sk/eskn/rest/services/VRM";
// The cadastre WAF rejects non-browser user agents, so impersonate one.
const USER_AGENT_VALUE: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";
const MAX_ATTEMPTS: usize = 3;

pub struct KatasterClient {
    client: Client,
    zbgis_base_url: String,
    eskn_base_url: String,
}

impl KatasterClient {
    pub fn new() -> Result<Self> {
        let zbgis_base_url = base_url_from_env("ZBGIS_BASE_URL", DEFAULT_ZBGIS_BASE_URL);
        let eskn_base_url = base_url_from_env("ESKN_BASE_URL", DEFAULT_ESKN_BASE_URL);
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
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
            let resp = match self
                .client
                .get(url)
                .query(query)
                .header(ACCEPT, "application/json")
                .send()
            {
                Ok(resp) => resp,
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(error.into());
                }
            };
            let status = resp.status();
            let final_url = resp.url().to_string();
            let body = resp.text().unwrap_or_default();
            if !status.is_success() {
                if retryable_status(status) && attempt + 1 < MAX_ATTEMPTS {
                    sleep(backoff(attempt));
                    continue;
                }
                return Err(anyhow!(
                    "HTTP {status} on GET {final_url}\n{body}",
                    status = format_status(status),
                    body = body.trim()
                ));
            }
            // A WAF block arrives as HTTP 200 + HTML - never retried.
            if body.trim_start().starts_with('<') {
                return Err(anyhow!(
                    "GET {final_url} returned HTML instead of JSON - the cadastre WAF blocked the request or the service is unavailable"
                ));
            }
            return Ok(body);
        }
        Err(anyhow!("GET {url} failed after {MAX_ATTEMPTS} attempts"))
    }
}
