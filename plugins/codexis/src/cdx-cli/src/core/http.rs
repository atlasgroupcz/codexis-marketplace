use std::collections::HashMap;
use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;
use crate::core::sources::publish_from_headers;

const CDX_SCHEME: &str = "cdx://";
const API_PREFIX: &str = "/rest/cdx-api";

/// Unique marker appended to curl stdout (via `-w`) so we can split the response body
/// from the JSON headers dump. The unit-separator bytes are virtually never present
/// in JSON responses or plain-text legal documents.
const HEADER_DUMP_DELIM: &str = "\n\x1fCDX_HEADER_JSON\x1f\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SearchFacetMode {
    #[default]
    Hidden,
    Summary,
    Full,
}

pub(crate) fn execute_search_request(
    base_url: &str,
    auth_header: &str,
    source_code: &str,
    payload: &str,
    dry_run: bool,
    facet_mode: SearchFacetMode,
) -> Result<(), CliError> {
    let curl_args = build_search_curl_args(base_url, auth_header, source_code, payload, facet_mode);

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, |body, headers| {
        print_response_body(body, facet_mode)?;
        try_publish_sources(headers);
        Ok(())
    })
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: &str,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    let curl_args = build_get_curl_args(base_url, auth_header, resource)?;

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let output = execute_curl(&curl_args)?;
    handle_curl_output(&output, |body, headers| {
        print_raw_response_body(body)?;
        try_publish_sources(headers);
        Ok(())
    })
}

/// Best-effort: if running inside a chat VM with daemon access AND the upstream response
/// declared sources via `X-Cdx-Sources`, attach them to the active tool-call. Any failure
/// is logged to stderr and never propagates — sources are a side channel; the agent's
/// primary response (already on stdout) must not be affected.
fn try_publish_sources(headers: &HashMap<String, String>) {
    let Some(daemon) = DaemonConfig::load() else {
        return;
    };
    if let Err(err) = publish_from_headers(headers, &daemon) {
        eprintln!("cdx-cli: failed to attach sources to chat: {err}");
    }
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

fn handle_curl_output<F>(output: &Output, handle: F) -> Result<(), CliError>
where
    F: Fn(&[u8], &HashMap<String, String>) -> Result<(), CliError>,
{
    let (body, headers) = split_curl_output(&output.stdout);
    match output.status.code() {
        Some(0) => handle(body, &headers),
        Some(code) => {
            if !body.is_empty() {
                handle(body, &headers)?;
            }
            Err(CliError::RequestFailed { code })
        }
        None => Err(CliError::CommandTerminated { command: "curl" }),
    }
}

/// Splits the curl stdout produced by `-w "<DELIM>%{header_json}"` into `(body, headers)`.
/// On older curl builds (or when our `-w` arg was not honored) the delimiter is absent
/// and we return all-body with empty headers.
fn split_curl_output(stdout: &[u8]) -> (&[u8], HashMap<String, String>) {
    let needle = HEADER_DUMP_DELIM.as_bytes();
    let Some(pos) = find_last_subslice(stdout, needle) else {
        return (stdout, HashMap::new());
    };
    let body = &stdout[..pos];
    let header_dump = &stdout[pos + needle.len()..];
    (body, parse_header_dump(header_dump))
}

fn find_last_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .rev()
        .find(|&start| &haystack[start..start + needle.len()] == needle)
}

/// curl `%{header_json}` writes a JSON object whose values are arrays of strings (one
/// per occurrence of the header). We flatten to the LAST value per name and lowercase
/// names so callers can look up headers case-insensitively.
fn parse_header_dump(bytes: &[u8]) -> HashMap<String, String> {
    let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return HashMap::new();
    };
    let serde_json::Value::Object(map) = parsed else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (name, value) in map {
        let last_value = match value {
            serde_json::Value::Array(items) => items
                .into_iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .next_back(),
            serde_json::Value::String(s) => Some(s),
            _ => None,
        };
        if let Some(v) = last_value {
            out.insert(name.to_ascii_lowercase(), v);
        }
    }
    out
}

fn print_response_body(body: &[u8], facet_mode: SearchFacetMode) -> Result<(), CliError> {
    let rendered = render_search_output(body, facet_mode);
    write_stdout(&rendered)
}

