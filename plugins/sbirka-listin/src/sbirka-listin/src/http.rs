//! Transport for the or.justice.cz register (the legacy portal).
//!
//! The new portal (verejnerejstriky.msp.gov.cz) has a clean JSON API but its
//! Azure WAF blocks datacenter egress IPs, so it is unreachable from the AWS
//! daemon (verified 2026-07-20: 403 to curl and reqwest alike from the daemon,
//! 200 from a normal IP). or.justice.cz runs on plain Tomcat with no such WAF
//! and answers from the daemon - so this is the reachable source. It is also the
//! official one; the new portal states it runs in trial operation.
//!
//! This layer is pure transport: fetch HTML or bytes. Turning the HTML into data
//! lives in `parse`, and the fetch-then-parse orchestration in `api`.

use std::io::Read;
use std::time::Duration;

use anyhow::{Error, Result};
use cdx_common::http_client::{call_with_retry, Response};
use ureq::Agent;

const DEFAULT_BASE_URL: &str = "https://or.justice.cz";
const BASE_URL_ENV: &str = "CODEXIS_PLUGIN_SBIRKA_LISTIN_API_URL";
const USER_AGENT_VALUE: &str = concat!("sbirka-listin-cli/", env!("CARGO_PKG_VERSION"));
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// A fetched document: its bytes and the Content-Type the server labelled them
/// with. The type is only knowable from the response, so the caller picks the
/// file extension from here, not from the listing.
pub struct Document {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

pub struct PortalClient {
    agent: Agent,
    base_url: String,
}

impl PortalClient {
    pub fn new() -> Result<Self> {
        let base_url = cdx_common::http::base_url_from_env(BASE_URL_ENV, DEFAULT_BASE_URL);
        let agent = cdx_common::http_client::agent(USER_AGENT_VALUE, CONNECT_TIMEOUT, READ_TIMEOUT);
        Ok(Self { agent, base_url })
    }

    /// GET a page and return its HTML. Retries on 429/5xx; a 404 is an answer,
    /// not something to retry.
    pub fn get_html(&self, path: &str) -> Result<String> {
        let mut resp = self.fetch(&cdx_common::http::resolve_url(&self.base_url, path))?;
        let mut buf = Vec::new();
        resp.body_mut().as_reader().read_to_end(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    /// GET a document's bytes plus its Content-Type. Retries on 429/5xx.
    pub fn get_bytes(&self, path: &str) -> Result<Document> {
        let mut resp = self.fetch(&cdx_common::http::resolve_url(&self.base_url, path))?;
        let content_type = resp
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or(v).trim().to_string());
        let mut bytes = Vec::new();
        resp.body_mut().as_reader().read_to_end(&mut bytes)?;
        Ok(Document {
            bytes,
            content_type,
        })
    }

    fn fetch(&self, url: &str) -> Result<Response> {
        call_with_retry(|| self.agent.get(url).call())
            .into_response("GET", url)
            .map_err(Error::msg)
    }
}
