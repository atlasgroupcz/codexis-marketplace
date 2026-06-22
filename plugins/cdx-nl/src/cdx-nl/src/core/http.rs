use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;
use crate::core::sources::publish_from_body;

const CDX_NL_SCHEME: &str = "cdx-nl://";

/// Display ID prefix -> API path (only Dutch domains)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("NLBWB", "NL/wetten/bwb"),
    ("NLUIT", "NL/rechtspraak/uitspraken"),
    ("NLCVDR", "NL/lokaleregelgeving/cvdr"),
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
        eprintln!("cdx-nl: failed to attach sources to chat: {err}");
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
    let Some(rest) = resource.strip_prefix(CDX_NL_SCHEME) else {
        return Err(CliError::InvalidCdxUrl(format!(
            "get expects a cdx-nl:// URL, got: {resource}"
        )));
    };

    resolve_cdx_url(base_url, rest).map_err(CliError::InvalidCdxUrl)
}

/// Resolve a cdx-nl:// path (everything after "cdx-nl://") into a full HTTP URL.
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
    } else if let Some(rest) = path.strip_prefix("law/NL/") {
        if rest.is_empty() {
            return Err("Missing BWB-id after law/NL/".to_string());
        }
        // rest is either "<BWB_ID>" or "<BWB_ID>/<endpoint>[/<sub>]"
        let (bwbid, endpoint_opt) = match rest.split_once('/') {
            Some((b, e)) => (b, Some(e)),
            None => (rest, None),
        };
        let api_path = "NL/wetten/bwb";
        Ok(match endpoint_opt {
            None => format!("{base}/{api_path}/bwbid/{bwbid}{encoded_query}"),
            Some(ep) => format!("{base}/{api_path}/bwbid/{bwbid}/{ep}{encoded_query}"),
        })
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
    } else if let Some(rest) = path.strip_prefix("afkorting/") {
        if rest.is_empty() {
            return Err("Missing abbreviation after afkorting/".to_string());
        }
        Ok(format!("{base}/NL/wetten/bwb/afkorting/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("ecli/") {
        if rest.is_empty() {
            return Err("Missing ECLI after ecli/".to_string());
        }
        Ok(format!("{base}/NL/rechtspraak/uitspraken/by-ecli/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("workid/") {
        // NLCVDR work-id resolver: cdx-nl://workid/<CVDR_WORK_ID> resolves a source work
        // identifier to its CVDR display document. CVDR-only (NLBWB uses law/NL + afkorting).
        if rest.is_empty() {
            return Err("Missing work-id after workid/".to_string());
        }
        Ok(format!("{base}/NL/lokaleregelgeving/cvdr/workid/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("publication/") {
        // Spec §6: only /<pubId>/resolve is exposed via the cdx-nl:// scheme.
        // The bare /publication/<id> bytes endpoint stays on the server but is
        // intentionally unreachable through this URI namespace.
        let pub_id = rest.strip_suffix("/resolve")
            .filter(|p| !p.is_empty() && !p.contains('/'))
            .ok_or_else(|| format!(
                "Only cdx-nl://publication/<id>/resolve is supported; got publication/{rest}"
            ))?;
        Ok(format!("{base}/NL/wetten/bwb/publication/{pub_id}/resolve{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        // Routed under /NL/ so the reverse proxy (haproxy map at
        // iac/ansible/roles/atlas-bastion-haproxy/files/cdx-daemon-routes.map)
        // forwards it to the ai-scraper backend like every other NL endpoint.
        Ok(format!("{base}/NL/resolve/{rest}{encoded_query}"))
    } else {
        Err(format!("Unknown cdx-nl:// path: {path}"))
    }
}

/// Rewrite ONLY `cdx-nl://…/attachment/…` links in `body` to absolute https URLs,
/// preserving any `#fragment` / `?query`. Non-attachment cdx:// links are left as-is.
/// Pure string scan (no JSON parse, no regex) — the matched token ends at the first
/// JSON/markdown delimiter.
pub(crate) fn resolve_attachment_links(body: &str, base_url: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(pos) = rest.find(CDX_NL_SCHEME) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + CDX_NL_SCHEME.len()..];
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
        let token = &after[..end]; // e.g. doc/NLBWB12/attachment/content_1.pdf#page=3
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
                    out.push_str(CDX_NL_SCHEME);
                    out.push_str(token);
                }
            }
        } else {
            out.push_str(CDX_NL_SCHEME);
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

fn lookup_search_prefix(source: &str) -> Result<&'static str, String> {
    match source {
        "NLBWB" => Ok("NL/wetten/bwb"),
        "NLUIT" => Ok("NL/rechtspraak/uitspraken"),
        "NLCVDR" => Ok("NL/lokaleregelgeving/cvdr"),
        other => Err(format!("Unknown source code: {other}")),
    }
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
    fn doc_nlbwb_meta_routes_to_wetten_bwb() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/meta").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/meta")
        );
    }

    #[test]
    fn doc_nluit_meta_routes_to_rechtspraak_uitspraken() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLUIT1/meta").unwrap(),
            format!("{BASE}/NL/rechtspraak/uitspraken/doc/NLUIT1/meta")
        );
    }

    #[test]
    fn doc_nlcvdr_meta_routes_to_lokaleregelgeving_cvdr() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLCVDR1/meta").unwrap(),
            format!("{BASE}/NL/lokaleregelgeving/cvdr/doc/NLCVDR1/meta")
        );
    }

    #[test]
    fn doc_nlcvdr_at_with_query_preserved() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLCVDR1/at?date=2024-01-01").unwrap(),
            format!("{BASE}/NL/lokaleregelgeving/cvdr/doc/NLCVDR1/at?date=2024-01-01")
        );
    }

    #[test]
    fn workid_routes_to_lokaleregelgeving_cvdr() {
        assert_eq!(
            resolve_cdx_url(BASE, "workid/CVDR72510").unwrap(),
            format!("{BASE}/NL/lokaleregelgeving/cvdr/workid/CVDR72510")
        );
        assert!(resolve_cdx_url(BASE, "workid/").is_err());
    }

    #[test]
    fn search_url_for_nlcvdr() {
        let args = build_search_curl_args(BASE, None, "NLCVDR", r#"{"query":"apv"}"#, None, None);
        assert!(args.contains(&format!("{BASE}/NL/lokaleregelgeving/cvdr/search")));
    }

    #[test]
    fn doc_related_with_query_string_preserved() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/related?direction=in&limit=10").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/related?direction=in&limit=10")
        );
    }

    #[test]
    fn doc_related_counts_suffix_routed() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/related/counts").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/related/counts")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLUIT1/related/counts").unwrap(),
            format!("{BASE}/NL/rechtspraak/uitspraken/doc/NLUIT1/related/counts")
        );
    }

    #[test]
    fn doc_citations_preserves_all_query_params() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/citations?toestandId=X&page=3").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/citations?toestandId=X&page=3")
        );
    }

    #[test]
    fn doc_at_with_date_query_routed() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/at?date=2020-06-15").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/at?date=2020-06-15")
        );
    }

    #[test]
    fn doc_cited_by_decisions_routed_with_paging() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/cited-by-decisions").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/cited-by-decisions")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "doc/NLBWB1/cited-by-decisions?limit=20&offset=10").unwrap(),
            format!("{BASE}/NL/wetten/bwb/doc/NLBWB1/cited-by-decisions?limit=20&offset=10")
        );
    }

    #[test]
    fn law_nl_at_and_cited_by_decisions_routed() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/NL/BWBR0001827/at?date=2020-06-15").unwrap(),
            format!("{BASE}/NL/wetten/bwb/bwbid/BWBR0001827/at?date=2020-06-15")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "law/NL/BWBR0001827/cited-by-decisions").unwrap(),
            format!("{BASE}/NL/wetten/bwb/bwbid/BWBR0001827/cited-by-decisions")
        );
    }

    #[test]
    fn afkorting_at_and_cited_by_decisions_routed() {
        assert_eq!(
            resolve_cdx_url(BASE, "afkorting/BW/at?date=2020-06-15").unwrap(),
            format!("{BASE}/NL/wetten/bwb/afkorting/BW/at?date=2020-06-15")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "afkorting/BW/cited-by-decisions").unwrap(),
            format!("{BASE}/NL/wetten/bwb/afkorting/BW/cited-by-decisions")
        );
    }

    #[test]
    fn versions_with_paging_and_window_preserved() {
        assert_eq!(
            resolve_cdx_url(
                BASE,
                "doc/NLBWB1/versions?limit=50&offset=0&from=2020-01-01&to=2024-12-31",
            )
            .unwrap(),
            format!(
                "{BASE}/NL/wetten/bwb/doc/NLBWB1/versions?limit=50&offset=0&from=2020-01-01&to=2024-12-31"
            )
        );
        assert_eq!(
            resolve_cdx_url(BASE, "afkorting/BW/versions?includeAll=true").unwrap(),
            format!("{BASE}/NL/wetten/bwb/afkorting/BW/versions?includeAll=true")
        );
    }

    #[test]
    fn law_nl_bwbid_routes_to_bwbid_endpoint() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/NL/BWBR0001827").unwrap(),
            format!("{BASE}/NL/wetten/bwb/bwbid/BWBR0001827")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "law/NL/BWBR0001827/text?toestandId=X").unwrap(),
            format!("{BASE}/NL/wetten/bwb/bwbid/BWBR0001827/text?toestandId=X")
        );
    }

    #[test]
    fn afkorting_routes_to_wetten_bwb() {
        assert_eq!(
            resolve_cdx_url(BASE, "afkorting/BW/text").unwrap(),
            format!("{BASE}/NL/wetten/bwb/afkorting/BW/text")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "afkorting/BW/parts").unwrap(),
            format!("{BASE}/NL/wetten/bwb/afkorting/BW/parts")
        );
    }

    #[test]
    fn ecli_routes_to_rechtspraak_uitspraken_preserving_colons() {
        assert_eq!(
            resolve_cdx_url(BASE, "ecli/ECLI:NL:HR:2024:1234").unwrap(),
            format!("{BASE}/NL/rechtspraak/uitspraken/by-ecli/ECLI:NL:HR:2024:1234")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "ecli/ECLI:NL:HR:2024:1234/text?page=2").unwrap(),
            format!("{BASE}/NL/rechtspraak/uitspraken/by-ecli/ECLI:NL:HR:2024:1234/text?page=2")
        );
    }

    #[test]
    fn publication_resolve_routes_to_wetten_bwb_publication() {
        assert_eq!(
            resolve_cdx_url(BASE, "publication/stb-2024-123/resolve").unwrap(),
            format!("{BASE}/NL/wetten/bwb/publication/stb-2024-123/resolve")
        );
    }

    #[test]
    fn publication_bare_id_rejected() {
        assert!(resolve_cdx_url(BASE, "publication/stb-2024-123").is_err());
    }

    #[test]
    fn publication_with_extra_path_rejected() {
        assert!(resolve_cdx_url(BASE, "publication/stb-2024-123/bytes").is_err());
    }

    #[test]
    fn resolve_global_endpoint_for_both_sources() {
        // Routed under /NL/ to match the haproxy /api/NL/ prefix filter.
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/NLBWB1234").unwrap(),
            format!("{BASE}/NL/resolve/NLBWB1234")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/NLUIT5678").unwrap(),
            format!("{BASE}/NL/resolve/NLUIT5678")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/BWBR0001827").unwrap(),
            format!("{BASE}/NL/resolve/BWBR0001827")
        );
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/ECLI:NL:HR:2024:1234").unwrap(),
            format!("{BASE}/NL/resolve/ECLI:NL:HR:2024:1234")
        );
    }

    #[test]
    fn unknown_route_returns_error() {
        assert!(resolve_cdx_url(BASE, "foo/NLBWB1/bar").is_err());
    }

    #[test]
    fn missing_prefix_returns_error() {
        assert!(resolve_cdx_url(BASE, "").is_err());
        assert!(resolve_cdx_url(BASE, "doc/").is_err());
        assert!(resolve_cdx_url(BASE, "ecli/").is_err());
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "doc/NLBWB1/meta").unwrap(),
            "https://api.example.com/api/NL/wetten/bwb/doc/NLBWB1/meta"
        );
    }

    #[test]
    fn search_url_for_nlbwb() {
        let args = build_search_curl_args(BASE, None, "NLBWB", r#"{"query":"BW"}"#, None, None);
        assert!(args.contains(&format!("{BASE}/NL/wetten/bwb/search")));
    }

    #[test]
    fn search_url_for_nluit() {
        let args = build_search_curl_args(BASE, None, "NLUIT", r#"{"query":"Hoge Raad"}"#, None, None);
        assert!(args.contains(&format!("{BASE}/NL/rechtspraak/uitspraken/search")));
    }

    // --- attachment link resolution + envelope splitting ---

    #[test]
    fn resolves_only_attachment_links_keeping_fragment() {
        let base = "https://example.com/api";
        let body = r#"{"a":"cdx-nl://doc/NLBWB12/attachment/content_1.pdf#page=3","b":"cdx-nl://doc/NLBWB12/meta","c":"plain"}"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains(
            "https://example.com/api/NL/wetten/bwb/doc/NLBWB12/attachment/content_1.pdf#page=3"
        ));
        assert!(
            out.contains("\"cdx-nl://doc/NLBWB12/meta\""),
            "non-attachment cdx:// must be left untouched: {out}"
        );
        assert!(out.contains("\"plain\""));
    }

    #[test]
    fn split_envelope_extracts_content_and_sources() {
        let body = r#"{"content":{"results":[{"docId":"NLBWB1"}]},"sources":[{"__kind":"url","url":"https://x/y.pdf"}]}"#;
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
        let body = r#""cdx-nl://doc/NLBWB12/attachment/content_1.pdf?v=2#page=7""#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("https://example.com/api/NL/wetten/bwb/doc/NLBWB12/attachment/content_1.pdf")
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
            r#"{"content":1,"sources":[{"__kind":"url","url":"cdx-nl://doc/NLBWB12/meta"}]}"#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("\"cdx-nl://doc/NLBWB12/meta\""),
            "non-attachment cdx:// in sources must stay unresolved: {out}"
        );
    }

    #[test]
    fn resolves_attachment_amid_multibyte_dutch_text() {
        let base = "https://example.com/api";
        // Multi-byte chars (…, é, ë) IMMEDIATELY adjacent to the token guard the char-boundary
        // slicing — this resolver runs on NL content where a mis-sliced index panics.
        let body = "Besluit…cdx-nl://doc/NLBWB12/attachment/content_1.pdf#page=3,méér ë";
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("https://example.com/api/NL/wetten/bwb/doc/NLBWB12/attachment/content_1.pdf#page=3"));
        assert!(out.starts_with("Besluit…https://"), "multi-byte text immediately before token preserved: {out}");
        assert!(out.ends_with(",méér ë"), "multi-byte text after the delimiter preserved: {out}");
    }

    #[test]
    fn resolves_multiple_attachment_links_in_one_body() {
        let base = "https://example.com/api";
        let body = r#"["cdx-nl://doc/NLBWB1/attachment/a.pdf","cdx-nl://doc/NLBWB2/attachment/b.pdf#page=2"]"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("https://example.com/api/NL/wetten/bwb/doc/NLBWB1/attachment/a.pdf"));
        assert!(out.contains("https://example.com/api/NL/wetten/bwb/doc/NLBWB2/attachment/b.pdf#page=2"));
        assert!(!out.contains("cdx-nl://"), "all attachment links resolved: {out}");
    }

    #[test]
    fn unknown_prefix_attachment_falls_back_to_cdx() {
        let base = "https://example.com/api";
        // Unknown ID prefix → resolve_cdx_url errors → original cdx:// link re-emitted unchanged.
        let body = r#""cdx-nl://doc/XXYY9/attachment/c.pdf#page=1""#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("\"cdx-nl://doc/XXYY9/attachment/c.pdf#page=1\""),
            "unknown-prefix attachment link must be left as cdx://: {out}");
    }
}

