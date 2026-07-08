use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;
use crate::core::sources::publish_from_body;

const REGISTR_SMLUV_SCHEME: &str = "registr-smluv://";

/// Display ID prefix -> API path (Czech smlouvy domain)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("CZRS", "CZ/smlouvy"),
];

pub(crate) fn execute_search_request(
    base_url: &str,
    auth_header: Option<&str>,
    source_code: &str,
    payload: &str,
    dry_run: bool,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<(), CliError> {
    let curl_args = build_search_curl_args(base_url, auth_header, source_code, payload, sort, order);

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, handle_api_response)
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    let curl_args = build_get_curl_args(base_url, auth_header, resource)?;

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, handle_api_response)
}

fn execute_curl(curl_args: &[String]) -> Result<Output, CliError> {
    Command::new("curl")
        .args(curl_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(CliError::CurlSpawn)
}

fn handle_curl_output<F>(output: &Output, print_body: F) -> Result<(), CliError>
where
    F: Fn(&[u8]) -> Result<(), CliError>,
{
    match output.status.code() {
        Some(0) => print_body(&output.stdout),
        Some(code) => {
            if !output.stdout.is_empty() {
                print_body(&output.stdout)?;
            }
            Err(CliError::RequestFailed { code })
        }
        None => Err(CliError::CommandTerminated { command: "curl" }),
    }
}

/// Unwrap the envelope, print content, best-effort publish sources.
fn handle_api_response(body: &[u8]) -> Result<(), CliError> {
    let raw = String::from_utf8_lossy(body);
    let (content, sources) = split_envelope_str(&raw);
    write_stdout(&content)?;
    try_publish_sources(sources.as_ref());
    Ok(())
}

/// Best-effort sources publish: any failure logs to stderr, never propagates.
fn try_publish_sources(sources: Option<&serde_json::Value>) {
    let Some(sources) = sources else {
        return;
    };
    let Some(daemon) = DaemonConfig::load() else {
        return;
    };
    if let Err(err) = publish_from_body(sources, &daemon) {
        eprintln!("registr-smluv: failed to attach sources to chat: {err}");
    }
}

fn write_stdout(rendered: &str) -> Result<(), CliError> {
    if rendered.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(rendered.as_bytes())
        .map_err(|source| CliError::Io {
            context: "failed to write response to stdout".to_string(),
            source,
        })?;
    if !rendered.ends_with('\n') {
        stdout.write_all(b"\n").map_err(|source| CliError::Io {
            context: "failed to write trailing newline to stdout".to_string(),
            source,
        })?;
    }
    Ok(())
}

fn build_search_curl_args(
    base_url: &str,
    auth_header: Option<&str>,
    source_code: &str,
    payload: &str,
    sort: Option<&str>,
    order: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "-sS".to_string(),
        "--fail-with-body".to_string(),
    ];
    if let Some(header) = auth_header {
        args.push("-H".to_string());
        args.push(header.to_string());
    }
    args.push("-X".to_string());
    args.push("POST".to_string());
    args.push("-H".to_string());
    args.push("Content-Type: application/json".to_string());
    args.push(build_search_url(base_url, source_code, sort, order));
    args.push("-d".to_string());
    args.push(payload.to_string());
    args
}

fn build_get_curl_args(base_url: &str, auth_header: Option<&str>, resource: &str) -> Result<Vec<String>, CliError> {
    let mut args = vec![
        "-sS".to_string(),
        "--fail-with-body".to_string(),
    ];
    if let Some(header) = auth_header {
        args.push("-H".to_string());
        args.push(header.to_string());
    }
    args.push(build_cdx_url(base_url, resource)?);
    Ok(args)
}

fn build_search_url(
    base_url: &str,
    source_code: &str,
    sort: Option<&str>,
    order: Option<&str>,
) -> String {
    let api_path = lookup_search_prefix(source_code)
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

fn build_cdx_url(base_url: &str, resource: &str) -> Result<String, CliError> {
    let Some(rest) = resource.strip_prefix(REGISTR_SMLUV_SCHEME) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a registr-smluv:// URL, got: {resource}"
        )));
    };

    resolve_cdx_url(base_url, rest).map_err(CliError::InvalidCdxUrl)
}

