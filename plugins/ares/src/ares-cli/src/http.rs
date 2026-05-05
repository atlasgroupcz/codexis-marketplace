use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::StatusCode;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://ares.gov.cz/ekonomicke-subjekty-v-be/rest";
const USER_AGENT_VALUE: &str = concat!("ares-cli/", env!("CARGO_PKG_VERSION"));

pub struct AresClient {
    client: Client,
    base_url: String,
}

impl AresClient {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("ARES_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let base_url = base_url.trim_end_matches('/').to_string();
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client, base_url })
    }

    pub fn get(&self, path: &str) -> Result<String> {
        let url = self.url_for(path);
        let resp = self
            .client
            .get(&url)
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, USER_AGENT_VALUE)
            .send()?;
        read_body(resp, &url, "GET")
    }

    pub fn post_json(&self, path: &str, body: &Value) -> Result<String> {
        let url = self.url_for(path);
        let resp = self
            .client
            .post(&url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, USER_AGENT_VALUE)
            .json(body)
            .send()?;
        read_body(resp, &url, "POST")
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

fn read_body(resp: reqwest::blocking::Response, url: &str, method: &str) -> Result<String> {
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if status.is_success() {
        Ok(body)
    } else {
        Err(anyhow!(
            "HTTP {status} on {method} {url}\n{body}",
            status = format_status(status),
            method = method,
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
