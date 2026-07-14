//! The CODEXIS search/get request flow, shared by the search plugins.
//!
//! The flow (build the request URL, shell out to curl, unwrap the envelope,
//! print content and publish sources) is identical across plugins. What varies
//! is captured by the [`SearchApi`] trait: the source-code table (data), the
//! `cdx://` scheme, the plugin name, and two behaviours — resolving a `cdx://`
//! resource URL and rewriting attachment links. A plugin implements the trait
//! and calls [`execute_search_request`] / [`execute_get_request`].

use crate::config::DaemonConfig;
use crate::curl::{
    execute_curl, format_command, handle_curl_output, redact_curl_args, split_envelope_str,
    write_stdout,
};
use crate::error::CliError;
use crate::sources::publish_from_body;

/// Per-plugin specifics the search engine needs. Everything else (the request
/// flow) is provided by this module.
pub trait SearchApi {
    /// Source-code → API-path table, e.g. `&[("ATBR", "AT/bundesrecht")]`.
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)];
    /// The plugin's cdx URL scheme, e.g. `"cdx-at://"`.
    fn cdx_scheme(&self) -> &'static str;
    /// The plugin name, used in the sources-publish log line.
    fn plugin_name(&self) -> &'static str;
    /// Resolves a cdx path (everything after the scheme) into a full HTTP URL.
    fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String>;
    /// Rewrites attachment links in a response body to absolute URLs.
    fn resolve_attachment_links(&self, body: &str, base_url: &str) -> String;
}

/// Looks up a source code in a prefix table; `Err` when the code is unknown.
pub fn lookup_prefix(
    table: &'static [(&'static str, &'static str)],
    code: &str,
) -> Result<&'static str, String> {
    for &(prefix, api_path) in table {
        if prefix == code {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown search prefix: {code}"))
}

/// Runs a source search: POST the payload to the source's `/search` endpoint.
pub fn execute_search_request(
    api: &impl SearchApi,
    base_url: &str,
    auth_header: Option<&str>,
    source_code: &str,
    payload: &str,
    dry_run: bool,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<(), CliError> {
    let curl_args =
        build_search_curl_args(api, base_url, auth_header, source_code, payload, sort, order);

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, |body| handle_api_response(api, body, base_url))
}

/// Fetches a `cdx://` resource: resolve it to an HTTP URL and GET it.
pub fn execute_get_request(
    api: &impl SearchApi,
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    let curl_args = build_get_curl_args(api, base_url, auth_header, resource)?;

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, |body| handle_api_response(api, body, base_url))
}

fn handle_api_response(api: &impl SearchApi, body: &[u8], base_url: &str) -> Result<(), CliError> {
    let raw = String::from_utf8_lossy(body);
    let resolved = api.resolve_attachment_links(&raw, base_url);
    let (content, sources) = split_envelope_str(&resolved);
    write_stdout(&content)?;
    try_publish_sources(api.plugin_name(), sources.as_ref());
    Ok(())
}

fn try_publish_sources(plugin_name: &str, sources: Option<&serde_json::Value>) {
    let Some(sources) = sources else {
        return;
    };
    let Some(daemon) = DaemonConfig::load() else {
        return;
    };
    if let Err(err) = publish_from_body(sources, &daemon) {
        eprintln!("{plugin_name}: failed to attach sources to chat: {err}");
    }
}

fn build_search_curl_args(
    api: &impl SearchApi,
    base_url: &str,
    auth_header: Option<&str>,
    source_code: &str,
    payload: &str,
    sort: Option<&str>,
    order: Option<&str>,
) -> Vec<String> {
    let mut args = vec!["-sS".to_string(), "--fail-with-body".to_string()];
    if let Some(header) = auth_header {
        args.push("-H".to_string());
        args.push(header.to_string());
    }
    args.push("-X".to_string());
    args.push("POST".to_string());
    args.push("-H".to_string());
    args.push("Content-Type: application/json".to_string());
    args.push(build_search_url(api, base_url, source_code, sort, order));
    args.push("-d".to_string());
    args.push(payload.to_string());
    args
}

fn build_get_curl_args(
    api: &impl SearchApi,
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
) -> Result<Vec<String>, CliError> {
    let mut args = vec!["-sS".to_string(), "--fail-with-body".to_string()];
    if let Some(header) = auth_header {
        args.push("-H".to_string());
        args.push(header.to_string());
    }
    args.push(build_cdx_url(api, base_url, resource)?);
    Ok(args)
}

fn build_search_url(
    api: &impl SearchApi,
    base_url: &str,
    source_code: &str,
    sort: Option<&str>,
    order: Option<&str>,
) -> String {
    let api_path = lookup_prefix(api.id_prefixes(), source_code)
        .unwrap_or_else(|_| panic!("unknown search source: {source_code}"));
    let base = base_url.trim_end_matches('/');
    let mut url = format!("{base}/{api_path}/search");

    let mut sep = '?';
    if let Some(sort) = sort {
        url.push(sep);
        url.push_str("sort=");
        url.push_str(sort);
        sep = '&';
    }
    if let Some(order) = order {
        url.push(sep);
        url.push_str("order=");
        url.push_str(order);
    }

    url
}

fn build_cdx_url(api: &impl SearchApi, base_url: &str, resource: &str) -> Result<String, CliError> {
    let Some(rest) = resource.strip_prefix(api.cdx_scheme()) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a {} URL, got: {resource}",
            api.cdx_scheme()
        )));
    };

    api.resolve_cdx_url(base_url, rest)
        .map_err(CliError::InvalidCdxUrl)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockApi;
    impl SearchApi for MockApi {
        fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
            &[("ATBR", "AT/bundesrecht"), ("ATJD", "AT/judikatur")]
        }
        fn cdx_scheme(&self) -> &'static str {
            "cdx-at://"
        }
        fn plugin_name(&self) -> &'static str {
            "test"
        }
        fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String> {
            Ok(format!("{base_url}/resolved/{cdx_path}"))
        }
        fn resolve_attachment_links(&self, body: &str, _base_url: &str) -> String {
            body.to_string()
        }
    }

    const BASE: &str = "https://api.example.com/api";

    #[test]
    fn lookup_prefix_maps_and_rejects() {
        assert_eq!(lookup_prefix(MockApi.id_prefixes(), "ATBR").unwrap(), "AT/bundesrecht");
        assert!(lookup_prefix(MockApi.id_prefixes(), "XXX").is_err());
    }

    #[test]
    fn search_url_includes_sort_and_order() {
        let args = build_search_curl_args(&MockApi, BASE, None, "ATBR", "{}", Some("date"), Some("desc"));
        assert!(args
            .iter()
            .any(|a| a == "https://api.example.com/api/AT/bundesrecht/search?sort=date&order=desc"));
    }

    #[test]
    fn search_curl_args_are_post_json_with_auth() {
        let args = build_search_curl_args(&MockApi, BASE, Some("Authorization: Bearer x"), "ATJD", "{\"q\":\"a\"}", None, None);
        assert!(args.contains(&"POST".to_string()));
        assert!(args.contains(&"Content-Type: application/json".to_string()));
        assert!(args.contains(&"Authorization: Bearer x".to_string()));
        assert!(args.contains(&"{\"q\":\"a\"}".to_string()));
    }

    #[test]
    fn get_url_uses_resolver_and_rejects_wrong_scheme() {
        let args = build_get_curl_args(&MockApi, BASE, None, "cdx-at://doc/1").unwrap();
        assert!(args.iter().any(|a| a.contains("/resolved/doc/1")));
        assert!(build_get_curl_args(&MockApi, BASE, None, "http://x").is_err());
    }
}
