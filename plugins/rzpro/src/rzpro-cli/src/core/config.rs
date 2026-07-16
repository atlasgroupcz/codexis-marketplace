//! Config for rzpro. Shared pieces live in `cdx-common`; the plugin keeps its
//! own API-URL env var and public default endpoint.

pub(crate) use cdx_common::config::SearchConfig as Config;

pub(crate) const CODEXIS_PLUGIN_RZPRO_API_URL_ENV: &str = "CODEXIS_PLUGIN_RZPRO_API_URL";

/// The device registry is served from the codexis backend. The env var above only
/// overrides this (e.g. staging); unset -> use this default.
pub(crate) const DEFAULT_API_BASE_URL: &str = "https://codexis.ai/sources/api";
