use crate::error::CdxctlError;
use reqwest::blocking::multipart::Form;
use serde_json::{json, Value};
use std::time::Duration;

const DEFAULT_API_URL: &str = "http://localhost:8086/graphql";

/// The shared client: ring, and the roots compiled into the binary.
///
/// A failure here means the TLS policy itself is broken, which no command can
/// work around - so it exits rather than handing back a client that would fail
/// every request for a reason the user cannot see.
fn http_client() -> reqwest::blocking::Client {
    cdx_common::reqwest_client::client_builder()
        .build()
        .unwrap_or_else(|e| {
            eprintln!("error: could not build the HTTP client: {e}");
            std::process::exit(2);
        })
}

pub struct GraphQLClient {
    url: String,
    auth: String,
    client: reqwest::blocking::Client,
}

impl GraphQLClient {
    pub fn new() -> Self {
        let url = std::env::var("CODEXIS_PUBLIC_DAEMON_URL")
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let auth = load_api_jwt_auth();
        GraphQLClient {
            url,
            auth,
            client: http_client(),
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

    /// POST a multipart body (used for /rest/v1/plugin/email/send and for
    /// /rest/files/upload-multiple, which takes the isolation as a query argument).
    pub fn rest_post_multipart(
        &self,
        path: &str,
        form: Form,
        query: &[(&str, String)],
        headers: &[(&str, String)],
    ) -> Result<Value, CdxctlError> {
        let url = format!("{}{}", self.rest_base_url(), path);
        let mut request = self
            .client
            .post(&url)
            .header("Authorization", &self.auth)
            .header("Accept", "application/json");
        if !query.is_empty() {
            request = request.query(query);
        }
        for (name, value) in headers {
            request = request.header(*name, value);
        }
        let response = request.multipart(form).send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// POST a JSON body (used for /rest/v1/plugin/bug-report).
    pub fn rest_post_json(&self, path: &str, body: &Value) -> Result<Value, CdxctlError> {
        let url = format!("{}{}", self.rest_base_url(), path);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .json(body)
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CdxctlError::Unsupported(format!(
                "this daemon has no {path} endpoint"
            )));
        }
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// GET a JSON document from a REST endpoint (used for /rest/v1/vm/packages/{name}).
    pub fn rest_get_json(&self, path: &str) -> Result<Value, CdxctlError> {
        let response = self
            .client
            .get(format!("{}{}", self.rest_base_url(), path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(30))
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// POST a JSON body and wait up to `timeout` for the daemon to finish the work
    /// (used for /rest/v1/vm/packages, where the daemon runs apt synchronously).
    pub fn rest_post_json_waiting(
        &self,
        path: &str,
        body: &Value,
        timeout: Duration,
    ) -> Result<Value, CdxctlError> {
        let response = self
            .client
            .post(format!("{}{}", self.rest_base_url(), path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json")
            .timeout(timeout)
            .json(body)
            .send()?;
        let status = response.status();
        let text = response.text()?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CdxctlError::Unsupported(format!(
                "this daemon has no {path} endpoint"
            )));
        }
        if !status.is_success() {
            return Err(CdxctlError::Network(format!("HTTP {status}: {text}")));
        }
        serde_json::from_str(&text).map_err(CdxctlError::from)
    }

    /// Test-only constructor: point the client at a loopback stub. `new()` cannot
    /// be used in tests because it exits the process when ~/.cdx/env/secret is missing.
    #[cfg(test)]
    pub(crate) fn with_endpoint(url: String, auth: String) -> Self {
        GraphQLClient {
            url,
            auth,
            client: http_client(),
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
    cdx_common::daemon_rest::daemon_bearer().unwrap_or_else(|| {
        eprintln!("error: no auth available (missing ~/.cdx/env/secret)");
        std::process::exit(2);
    })
}
