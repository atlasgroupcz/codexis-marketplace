use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;

const DEFAULT_BASE: &str = "https://eregpublicsecure.ksrzis.cz/Registr/RZPRO";
const USER_AGENT_VALUE: &str = concat!("rzpro-cli/", env!("CARGO_PKG_VERSION"));

pub fn base_url() -> String {
    std::env::var("RZPRO_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_BASE.to_string())
        .trim_end_matches('/')
        .to_string()
}

pub struct RzproClient {
    client: Client,
    base: String,
}

impl RzproClient {
    pub fn new() -> Result<Self> {
        // The portal gates access behind a redirect that sets a session cookie;
        // a cookie store lets the client satisfy it on the automatic re-request.
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .cookie_store(true)
            .timeout(Duration::from_secs(180))
            .build()?;
        Ok(Self {
            client,
            base: base_url(),
        })
    }

    /// The OpenData landing page (HTML). Carries "Datum aktualizace DD.MM.YYYY".
    pub fn opendata_page(&self) -> Result<String> {
        let url = format!("{}/OpenData", self.base);
        let resp = self.client.get(&url).send()?;
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow!("HTTP {status} on GET {url}"))
        }
    }

    /// Download one dataset as raw ZIP bytes (contains a single CSV).
    pub fn dataset_zip(&self, typ: &str) -> Result<Vec<u8>> {
        let url = format!("{}/OpenData/GetFile?typSouboru={typ}", self.base);
        let resp = self.client.get(&url).send()?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} on GET {url}"));
        }
        Ok(resp.bytes()?.to_vec())
    }
}