fn print_raw_response_body(body: &[u8]) -> Result<(), CliError> {
    let rendered = String::from_utf8_lossy(body).into_owned();
    write_stdout(&rendered)
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

fn render_search_output(body: &[u8], facet_mode: SearchFacetMode) -> String {
    let text = String::from_utf8_lossy(body);
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(mut value) => {
            apply_facet_mode(&mut value, facet_mode);
            serde_json::to_string(&value).unwrap_or_else(|_| text.into_owned())
        }
        Err(_) => text.into_owned(),
    }
}

fn apply_facet_mode(value: &mut serde_json::Value, facet_mode: SearchFacetMode) {
    if matches!(facet_mode, SearchFacetMode::Hidden) {
        if let Some(object) = value.as_object_mut() {
            object.remove("availableFilters");
        }
    }
}

fn build_search_curl_args(
    base_url: &str,
    auth_header: &str,
    source_code: &str,
    payload: &str,
    facet_mode: SearchFacetMode,
) -> Vec<String> {
    vec![
        "-sS".to_string(),
        "--fail-with-body".to_string(),
        "-H".to_string(),
        auth_header.to_string(),
        "-X".to_string(),
        "POST".to_string(),
        "-H".to_string(),
        "Content-Type: application/json".to_string(),
        "-w".to_string(),
        header_dump_write_format(),
        build_search_url(base_url, source_code, facet_mode),
        "-d".to_string(),
        payload.to_string(),
    ]
}

fn build_get_curl_args(
    base_url: &str,
    auth_header: &str,
    resource: &str,
) -> Result<Vec<String>, CliError> {
    Ok(vec![
        "-sS".to_string(),
        "--fail-with-body".to_string(),
        "-H".to_string(),
        auth_header.to_string(),
        "-w".to_string(),
        header_dump_write_format(),
        build_cdx_url(base_url, resource)?,
    ])
}

fn header_dump_write_format() -> String {
    format!("{HEADER_DUMP_DELIM}%{{header_json}}")
}

fn build_search_url(base_url: &str, source_code: &str, facet_mode: SearchFacetMode) -> String {
    let mut url = build_api_url(base_url, &format!("search/{source_code}"));
    if matches!(facet_mode, SearchFacetMode::Full) {
        url.push_str("?fullFacets=true");
    }
    url
}

fn build_cdx_url(base_url: &str, resource: &str) -> Result<String, CliError> {
    let Some(rest) = resource.strip_prefix(CDX_SCHEME) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a cdx:// URL, got: {resource}"
        )));
    };

    Ok(build_api_url(base_url, rest))
}

fn build_api_url(base_url: &str, rest: &str) -> String {
    let path = rest.trim_start_matches('/');
    if path.is_empty() {
        format!("{base_url}{API_PREFIX}")
    } else {
        format!("{base_url}{API_PREFIX}/{path}")
    }
}