/// Resolve a registr-smluv:// path (everything after "registr-smluv://") into a full HTTP URL.
/// Supported routes: `doc/<DOC_ID>/meta` (contract record) and `resolve/<DISPLAY_ID>`.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    if let Some(rest) = cdx_path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = lookup_display_id_prefix(id)?;
        Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}"))
    } else if let Some(rest) = cdx_path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        Ok(format!("{base}/resolve/{rest}"))
    } else {
        Err(format!("Unknown registr-smluv:// path: {cdx_path}"))
    }
}

/// Split the `{content, sources}` envelope. Returns `(content_to_print, sources)`.
/// Falls back to `(whole_body, None)` for a body that isn't a `{content, …}` envelope.
pub(crate) fn split_envelope_str(body: &str) -> (String, Option<serde_json::Value>) {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Object(mut map)) if map.contains_key("content") => {
            let sources = map.remove("sources");
            let printed = match map.remove("content") {
                Some(serde_json::Value::String(s)) => s,
                Some(other) => serde_json::to_string(&other).unwrap_or_default(),
                None => String::new(),
            };
            (printed, sources)
        }
        _ => (body.to_string(), None),
    }
}

fn lookup_search_prefix(code: &str) -> Result<&'static str, String> {
    for &(prefix, api_path) in ID_PREFIXES {
        if prefix == code {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown search prefix: {code}"))
}

fn lookup_display_id_prefix(id: &str) -> Result<&'static str, String> {
    for &(prefix, api_path) in ID_PREFIXES {
        if id.starts_with(prefix) {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown display ID prefix in: {id}"))
}

fn redact_curl_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            if is_authorization_header(arg) {
                redact_authorization_header(arg)
            } else {
                arg.clone()
            }
        })
        .collect()
}

fn is_authorization_header(value: &str) -> bool {
    value
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("authorization:")
}

fn redact_authorization_header(header: &str) -> String {
    let value = header
        .split_once(':')
        .map(|(_, value)| value.trim())
        .unwrap_or_default();

    if value.to_ascii_lowercase().starts_with("bearer ") {
        "Authorization: Bearer <redacted>".to_string()
    } else if let Some((scheme, _)) = value.split_once(' ') {
        format!("Authorization: {scheme} <redacted>")
    } else {
        "Authorization: <redacted>".to_string()
    }
}

