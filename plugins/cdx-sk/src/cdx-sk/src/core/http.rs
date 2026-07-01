use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;
use crate::core::sources::publish_from_body;

const CDX_SK_SCHEME: &str = "cdx-sk://";

/// Display ID prefix -> API path (only Slovak domains).
///
/// Order matters: `lookup_display_id_prefix` matches by `starts_with` in declaration order, and
/// "SKNS" is a strict prefix of "SKNSS" — so SKNSS MUST appear before SKNS, else `SKNSS123` would
/// mis-route to najvyssi-sud. Longer prefixes are listed first to preserve this invariant.
const ID_PREFIXES: &[(&str, &str)] = &[
    ("SKNUS", "SK/najvyssi-ustavny-sud"),
    ("SKNSS", "SK/najvyssi-spravny-sud"),
    ("SKVS", "SK/vseobecne-sudy"),
    ("SKUS", "SK/ustavny-sud"),
    ("SKEZ", "SK/ezbierka"),
    ("SKNS", "SK/najvyssi-sud"),
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
    handle_curl_output(&output, |body| handle_api_response(body, base_url))
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
    handle_curl_output(&output, |body| handle_api_response(body, base_url))
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

/// Resolve attachment links, unwrap the envelope, print content, best-effort publish sources.
fn handle_api_response(body: &[u8], base_url: &str) -> Result<(), CliError> {
    let raw = String::from_utf8_lossy(body);
    let resolved = resolve_attachment_links(&raw, base_url);
    let (content, sources) = split_envelope_str(&resolved);
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
        eprintln!("cdx-sk: failed to attach sources to chat: {err}");
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

/// Rewrite ONLY `cdx-sk://…/attachment/…` links in `body` to absolute https URLs,
/// preserving any `#fragment` / `?query`. Non-attachment cdx:// links are left as-is.
/// Pure string scan (no JSON parse, no regex) — the matched token ends at the first
/// JSON/markdown delimiter.
pub(crate) fn resolve_attachment_links(body: &str, base_url: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(pos) = rest.find(CDX_SK_SCHEME) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + CDX_SK_SCHEME.len()..];
        let end = after
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '"' | '\''
                            | '<'
                            | '>'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '\\'
                            | ','
                            | '`'
                    )
            })
            .unwrap_or(after.len());
        let token = &after[..end]; // e.g. doc/SKVS100/attachment/content_1.pdf#page=3
        if token.contains("/attachment/") {
            // Split off the #fragment so it is preserved verbatim (never query-encoded by
            // resolve_cdx_url, which only splits on '?'). Handles fragment-only AND query+fragment.
            let (route, frag) = match token.find('#') {
                Some(h) => (&token[..h], &token[h..]),
                None => (token, ""),
            };
            match resolve_cdx_url(base_url, route) {
                Ok(url) => {
                    out.push_str(&url);
                    out.push_str(frag);
                }
                Err(_) => {
                    out.push_str(CDX_SK_SCHEME);
                    out.push_str(token);
                }
            }
        } else {
            out.push_str(CDX_SK_SCHEME);
            out.push_str(token);
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Split the `{content, sources}` envelope. Returns `(content_to_print, sources)`.
/// Falls back to `(whole_body, None)` for non-enveloped responses (meta/text/toc).
/// String `content` (e.g. /doc/{id}/text) is unwrapped from its JSON quotes.
pub(crate) fn split_envelope_str(body: &str) -> (String, Option<serde_json::Value>) {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Object(mut map)) if map.contains_key("content") => {
            let sources = map.remove("sources");
            let content = map.remove("content").unwrap_or(serde_json::Value::Null);
            let printed = match &content {
                serde_json::Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            };
            (printed, sources)
        }
        _ => (body.to_string(), None),
    }
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
    let Some(rest) = resource.strip_prefix(CDX_SK_SCHEME) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a cdx-sk:// URL, got: {resource}"
        )));
    };

    resolve_cdx_url(base_url, rest).map_err(CliError::InvalidCdxUrl)
}

