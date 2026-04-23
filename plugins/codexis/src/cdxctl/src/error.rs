use std::fmt;

/// Exit codes for cdxctl
pub const EXIT_NETWORK: i32 = 2;
pub const EXIT_GRAPHQL: i32 = 3;
pub const EXIT_PARSE: i32 = 4;

#[derive(Debug)]
pub enum CdxctlError {
    Network(String),
    GraphQL(Vec<String>),
    Parse(String),
}

impl CdxctlError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CdxctlError::Network(_) => EXIT_NETWORK,
            CdxctlError::GraphQL(_) => EXIT_GRAPHQL,
            CdxctlError::Parse(_) => EXIT_PARSE,
        }
    }
}

impl fmt::Display for CdxctlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CdxctlError::Network(msg) => write!(f, "Network error: {msg}"),
            CdxctlError::GraphQL(errors) => {
                write!(f, "GraphQL error: {}", errors.join("; "))
            }
            CdxctlError::Parse(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for CdxctlError {}

impl From<reqwest::Error> for CdxctlError {
    fn from(e: reqwest::Error) -> Self {
        CdxctlError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for CdxctlError {
    fn from(e: serde_json::Error) -> Self {
        CdxctlError::Parse(e.to_string())
    }
}

impl From<std::io::Error> for CdxctlError {
    fn from(e: std::io::Error) -> Self {
        CdxctlError::Network(format!("I/O error: {e}"))
    }
}
