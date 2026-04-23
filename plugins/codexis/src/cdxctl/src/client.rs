use crate::error::CdxctlError;
use serde_json::{json, Value};
use std::fs;

const DEFAULT_API_URL: &str = "http://localhost:8086/graphql";

pub struct GraphQLClient {
    url: String,
    auth: String,
    client: reqwest::blocking::Client,
}

impl GraphQLClient {
    pub fn new() -> Self {
        let url = std::env::var("CDXCTL_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let auth = load_daemon_auth();
        GraphQLClient {
            url,
            auth,
            client: reqwest::blocking::Client::new(),
        }
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

fn load_daemon_auth() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/codexis".to_string());
    let env_file = format!("{}/.cdx/.daemon.env", home);
    if let Ok(content) = fs::read_to_string(&env_file) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("CDX_DAEMON_AUTH=") {
                let val = val.trim();
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    if let Ok(val) = std::env::var("CDX_DAEMON_AUTH") {
        if !val.is_empty() {
            return val;
        }
    }
    eprintln!("error: CDX_DAEMON_AUTH not found in ~/.cdx/.daemon.env or environment");
    std::process::exit(2);
}
