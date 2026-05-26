use crate::error::CdxctlError;
use reqwest::blocking::multipart::Form;
use serde_json::{json, Value};

const DEFAULT_API_URL: &str = "http://localhost:8086/graphql";
const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";
const SECRET_FILE_RELATIVE: &str = ".cdx/env/secret";

pub struct GraphQLClient {
    url: String,
    auth: String,
    client: reqwest::blocking::Client,
}

impl GraphQLClient {
    pub fn new() -> Self {
        let url = std::env::var("CODEXIS_PUBLIC_DAEMON_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let auth = load_api_jwt_auth();
        GraphQLClient {
            url,
            auth,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Base URL without the trailing `/graphql` so REST endpoints can be derived.
    pub fn rest_base_url(&self) -> String {
        self.url
            .trim_end_matches('/')
            .trim_end_matches("/graphql")
            .trim_end_matches('/')
            .to_string()
    }

    /// GET a REST endpoint and decode JSON.
    pub fn rest_get(&self, path: &str) -> Result<Value, CdxctlError> {
        let url = format!("{}{}", self.rest_base_url(), path);
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// POST a multipart body (used for /rest/v1/channels/email/send).
    pub fn rest_post_multipart(&self, path: &str, form: Form) -> Result<Value, CdxctlError> {
        let url = format!("{}{}", self.rest_base_url(), path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .multipart(form)
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// POST a JSON-less endpoint (no body), used for /test triggers.
    pub fn rest_post_empty(&self, path: &str) -> Result<Value, CdxctlError> {
        let url = format!("{}{}", self.rest_base_url(), path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// Execute a GraphQL operation and return the `data` field.
    pub fn execute(&self, query: &str, variables: Value) -> Result<Value, CdxctlError> {
        let body = json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Authorization", &self.auth)
            .json(&body)
            .send()?;

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }

        let parsed: Value = serde_json::from_str(&text)?;

        // Check for GraphQL errors
        if let Some(errors) = parsed.get("errors") {
            if let Some(arr) = errors.as_array() {
                if !arr.is_empty() {
                    let messages: Vec<String> = arr
                        .iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .map(String::from)
                        .collect();
                    return Err(CdxctlError::GraphQL(messages));
                }
            }
        }

        parsed
            .get("data")
            .cloned()
            .ok_or_else(|| CdxctlError::Parse("Response missing 'data' field".into()))
    }
}

fn load_api_jwt_auth() -> String {
    if let Some(secret) = read_secret_file() {
        return ensure_bearer_prefix(&secret);
    }
    if let Ok(val) = std::env::var(CODEXIS_USER_API_TOKEN_ENV) {
        if !val.is_empty() {
            return ensure_bearer_prefix(&val);
        }
    }
    eprintln!("error: no auth available (tried ~/{SECRET_FILE_RELATIVE} and ${CODEXIS_USER_API_TOKEN_ENV})");
    std::process::exit(2);
}

fn read_secret_file() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = std::path::Path::new(&home).join(SECRET_FILE_RELATIVE);
    let contents = std::fs::read_to_string(&path).ok()?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn ensure_bearer_prefix(value: &str) -> String {
    let mut s = value.trim();
    if let Some(stripped) = s.strip_prefix("Authorization:") {
        s = stripped.trim();
    }
    if s.len() >= 7 && s[..7].eq_ignore_ascii_case("Bearer ") {
        s.to_string()
    } else {
        format!("Bearer {s}")
    }
}