fn format_command(program: &str, args: &[String]) -> String {
    let mut rendered = Vec::with_capacity(args.len() + 1);
    rendered.push(shell_escape(program));
    rendered.extend(args.iter().map(|arg| shell_escape(arg)));
    rendered.join(" ")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_curl_args_are_built_as_post_json_request() {
        let args = build_search_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "JD",
            r#"{"query":"náhrada škody","limit":5}"#,
            SearchFacetMode::Hidden,
        );

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "-H",
                "Authorization: Bearer token",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-w",
                &header_dump_write_format(),
                "https://app.codexis.cz/rest/cdx-api/search/JD",
                "-d",
                r#"{"query":"náhrada škody","limit":5}"#,
            ]
        );
    }

    #[test]
    fn full_facet_mode_adds_query_parameter_to_search_url() {
        let args = build_search_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "JD",
            r#"{"query":"pes","limit":10}"#,
            SearchFacetMode::Full,
        );

        let url_pos = args
            .iter()
            .position(|a| a.starts_with("https://"))
            .expect("expected an HTTPS url arg");
        assert_eq!(
            args[url_pos],
            "https://app.codexis.cz/rest/cdx-api/search/JD?fullFacets=true"
        );
    }

    #[test]
    fn get_curl_args_translate_cdx_resource_to_api_url() {
        let args = build_get_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "cdx://doc/CR10_2026_01_01/text",
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "-H",
                "Authorization: Bearer token",
                "-w",
                &header_dump_write_format(),
                "https://app.codexis.cz/rest/cdx-api/doc/CR10_2026_01_01/text",
            ]
        );
    }

    #[test]
    fn get_curl_args_accept_empty_cdx_root() {
        let args = build_get_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "cdx://",
        )
        .unwrap();

        let url_pos = args
            .iter()
            .position(|a| a.starts_with("https://"))
            .expect("expected an HTTPS url arg");
        assert_eq!(args[url_pos], "https://app.codexis.cz/rest/cdx-api");
    }

    #[test]
    fn get_curl_args_reject_non_cdx_urls() {
        let error = build_get_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "https://example.com/doc/1",
        )
        .unwrap_err();

        assert!(matches!(error, CliError::InvalidCdxUrl(_)));
    }

    #[test]
    fn shell_escape_quotes_json_for_dry_run_output() {
        let rendered = format_command(
            "curl",
            &[
                "-d".to_string(),
                r#"{"query":"náhrada škody","limit":5}"#.to_string(),
            ],
        );

        assert_eq!(
            rendered,
            "curl -d '{\"query\":\"náhrada škody\",\"limit\":5}'"
        );
    }

    #[test]
    fn dry_run_output_redacts_authorization_header() {
        let rendered = format_command(
            "curl",
            &redact_curl_args(&[
                "-H".to_string(),
                "Authorization: Bearer super-secret-token".to_string(),
                "-d".to_string(),
                r#"{"query":"test"}"#.to_string(),
            ]),
        );

        assert_eq!(
            rendered,
            "curl -H 'Authorization: Bearer <redacted>' -d '{\"query\":\"test\"}'"
        );
    }

    #[test]
    fn default_output_hides_available_filters() {
        let rendered = render_search_output(
            br#"{"results":[{"docId":"JD1"}],"availableFilters":[{"key":"court"}],"limit":1}"#,
            SearchFacetMode::Hidden,
        );

        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert!(value.get("availableFilters").is_none());
        assert_eq!(value["results"][0]["docId"], "JD1");
    }

    #[test]
    fn facet_output_keeps_available_filters() {
        let rendered = render_search_output(
            br#"{"results":[{"docId":"JD1"}],"availableFilters":[{"key":"court"}],"limit":1}"#,
            SearchFacetMode::Summary,
        );

        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(value["availableFilters"][0]["key"], "court");
    }

    #[test]
    fn facet_output_tolerates_missing_available_filters() {
        let rendered = render_search_output(
            br#"{"results":[{"docId":"JD1"}],"limit":1}"#,
            SearchFacetMode::Summary,
        );

        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert!(value.get("availableFilters").is_none());
        assert_eq!(value["results"][0]["docId"], "JD1");
    }

    #[test]
    fn full_facet_output_tolerates_missing_available_filters() {
        let rendered = render_search_output(
            br#"{"results":[{"docId":"JD1"}],"limit":1}"#,
            SearchFacetMode::Full,
        );

        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert!(value.get("availableFilters").is_none());
        assert_eq!(value["results"][0]["docId"], "JD1");
    }

    #[test]
    fn invalid_json_output_is_printed_raw() {
        let rendered = render_search_output(br#"not-json"#, SearchFacetMode::Hidden);
        assert_eq!(rendered, "not-json");
    }

    #[test]
    fn split_curl_output_extracts_body_and_headers_from_dump() {
        let mut stdout = b"{\"results\":[]}".to_vec();
        stdout.extend_from_slice(HEADER_DUMP_DELIM.as_bytes());
        stdout.extend_from_slice(
            br#"{"x-cdx-sources":["%5B%5D"],"content-type":["application/json"]}"#,
        );

        let (body, headers) = split_curl_output(&stdout);

        assert_eq!(body, b"{\"results\":[]}");
        assert_eq!(headers.get("x-cdx-sources"), Some(&"%5B%5D".to_string()));
        assert_eq!(
            headers.get("content-type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn split_curl_output_returns_body_only_when_delim_missing() {
        let stdout = b"plain text without delim";

        let (body, headers) = split_curl_output(stdout);

        assert_eq!(body, stdout);
        assert!(headers.is_empty());
    }

    #[test]
    fn split_curl_output_lowercases_header_names_and_keeps_last_value() {
        let mut stdout = b"body".to_vec();
        stdout.extend_from_slice(HEADER_DUMP_DELIM.as_bytes());
        stdout.extend_from_slice(br#"{"X-Cdx-Sources":["first","last"]}"#);

        let (_, headers) = split_curl_output(&stdout);

        assert_eq!(headers.get("x-cdx-sources"), Some(&"last".to_string()));
    }
}
