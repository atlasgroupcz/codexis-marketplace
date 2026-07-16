//! Config for leciva. The shared pieces (`SearchConfig`, env/auth helpers) live
//! in `cdx-common`; the plugin keeps its own API-URL env var and public default.

pub(crate) use cdx_common::config::SearchConfig as Config;

pub(crate) const CODEXIS_PLUGIN_LECIVA_API_URL_ENV: &str = "CODEXIS_PLUGIN_LECIVA_API_URL";

/// The medicines registry is served from the codexis backend. The env var above
/// only overrides this (e.g. a VM/staging instance); if unset, use this default
/// so the plugin works out of the box.
pub(crate) const DEFAULT_API_BASE_URL: &str = "https://codexis.ai/sources/api";
