use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::error::CliError;

const CDX_CZ_SPP_SCHEME: &str = "cdx-cz-spp://";

/// Display ID prefix -> API path (Czech sbirkapp domain)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("CZSB", "CZ/sbirkapp"),
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
    handle_curl_output(&output, print_raw_response_body)
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
    let Some(rest) = resource.strip_prefix(CDX_CZ_SPP_SCHEME) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a cdx-cz-spp:// URL, got: {resource}"
        )));
    };

    resolve_cdx_url(base_url, rest).map_err(CliError::InvalidCdxUrl)
}

/// Resolve a cdx-cz-spp:// path (everything after "cdx-cz-spp://") into a full HTTP URL.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    // Split path and query string
    let (path, query) = match cdx_path.find('?') {
        Some(idx) => (&cdx_path[..idx], &cdx_path[idx..]),
        None => (cdx_path, ""),
    };

    let encoded_query = encode_query_values(query);

    if let Some(domain) = path.strip_prefix("search/") {
        let api_path = lookup_search_prefix(domain)?;
        Ok(format!("{base}/{api_path}/search{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = lookup_display_id_prefix(id)?;
        Ok(format!(
            "{base}/{api_path}/doc/{id}/{endpoint}{encoded_query}"
        ))
    } else if let Some(rest) = path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        Ok(format!("{base}/resolve/{rest}{encoded_query}"))
    } else {
        Err(format!("Unknown cdx-cz-spp:// path: {path}"))
    }
}

/// Percent-encode non-ASCII bytes in query parameter values.
/// Preserves query structure characters (`?`, `&`, `=`) and ASCII values untouched.
fn encode_query_values(query: &str) -> String {
    let query = match query.strip_prefix('?') {
        Some(q) => q,
        None => return query.to_string(),
    };

    let mut result = String::with_capacity(query.len() + 16);
    result.push('?');

    for (i, pair) in query.split('&').enumerate() {
        if i > 0 {
            result.push('&');
        }
        match pair.split_once('=') {
            Some((key, value)) => {
                result.push_str(key);
                result.push('=');
                percent_encode_utf8(&mut result, value);
            }
            None => result.push_str(pair),
        }
    }

    result
}

/// Percent-encode any non-ASCII bytes in a UTF-8 string.
/// ASCII characters are passed through unchanged.
fn percent_encode_utf8(out: &mut String, input: &str) {
    for byte in input.bytes() {
        if byte.is_ascii() {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(to_hex_digit(byte >> 4));
            out.push(to_hex_digit(byte & 0x0F));
        }
    }
}

fn to_hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + nibble - 10) as char,
        _ => unreachable!(),
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
            "CZSB",
            r#"{"query":"vyhláška","limit":5}"#,
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
                "https://api.example.com/api/CZ/sbirkapp/search",
                "-d",
                r#"{"query":"vyhláška","limit":5}"#,
            ]
        );
    }

    #[test]
    fn search_url_includes_sort_and_order_query_params() {
        let args = build_search_curl_args(
            BASE,
            None,
            "CZSB",
            r#"{"query":"test"}"#,
            Some("date"),
            Some("desc"),
        );

        assert_eq!(
            args[6],
            "https://api.example.com/api/CZ/sbirkapp/search?sort=date&order=desc"
        );
    }

    #[test]
    fn search_url_includes_sort_only() {
        let url = build_search_url(BASE, "CZSB", Some("relevance"), None);
        assert_eq!(
            url,
            "https://api.example.com/api/CZ/sbirkapp/search?sort=relevance"
        );
    }

    #[test]
    fn get_curl_args_translate_cdx_cz_spp_resource_to_api_url() {
        let args =
            build_get_curl_args(BASE, None, "cdx-cz-spp://doc/CZSB1234/meta").unwrap();

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "https://api.example.com/api/CZ/sbirkapp/doc/CZSB1234/meta",
            ]
        );
    }

    #[test]
    fn get_curl_args_reject_non_cdx_cz_spp_urls() {
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
                r#"{"query":"vyhláška","limit":5}"#.to_string(),
            ],
        );

        assert_eq!(
            rendered,
            r#"curl -d '{"query":"vyhláška","limit":5}'"#
        );
    }

    #[test]
    fn search_curl_args_include_auth_header_when_present() {
        let args = build_search_curl_args(
            BASE,
            Some("Authorization: Bearer token"),
            "CZSB",
            r#"{"query":"test"}"#,
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
            "cdx-cz-spp://doc/CZSB1234/meta",
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
                r#"{"query":"test"}"#.to_string(),
            ]),
        );

        assert_eq!(
            rendered,
            "curl -H 'Authorization: Bearer <redacted>' -d '{\"query\":\"test\"}'"
        );
    }

    // --- URL resolution tests ---

    #[test]
    fn search_czsb() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/CZSB"),
            Ok(format!("{BASE}/CZ/sbirkapp/search"))
        );
    }

    #[test]
    fn search_unknown_domain() {
        assert!(resolve_cdx_url(BASE, "search/unknown").is_err());
    }

    #[test]
    fn doc_czsb_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB1234/meta"),
            Ok(format!("{BASE}/CZ/sbirkapp/doc/CZSB1234/meta"))
        );
    }

    #[test]
    fn doc_czsb_text() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB9012/text"),
            Ok(format!("{BASE}/CZ/sbirkapp/doc/CZSB9012/text"))
        );
    }

    #[test]
    fn doc_czsb_related() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB5678/related"),
            Ok(format!(
                "{BASE}/CZ/sbirkapp/doc/CZSB5678/related"
            ))
        );
    }

    #[test]
    fn doc_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB1/text?part=paragraf-1"),
            Ok(format!(
                "{BASE}/CZ/sbirkapp/doc/CZSB1/text?part=paragraf-1"
            ))
        );
    }

    #[test]
    fn doc_with_czech_diacritics_in_query_value() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB1/text?part=odůvodnění"),
            Ok(format!(
                "{BASE}/CZ/sbirkapp/doc/CZSB1/text?part=od%C5%AFvodn%C4%9Bn%C3%AD"
            ))
        );
    }

    #[test]
    fn query_with_multiple_params_and_diacritics() {
        assert_eq!(
            resolve_cdx_url(
                BASE,
                "doc/CZSB1/text?part=paragraf-1&timecutId=účinnost"
            ),
            Ok(format!(
                "{BASE}/CZ/sbirkapp/doc/CZSB1/text?part=paragraf-1&timecutId=%C3%BA%C4%8Dinnost"
            ))
        );
    }

    #[test]
    fn doc_attachment() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB100/attachment/content_1.pdf"),
            Ok(format!(
                "{BASE}/CZ/sbirkapp/doc/CZSB100/attachment/content_1.pdf"
            ))
        );
    }

    #[test]
    fn doc_unknown_prefix() {
        assert!(resolve_cdx_url(BASE, "doc/UNKNOWN123/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZSB1234").is_err());
    }

    #[test]
    fn doc_empty_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZSB1234/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/CZSB1234"),
            Ok(format!("{BASE}/resolve/CZSB1234"))
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
            resolve_cdx_url("https://api.example.com/api/", "resolve/CZSB1"),
            Ok("https://api.example.com/api/resolve/CZSB1".to_string())
        );
    }
}
