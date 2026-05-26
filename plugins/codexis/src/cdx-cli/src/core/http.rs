use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;
use crate::core::sources::publish_from_body;

const CDX_SCHEME: &str = "cdx://";
const API_PREFIX: &str = "/rest/cdx-api/v2";

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
    handle_curl_output(&output, |body| {
        let (content, sources) = split_envelope(body);
        print_search_content(content, facet_mode)?;
        try_publish_sources(sources.as_ref());
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
    handle_curl_output(&output, |body| {
        let (content, sources) = split_envelope(body);
        print_get_content(content)?;
        try_publish_sources(sources.as_ref());
        Ok(())
    })
}

/// Best-effort: if running inside a chat VM with daemon access AND the upstream response
/// declared sources in its `sources` field, attach them to the active tool-call. Any failure
/// is logged to stderr and never propagates — sources are a side channel; the agent's
/// primary response (already on stdout) must not be affected.
fn try_publish_sources(sources: Option<&serde_json::Value>) {
    let Some(sources) = sources else {
        return;
    };
    let Some(daemon) = DaemonConfig::load() else {
        return;
    };
    if let Err(err) = publish_from_body(sources, &daemon) {
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
    F: Fn(&[u8]) -> Result<(), CliError>,
{
    let body = &output.stdout[..];
    match output.status.code() {
        Some(0) => handle(body),
        Some(code) => {
            if !body.is_empty() {
                handle(body)?;
            }
            Err(CliError::RequestFailed { code })
        }
        None => Err(CliError::CommandTerminated { command: "curl" }),
    }
}

/// Parses the v2 envelope `{content, sources}`. Returns `(content_json, sources_value)`.
/// If the body is not a JSON object with a `content` field, falls back to returning the
/// raw body as `content` and `None` sources — keeps the CLI usable against non-v2
/// responses or error payloads.
fn split_envelope(body: &[u8]) -> (EnvelopeContent, Option<serde_json::Value>) {
    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (EnvelopeContent::Raw(body.to_vec()), None);
    };
    let serde_json::Value::Object(ref mut map) = value else {
        return (EnvelopeContent::Json(value), None);
    };
    if !map.contains_key("content") {
        return (EnvelopeContent::Json(serde_json::Value::Object(map.clone())), None);
    }
    let sources = map.remove("sources");
    let content = map.remove("content").unwrap_or(serde_json::Value::Null);
    (EnvelopeContent::Json(content), sources)
}

enum EnvelopeContent {
    Json(serde_json::Value),
    Raw(Vec<u8>),
}

fn print_search_content(content: EnvelopeContent, facet_mode: SearchFacetMode) -> Result<(), CliError> {
    match content {
        EnvelopeContent::Json(mut value) => {
            apply_facet_mode(&mut value, facet_mode);
            let rendered = serde_json::to_string(&value)
                .unwrap_or_else(|_| String::from_utf8_lossy(b"").into_owned());
            write_stdout(&rendered)
        }
        EnvelopeContent::Raw(bytes) => {
            let rendered = String::from_utf8_lossy(&bytes).into_owned();
            write_stdout(&rendered)
        }
    }
}

fn print_get_content(content: EnvelopeContent) -> Result<(), CliError> {
    match content {
        EnvelopeContent::Json(serde_json::Value::String(s)) => write_stdout(&s),
        EnvelopeContent::Json(value) => {
            let rendered = serde_json::to_string(&value)
                .unwrap_or_else(|_| String::new());
            write_stdout(&rendered)
        }
        EnvelopeContent::Raw(bytes) => {
            let rendered = String::from_utf8_lossy(&bytes).into_owned();
            write_stdout(&rendered)
        }
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
        build_cdx_url(base_url, resource)?,
    ])
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
                "https://app.codexis.cz/rest/cdx-api/v2/search/JD",
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
            "https://app.codexis.cz/rest/cdx-api/v2/search/JD?fullFacets=true"
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
                "https://app.codexis.cz/rest/cdx-api/v2/doc/CR10_2026_01_01/text",
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
        assert_eq!(args[url_pos], "https://app.codexis.cz/rest/cdx-api/v2");
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
    fn split_envelope_extracts_content_and_sources() {
        let body = br#"{"content":{"results":[{"docId":"JD1"}]},"sources":[{"url":"https://app.codexis.cz/doc/JD1","title":"x"}]}"#;
        let (content, sources) = split_envelope(body);
        match content {
            EnvelopeContent::Json(value) => {
                assert_eq!(value["results"][0]["docId"], "JD1");
            }
            EnvelopeContent::Raw(_) => panic!("expected json content"),
        }
        let sources = sources.expect("expected sources");
        assert_eq!(sources[0]["url"], "https://app.codexis.cz/doc/JD1");
    }

    #[test]
    fn split_envelope_handles_string_content() {
        let body = br#"{"content":"plain text body","sources":[]}"#;
        let (content, sources) = split_envelope(body);
        match content {
            EnvelopeContent::Json(serde_json::Value::String(s)) => {
                assert_eq!(s, "plain text body");
            }
            _ => panic!("expected string json content"),
        }
        assert!(sources.is_some());
    }

    #[test]
    fn split_envelope_passes_through_non_envelope_json() {
        let body = br#"{"some":"other","shape":1}"#;
        let (content, sources) = split_envelope(body);
        match content {
            EnvelopeContent::Json(value) => {
                assert_eq!(value["some"], "other");
                assert_eq!(value["shape"], 1);
            }
            EnvelopeContent::Raw(_) => panic!("expected json content"),
        }
        assert!(sources.is_none());
    }

    #[test]
    fn split_envelope_passes_through_non_json_bodies() {
        let body = b"not json at all";
        let (content, sources) = split_envelope(body);
        match content {
            EnvelopeContent::Raw(bytes) => assert_eq!(bytes.as_slice(), body),
            EnvelopeContent::Json(_) => panic!("expected raw content"),
        }
        assert!(sources.is_none());
    }

    #[test]
    fn print_search_content_strips_available_filters_when_hidden() {
        let envelope: serde_json::Value = serde_json::from_slice(
            br#"{"content":{"results":[{"docId":"JD1"}],"availableFilters":[{"key":"court"}]},"sources":[]}"#,
        )
        .unwrap();
        let mut content_value = envelope
            .as_object()
            .unwrap()
            .get("content")
            .unwrap()
            .clone();
        apply_facet_mode(&mut content_value, SearchFacetMode::Hidden);
        assert!(content_value.as_object().unwrap().get("availableFilters").is_none());
        assert_eq!(content_value["results"][0]["docId"], "JD1");
    }

    #[test]
    fn apply_facet_mode_summary_keeps_available_filters() {
        let mut value: serde_json::Value = serde_json::from_slice(
            br#"{"results":[{"docId":"JD1"}],"availableFilters":[{"key":"court"}]}"#,
        )
        .unwrap();
        apply_facet_mode(&mut value, SearchFacetMode::Summary);
        assert_eq!(value["availableFilters"][0]["key"], "court");
    }
}
