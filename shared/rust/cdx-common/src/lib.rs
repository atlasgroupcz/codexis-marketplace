//! Shared building blocks for the codexis-marketplace Rust CLI plugins.
//!
//! These plugins are separate Cargo crates (no workspace) that each ship a
//! single binary. Historically they copy-pasted the same daemon-publishing
//! plumbing, config loading and error type. This crate holds the parts that
//! were byte-identical (or differed only by a plugin-specific env var) so the
//! plugins can depend on it by relative path instead of duplicating code.
//!
//! Nothing here is plugin-specific. Anything that genuinely differs per plugin
//! (search schemas, CLI commands, the curl search engine) stays in the plugin.

pub mod config;
pub mod daemon_rest;
pub mod error;

#[cfg(feature = "curl")]
pub mod curl;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "schema")]
pub mod schema;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "search-engine")]
pub mod search_engine;

#[cfg(feature = "sources")]
pub mod sources;

#[cfg(feature = "ico")]
pub mod ico;