/// Resolve a cdx-sk:// path (everything after "cdx-sk://") into a full HTTP URL.
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
    } else if let Some(rest) = path.strip_prefix("law/SK/") {
        if rest.is_empty() {
            return Err("Missing law number/year after law/SK/".to_string());
        }
        Ok(format!("{base}/SK/ezbierka/law/{rest}{encoded_query}"))
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
        Err(format!("Unknown cdx-sk:// path: {path}"))
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
            "SKEZ",
            r#"{"query":"občiansky zákonník","limit":5}"#,
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
                "https://api.example.com/api/SK/ezbierka/search",
                "-d",
                r#"{"query":"občiansky zákonník","limit":5}"#,
            ]
        );
    }

    #[test]
    fn search_url_includes_sort_and_order_query_params() {
        let args = build_search_curl_args(
            BASE,
            None,
            "SKVS",
            r#"{"query":"test"}"#,
            Some("date"),
            Some("desc"),
        );

        assert_eq!(
            args[6],
            "https://api.example.com/api/SK/vseobecne-sudy/search?sort=date&order=desc"
        );
    }

    #[test]
    fn search_url_includes_sort_only() {
        let url = build_search_url(BASE, "SKEZ", Some("relevance"), None);
        assert_eq!(
            url,
            "https://api.example.com/api/SK/ezbierka/search?sort=relevance"
        );
    }

    #[test]
    fn get_curl_args_translate_cdx_sk_resource_to_api_url() {
        let args =
            build_get_curl_args(BASE, None, "cdx-sk://doc/SKEZ1234/meta").unwrap();

        assert_eq!(
            args,
            vec![
                "-sS",
                "--fail-with-body",
                "https://api.example.com/api/SK/ezbierka/doc/SKEZ1234/meta",
            ]
        );
    }

    #[test]
    fn get_curl_args_reject_non_cdx_sk_urls() {
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
                r#"{"query":"občiansky zákonník","limit":5}"#.to_string(),
            ],
        );

        assert_eq!(
            rendered,
            r#"curl -d '{"query":"občiansky zákonník","limit":5}'"#
        );
    }

    #[test]
    fn search_curl_args_include_auth_header_when_present() {
        let args = build_search_curl_args(
            BASE,
            Some("Authorization: Bearer token"),
            "SKEZ",
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
            "cdx-sk://doc/SKEZ1234/meta",
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

    // --- URL resolution tests (ported from old main.rs) ---

    #[test]
    fn search_skez() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKEZ"),
            Ok(format!("{BASE}/SK/ezbierka/search"))
        );
    }

    #[test]
    fn search_skvs() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKVS"),
            Ok(format!("{BASE}/SK/vseobecne-sudy/search"))
        );
    }

    #[test]
    fn search_sknus() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKNUS"),
            Ok(format!("{BASE}/SK/najvyssi-ustavny-sud/search"))
        );
    }

    #[test]
    fn search_sknss() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKNSS"),
            Ok(format!("{BASE}/SK/najvyssi-spravny-sud/search"))
        );
    }

    #[test]
    fn search_skus() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKUS"),
            Ok(format!("{BASE}/SK/ustavny-sud/search"))
        );
    }

    #[test]
    fn search_skns() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/SKNS"),
            Ok(format!("{BASE}/SK/najvyssi-sud/search"))
        );
    }

    #[test]
    fn doc_skus_text() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKUS9012/text"),
            Ok(format!("{BASE}/SK/ustavny-sud/doc/SKUS9012/text"))
        );
    }

    #[test]
    fn doc_sknss_and_skns_prefixes_disambiguate() {
        // "SKNS" is a strict prefix of "SKNSS"; longest-prefix ordering must route each correctly.
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKNSS123/meta"),
            Ok(format!("{BASE}/SK/najvyssi-spravny-sud/doc/SKNSS123/meta"))
        );
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKNS123/meta"),
            Ok(format!("{BASE}/SK/najvyssi-sud/doc/SKNS123/meta"))
        );
    }

    #[test]
    fn search_unknown_domain() {
        assert!(resolve_cdx_url(BASE, "search/unknown").is_err());
    }

    #[test]
    fn law_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/SK/40/1964"),
            Ok(format!("{BASE}/SK/ezbierka/law/40/1964"))
        );
    }

    #[test]
    fn law_with_sub_endpoint() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/SK/40/1964/meta"),
            Ok(format!("{BASE}/SK/ezbierka/law/40/1964/meta"))
        );
    }

    #[test]
    fn law_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/SK/40/1964/text?timecutId=abc"),
            Ok(format!(
                "{BASE}/SK/ezbierka/law/40/1964/text?timecutId=abc"
            ))
        );
    }

    #[test]
    fn law_empty_rest() {
        assert!(resolve_cdx_url(BASE, "law/SK/").is_err());
    }

    #[test]
    fn doc_skez_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKEZ1234/meta"),
            Ok(format!("{BASE}/SK/ezbierka/doc/SKEZ1234/meta"))
        );
    }

    #[test]
    fn doc_skvs_text() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKVS9012/text"),
            Ok(format!("{BASE}/SK/vseobecne-sudy/doc/SKVS9012/text"))
        );
    }

    #[test]
    fn doc_sknus_related() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKNUS5678/related"),
            Ok(format!(
                "{BASE}/SK/najvyssi-ustavny-sud/doc/SKNUS5678/related"
            ))
        );
    }

    #[test]
    fn doc_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKEZ1/text?part=paragraf-1"),
            Ok(format!(
                "{BASE}/SK/ezbierka/doc/SKEZ1/text?part=paragraf-1"
            ))
        );
    }

    #[test]
    fn doc_with_slovak_diacritics_in_query_value() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKVS1/text?part=odôvodnenie"),
            Ok(format!(
                "{BASE}/SK/vseobecne-sudy/doc/SKVS1/text?part=od%C3%B4vodnenie"
            ))
        );
    }

    #[test]
    fn query_with_multiple_params_and_diacritics() {
        assert_eq!(
            resolve_cdx_url(
                BASE,
                "doc/SKEZ1/text?part=paragraf-1&timecutId=účinnosť"
            ),
            Ok(format!(
                "{BASE}/SK/ezbierka/doc/SKEZ1/text?part=paragraf-1&timecutId=%C3%BA%C4%8Dinnos%C5%A5"
            ))
        );
    }

    #[test]
    fn law_with_diacritics_in_query() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/SK/40/1964/text?part=časť-1"),
            Ok(format!(
                "{BASE}/SK/ezbierka/law/40/1964/text?part=%C4%8Das%C5%A5-1"
            ))
        );
    }

    #[test]
    fn doc_attachment() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/SKVS100/attachment/content_1.pdf"),
            Ok(format!(
                "{BASE}/SK/vseobecne-sudy/doc/SKVS100/attachment/content_1.pdf"
            ))
        );
    }

    #[test]
    fn doc_unknown_prefix() {
        assert!(resolve_cdx_url(BASE, "doc/UNKNOWN123/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/SKEZ1234").is_err());
    }

    #[test]
    fn doc_empty_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/SKEZ1234/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/SKEZ1234"),
            Ok(format!("{BASE}/resolve/SKEZ1234"))
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
            resolve_cdx_url("https://api.example.com/api/", "resolve/SKEZ1"),
            Ok("https://api.example.com/api/resolve/SKEZ1".to_string())
        );
    }

    // --- attachment link resolution + envelope splitting ---

    #[test]
    fn resolves_only_attachment_links_keeping_fragment() {
        let base = "https://example.com/api";
        let body = r#"{"a":"cdx-sk://doc/SKVS100/attachment/content_1.pdf#page=3","b":"cdx-sk://doc/SKVS100/meta","c":"plain"}"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains(
            "https://example.com/api/SK/vseobecne-sudy/doc/SKVS100/attachment/content_1.pdf#page=3"
        ));
        assert!(
            out.contains("\"cdx-sk://doc/SKVS100/meta\""),
            "non-attachment cdx:// must be left untouched: {out}"
        );
        assert!(out.contains("\"plain\""));
    }

    #[test]
    fn split_envelope_extracts_content_and_sources() {
        let body = r#"{"content":{"results":[{"docId":"SKVS1"}]},"sources":[{"__kind":"url","url":"https://x/y.pdf"}]}"#;
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

    #[test]
    fn resolves_attachment_with_query_and_fragment() {
        let base = "https://example.com/api";
        let body = r#""cdx-sk://doc/SKVS100/attachment/content_1.pdf?v=2#page=7""#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("https://example.com/api/SK/vseobecne-sudy/doc/SKVS100/attachment/content_1.pdf")
        );
        assert!(
            out.ends_with("#page=7\""),
            "fragment must be preserved verbatim: {out}"
        );
    }

    #[test]
    fn non_attachment_url_in_sources_stays_cdx() {
        let base = "https://example.com/api";
        let body =
            r#"{"content":1,"sources":[{"__kind":"url","url":"cdx-sk://doc/SKVS100/meta"}]}"#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("\"cdx-sk://doc/SKVS100/meta\""),
            "non-attachment cdx:// in sources must stay unresolved: {out}"
        );
    }

    #[test]
    fn resolves_attachment_amid_multibyte_slovak_text() {
        let base = "https://example.com/api";
        // Multi-byte chars (…, š, í) IMMEDIATELY adjacent to the token guard the char-boundary
        // slicing — this resolver runs on Slovak content where a mis-sliced index panics.
        let body = "Rozhodnutie…cdx-sk://doc/SKVS100/attachment/content_1.pdf#page=3,viac š";
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("https://example.com/api/SK/vseobecne-sudy/doc/SKVS100/attachment/content_1.pdf#page=3"));
        assert!(out.starts_with("Rozhodnutie…https://"), "multi-byte text immediately before token preserved: {out}");
        assert!(out.ends_with(",viac š"), "multi-byte text after the delimiter preserved: {out}");
    }

    #[test]
    fn resolves_multiple_attachment_links_in_one_body() {
        let base = "https://example.com/api";
        let body = r#"["cdx-sk://doc/SKVS1/attachment/a.pdf","cdx-sk://doc/SKVS2/attachment/b.pdf#page=2"]"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("https://example.com/api/SK/vseobecne-sudy/doc/SKVS1/attachment/a.pdf"));
        assert!(out.contains("https://example.com/api/SK/vseobecne-sudy/doc/SKVS2/attachment/b.pdf#page=2"));
        assert!(!out.contains("cdx-sk://"), "all attachment links resolved: {out}");
    }

    #[test]
    fn unknown_prefix_attachment_falls_back_to_cdx() {
        let base = "https://example.com/api";
        // Unknown ID prefix → resolve_cdx_url errors → original cdx:// link re-emitted unchanged.
        let body = r#""cdx-sk://doc/XXYY9/attachment/c.pdf#page=1""#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("\"cdx-sk://doc/XXYY9/attachment/c.pdf#page=1\""),
            "unknown-prefix attachment link must be left as cdx://: {out}");
    }
}