fn format_command(program: &str, args: &[String]) -> String {
    let mut rendered = Vec::with_capacity(args.len() + 1);
    rendered.push(shell_escape(program));
    rendered.extend(args.iter().map(|arg| shell_escape(arg)));
    rendered.join(" ")
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    if value.chars().all(is_shell_safe_char) {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn is_shell_safe_char(ch: char) -> bool {
    matches!(
        ch,
        'a'..='z'
            | 'A'..='Z'
            | '0'..='9'
            | '-'
            | '_'
            | '.'
            | '/'
            | ':'
            | '='
            | '@'
            | '%'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    #[test]
    fn search_curl_args_are_built_as_post_json_request() {
        let args = build_search_curl_args(
            BASE,
            None,
            "CZRS",
            r#"{"q":"silnice","limit":5}"#,
            None,
            None,
        );

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "https://api.example.com/api/CZ/smlouvy/search",
                "-d",
                r#"{"q":"silnice","limit":5}"#,
            ]
        );
    }

    #[test]
    fn search_url_includes_sort_and_order_query_params() {
        let args = build_search_curl_args(
            BASE,
            None,
            "CZRS",
            r#"{"q":"test"}"#,
            Some("date"),
            Some("desc"),
        );

        assert_eq!(
            args[6],
            "https://api.example.com/api/CZ/smlouvy/search?sort=date&order=desc"
        );
    }

    #[test]
    fn search_url_includes_sort_only() {
        let url = build_search_url(BASE, "CZRS", Some("relevance"), None);
        assert_eq!(
            url,
            "https://api.example.com/api/CZ/smlouvy/search?sort=relevance"
        );
    }

    #[test]
    fn get_curl_args_translate_registr_smluv_resource_to_api_url() {
        let args =
            build_get_curl_args(BASE, None, "registr-smluv://doc/CZRS1234/meta").unwrap();

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "https://api.example.com/api/CZ/smlouvy/doc/CZRS1234/meta",
            ]
        );
    }

    #[test]
    fn get_curl_args_reject_non_registr_smluv_urls() {
        let error =
            build_get_curl_args(BASE, None, "https://example.com/doc/1").unwrap_err();

        assert!(matches!(error, CliError::InvalidCdxUrl(_)));
    }

    #[test]
    fn shell_escape_quotes_json_for_dry_run_output() {
        let rendered = format_command(
            "curl",
            &[
                "-d".to_string(),
                r#"{"q":"silnice","limit":5}"#.to_string(),
            ],
        );

        assert_eq!(
            rendered,
            r#"curl -d '{"q":"silnice","limit":5}'"#
        );
    }

    #[test]
    fn search_curl_args_include_auth_header_when_present() {
        let args = build_search_curl_args(
            BASE,
            Some("Authorization: Bearer token"),
            "CZRS",
            r#"{"q":"test"}"#,
            None,
            None,
        );

        assert_eq!(args[2], "-H");
        assert_eq!(args[3], "Authorization: Bearer token");
    }

    #[test]
    fn get_curl_args_include_auth_header_when_present() {
        let args = build_get_curl_args(
            BASE,
            Some("Authorization: Bearer token"),
            "registr-smluv://doc/CZRS1234/meta",
        )
        .unwrap();

        assert_eq!(args[2], "-H");
        assert_eq!(args[3], "Authorization: Bearer token");
    }

    #[test]
    fn dry_run_output_redacts_authorization_header() {
        let rendered = format_command(
            "curl",
            &redact_curl_args(&[
                "-H".to_string(),
                "Authorization: Bearer super-secret-token".to_string(),
                "-d".to_string(),
                r#"{"q":"test"}"#.to_string(),
            ]),
        );

        assert_eq!(
            rendered,
            "curl -H 'Authorization: Bearer <redacted>' -d '{\"q\":\"test\"}'"
        );
    }

    // --- URL resolution tests ---

    #[test]
    fn doc_czrs_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZRS1234/meta"),
            Ok(format!("{BASE}/CZ/smlouvy/doc/CZRS1234/meta"))
        );
    }

    #[test]
    fn doc_unknown_prefix() {
        assert!(resolve_cdx_url(BASE, "doc/UNKNOWN123/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZRS1234").is_err());
    }

    #[test]
    fn doc_empty_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZRS1234/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/CZRS1234"),
            Ok(format!("{BASE}/resolve/CZRS1234"))
        );
    }

    #[test]
    fn resolve_empty() {
        assert!(resolve_cdx_url(BASE, "resolve/").is_err());
    }

    #[test]
    fn unknown_route() {
        assert!(resolve_cdx_url(BASE, "foo/bar").is_err());
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "resolve/CZRS1"),
            Ok("https://api.example.com/api/resolve/CZRS1".to_string())
        );
    }

    // --- envelope splitting ---

    #[test]
    fn split_envelope_extracts_content_and_sources() {
        let body = r#"{"content":{"results":[{"docId":"CZRS1"}]},"sources":[{"__kind":"url","url":"https://x/y.pdf"}]}"#;
        let (content, sources) = split_envelope_str(body);
        assert!(content.contains("\"results\""));
        assert!(sources.is_some());
        assert_eq!(sources.unwrap()[0]["url"], "https://x/y.pdf");
    }

    #[test]
    fn split_envelope_passthrough_when_no_content_key() {
        let body = r#"{"results":[1,2]}"#;
        let (content, sources) = split_envelope_str(body);
        assert_eq!(content, body);
        assert!(sources.is_none());
    }
}
