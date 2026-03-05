use crate::error::CdxctlError;
use serde_json::{json, Value};

const DEFAULT_API_URL: &str = "http://localhost:38083/graphql";

pub struct GraphQLClient {
    url: String,
    client: reqwest::blocking::Client,
}

impl GraphQLClient {
    pub fn new() -> Self {
        let url = std::env::var("CDXCTL_API_URL")
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        GraphQLClient {
            url,
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
            .json(&body)
            .send()?;

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            return Err(CdxctlError::Network(format!(
                "HTTP {status}: {text}"
            )));
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
