//! Config for cdx-at. The shared pieces (`DaemonConfig`, the search-config
//! loader, env/auth helpers) live in the `cdx-common` crate; only the
//! plugin-specific API-URL env var name stays here.

pub(crate) use cdx_common::config::SearchConfig as Config;

pub(crate) const CODEXIS_PLUGIN_AT_API_URL_ENV: &str = "CODEXIS_PLUGIN_AT_API_URL";
