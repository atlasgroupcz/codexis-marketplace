//! Config for register-zmluv. The shared pieces (`DaemonConfig`, the
//! search-config loader, env/auth helpers) live in the `cdx-common` crate; the
//! plugin keeps its own API-URL env var and public default endpoint.

pub(crate) use cdx_common::config::SearchConfig as Config;

pub(crate) const CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL_ENV: &str =
    "CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL";

/// The Register of Contracts is always served from the codexis backend. The env
/// var above only overrides this (e.g. to point at a VM/staging instance); if
/// unset, use this default so the plugin works out of the box.
pub(crate) const DEFAULT_API_BASE_URL: &str = "https://codexis.ai/sources/api";
