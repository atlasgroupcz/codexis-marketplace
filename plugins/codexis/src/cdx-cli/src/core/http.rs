use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::error::CliError;

const CDX_SCHEME: &str = "cdx://";
const API_PREFIX: &str = "/rest/cdx-api";

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
    handle_curl_output(&output, |body| print_response_body(body, facet_mode))
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
    handle_curl_output(&output, print_raw_response_body)
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

        assert_eq!(
            args[8],
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

        assert_eq!(args[4], "https://app.codexis.cz/rest/cdx-api");
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
}
