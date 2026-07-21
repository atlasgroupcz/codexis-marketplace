//! Transport for the or.justice.cz register (the legacy portal).
//!
//! The new portal (verejnerejstriky.msp.gov.cz) has a clean JSON API but its
//! Azure WAF blocks datacenter egress IPs, so it is unreachable from the AWS
//! daemon (verified 2026-07-20: 403 to curl and reqwest alike from the daemon,
//! 200 from a normal IP). or.justice.cz runs on plain Tomcat with no such WAF
//! and answers from the daemon — so this is the reachable source. It is also the
//! official one; the new portal states it runs in trial operation.
//!
//! This layer is pure transport: fetch HTML or bytes. Turning the HTML into data
//! lives in `parse`, and the fetch-then-parse orchestration in `api`.

use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_TYPE, USER_AGENT};

use cdx_common::http::{backoff, format_status, resolve_url, retryable_status};

const DEFAULT_BASE_URL: &str = "https://or.justice.cz";
const BASE_URL_ENV: &str = "CODEXIS_PLUGIN_SBIRKA_LISTIN_API_URL";
const USER_AGENT_VALUE: &str = concat!("sbirka-listin-cli/", env!("CARGO_PKG_VERSION"));
const TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ATTEMPTS: usize = 3;

/// A fetched document: its bytes and the Content-Type the server labelled them
/// with. The type is only knowable from the response, so the caller picks the
/// file extension from here, not from the listing.
pub struct Document {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

pub struct PortalClient {
    client: Client,
    base_url: String,
}

impl PortalClient {
    pub fn new() -> Result<Self> {
        let base_url = cdx_common::http::base_url_from_env(BASE_URL_ENV, DEFAULT_BASE_URL);
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(TIMEOUT)
            .build()?;
        Ok(Self { client, base_url })
    }

    /// GET a page and return its HTML. Retries on 429/5xx; a 404 is an answer,
    /// not something to retry.
    pub fn get_html(&self, path: &str) -> Result<String> {
        let mut resp = self.fetch(&resolve_url(&self.base_url, path))?;
        let mut buf = Vec::new();
        resp.read_to_end(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    /// GET a document's bytes plus its Content-Type. Retries on 429/5xx.
    pub fn get_bytes(&self, path: &str) -> Result<Document> {
        let mut resp = self.fetch(&resolve_url(&self.base_url, path))?;
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or(v).trim().to_string());
        let mut bytes = Vec::new();
        resp.read_to_end(&mut bytes)?;
        Ok(Document {
            bytes,
            content_type,
        })
    }

    fn fetch(&self, url: &str) -> Result<Response> {
        for attempt in 0..MAX_ATTEMPTS {
            match self
                .client
                .get(url)
                .header(USER_AGENT, USER_AGENT_VALUE)
                .send()
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return Ok(resp);
                    }
                    if retryable_status(status) && attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(anyhow!("HTTP {} on GET {url}", format_status(status)));
                }
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(error.into());
                }
            }
        }
        Err(anyhow!("GET {url} failed after {MAX_ATTEMPTS} attempts"))
    }
}
