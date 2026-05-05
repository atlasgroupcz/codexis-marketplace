use crate::error::CdxctlError;
use serde_json::{json, Value};
use std::fs;

const DEFAULT_API_URL: &str = "http://localhost:8086/graphql";
const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";
const CDX_ENV_FILE_RELATIVE_PATH: &str = ".cdx/.env";

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
    if let Ok(val) = std::env::var(CODEXIS_USER_API_TOKEN_ENV) {
        if !val.is_empty() {
            return normalize_authorization_value(&val);
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/codexis".to_string());
    let env_file = format!("{}/{}", home, CDX_ENV_FILE_RELATIVE_PATH);
    if let Ok(content) = fs::read_to_string(&env_file) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix(&format!("{CODEXIS_USER_API_TOKEN_ENV}=")) {
                let val = val.trim();
                if !val.is_empty() {
                    return normalize_authorization_value(val);
                }
            }
        }
    }
    eprintln!("error: CODEXIS_USER_API_TOKEN not found in ~/.cdx/.env or environment");
    std::process::exit(2);
}

fn normalize_authorization_value(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(stripped) = trimmed.strip_prefix("Authorization:") {
        return stripped.trim().to_string();
    }
    trimmed.to_string()
}
