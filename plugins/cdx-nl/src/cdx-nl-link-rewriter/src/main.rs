const CDX_SCHEME: &str = "cdx-nl://";

/// Display ID prefix -> API path (same as cdx-nl binary)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("NLBWB", "NL/wetten/bwb"),
    ("NLUIT", "NL/rechtspraak/uitspraken"),
];

use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

fn sign_path(secret: &[u8], path: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .expect("HMAC accepts any key length");
    mac.update(path.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

/// Extract the path component from a full URL (everything from the first `/` after `://`).
fn extract_url_path(url: &str) -> &str {
    match url.find("://") {
        Some(scheme_end) => {
            let after_scheme = &url[scheme_end + 3..];
            match after_scheme.find('/') {
                Some(slash) => &after_scheme[slash..],
                None => "/",
            }
        }
        None => url,
    }
}

/// Split a cdx-nl:// path tail (without scheme) into `(route, query_and_fragment)`.
/// `query_and_fragment` includes the leading `?` or `#`, or is empty.
fn split_query_and_fragment(path: &str) -> (&str, &str) {
    match path.find(|c| c == '?' || c == '#') {
        Some(pos) => (&path[..pos], &path[pos..]),
        None => (path, ""),
    }
}

/// Percent-encode non-ASCII bytes in query parameter values.
/// Preserves query structure characters (`?`, `&`, `=`), the fragment (`#...`),
/// and ASCII values unchanged.
fn encode_query_values(query_and_fragment: &str) -> String {
    // Split off any fragment so it passes through untouched.
    let (q, frag) = match query_and_fragment.find('#') {
        Some(pos) => (&query_and_fragment[..pos], &query_and_fragment[pos..]),
        None => (query_and_fragment, ""),
    };

    let q_stripped = match q.strip_prefix('?') {
        Some(rest) => rest,
        None => {
            // No query string — pass through (still attach any fragment).
            let mut out = q.to_string();
            out.push_str(frag);
            return out;
        }
    };

    let mut result = String::with_capacity(query_and_fragment.len() + 16);
    result.push('?');
    for (i, pair) in q_stripped.split('&').enumerate() {
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
    result.push_str(frag);
    result
}

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

/// Sign `url` if a secret is provided AND the resolved path is an attachment URL.
/// HMAC-SHA256 over the path component, URL-safe base64 (no padding),
/// inserted as `?sig=…` BEFORE any `#fragment`.
fn maybe_sign(url: String, signing_secret: Option<&str>) -> String {
    if let Some(secret) = signing_secret {
        if url.contains("/attachment/") {
            // Split off any fragment so the signature goes before it.
            let (head, frag) = match url.find('#') {
                Some(pos) => (&url[..pos], &url[pos..]),
                None => (url.as_str(), ""),
            };
            let url_path = extract_url_path(head);
            // Sign just the path (no query string) — query params are not part of the signed surface.
            let path_only = match url_path.find('?') {
                Some(pos) => &url_path[..pos],
                None => url_path,
            };
            let sig = sign_path(secret.as_bytes(), path_only);
            let sep = if head.contains('?') { '&' } else { '?' };
            return format!("{head}{sep}sig={sig}{frag}");
        }
    }
    url
}

fn main() {
    let base_url = match std::env::var("CODEXIS_PLUGIN_NL_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CODEXIS_PLUGIN_NL_API_URL must be set (e.g., export CODEXIS_PLUGIN_NL_API_URL=\"https://search.example.com/api\")");
            std::process::exit(2);
        }
    };

    // Signing disabled — secret must not be on the VM (user has root).
    // When re-enabling, move signing to the daemon side (post-processing).
    let signing_secret: Option<String> = None;

    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .expect("failed to read stdin");

    let output = rewrite_cdx_nl_links(&input, &base_url, signing_secret.as_deref());
    print!("{output}");
}

fn rewrite_cdx_nl_links(html: &str, base_url: &str, signing_secret: Option<&str>) -> String {
    let base = base_url.trim_end_matches('/');
    let needle = "href=\"cdx-nl://";
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(href_start) = remaining.find(needle) {
        // Append everything before the match
        result.push_str(&remaining[..href_start]);

        // Find the opening quote position (after href=)
        let url_start = href_start + "href=\"".len();
        let after_needle = &remaining[url_start..];

        // Find the closing quote
        let closing_quote = match after_needle.find('"') {
            Some(pos) => pos,
            None => {
                // Malformed, just pass through
                result.push_str(&remaining[href_start..href_start + needle.len()]);
                remaining = &remaining[href_start + needle.len()..];
                continue;
            }
        };

        let cdx_url = &after_needle[..closing_quote];
        let path = &cdx_url[CDX_SCHEME.len()..]; // everything after cdx-nl://

        match resolve_cdx_nl_path(path, base, signing_secret) {
            Some(resolved) => {
                result.push_str("href=\"");
                result.push_str(&resolved);
                result.push('"');

                // Advance past the closing quote
                let after_closing = &remaining[url_start + closing_quote + 1..];

                // Find the end of the tag to check for existing target=
                let tag_end = after_closing.find('>').unwrap_or(after_closing.len());
                let rest_of_tag = &after_closing[..tag_end];

                if !rest_of_tag.contains(" target=") {
                    result.push_str(" target=\"_blank\"");
                }

                remaining = after_closing;
            }
            None => {
                // Unknown prefix, pass through unchanged
                result.push_str(&remaining[href_start..url_start + closing_quote + 1]);
                remaining = &remaining[url_start + closing_quote + 1..];
            }
        }
    }

    result.push_str(remaining);
    result
}

fn resolve_cdx_nl_path(
    path: &str,
    base: &str,
    signing_secret: Option<&str>,
) -> Option<String> {
    let base = base.trim_end_matches('/');
    let (route, query_and_fragment) = split_query_and_fragment(path);
    let encoded_qf = encode_query_values(query_and_fragment);

    let resolved = if let Some(rest) = route.strip_prefix("doc/") {
        let (id, endpoint) = rest.split_once('/')?;
        if endpoint.is_empty() { return None; }
        let api_path = lookup_display_id_prefix(id)?;
        format!("{base}/{api_path}/doc/{id}/{endpoint}{encoded_qf}")
    } else if let Some(rest) = route.strip_prefix("law/NL/") {
        if rest.is_empty() { return None; }
        match rest.split_once('/') {
            None        => format!("{base}/NL/wetten/bwb/bwbid/{rest}{encoded_qf}"),
            Some((b,e)) => format!("{base}/NL/wetten/bwb/bwbid/{b}/{e}{encoded_qf}"),
        }
    } else if let Some(rest) = route.strip_prefix("afkorting/") {
        if rest.is_empty() { return None; }
        format!("{base}/NL/wetten/bwb/afkorting/{rest}{encoded_qf}")
    } else if let Some(rest) = route.strip_prefix("ecli/") {
        if rest.is_empty() { return None; }
        format!("{base}/NL/rechtspraak/uitspraken/by-ecli/{rest}{encoded_qf}")
    } else if let Some(rest) = route.strip_prefix("publication/") {
        // Only /<pubId>/resolve is reachable via cdx-nl://. The bare
        // /publication/<id> bytes endpoint stays server-side only.
        let pub_id = rest.strip_suffix("/resolve")
            .filter(|p| !p.is_empty() && !p.contains('/'))?;
        format!("{base}/NL/wetten/bwb/publication/{pub_id}/resolve{encoded_qf}")
    } else if let Some(rest) = route.strip_prefix("resolve/") {
        if rest.is_empty() { return None; }
        format!("{base}/resolve/{rest}{encoded_qf}")
    } else {
        return None;
    };

    // Signing pass-through — attach ?sig= only when the resolved path is an
    // attachment AND a signing_secret is supplied.
    Some(maybe_sign(resolved, signing_secret))
}

fn lookup_display_id_prefix(id: &str) -> Option<&'static str> {
    ID_PREFIXES.iter().find(|(k, _)| id.starts_with(k)).map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    #[test]
    fn rewrites_doc_nlbwb_attachment_href() {
        let html = r#"<a href="cdx-nl://doc/NLBWB1/attachment/content_5.pdf#page=3">link</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="https://api.example.com/api/NL/wetten/bwb/doc/NLBWB1/attachment/content_5.pdf#page=3""#));
        assert!(out.contains(r#"target="_blank""#));
    }

    #[test]
    fn rewrites_doc_nluit_attachment_href() {
        let html = r#"<a href="cdx-nl://doc/NLUIT5/attachment/decision.pdf">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="https://api.example.com/api/NL/rechtspraak/uitspraken/doc/NLUIT5/attachment/decision.pdf""#));
    }

    #[test]
    fn rewrites_ecli_attachment_href_preserving_colons() {
        let html = r#"<a href="cdx-nl://ecli/ECLI:NL:HR:2024:1234/attachment/decision.pdf">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains("/by-ecli/ECLI:NL:HR:2024:1234/attachment/decision.pdf"));
    }

    // NB: there are NO /law/NL/<id>/attachment or /afkorting/<id>/attachment routes on
    // the backend — only /doc/{docId}/attachment and /by-ecli/{ecli}/attachment exist
    // (see SearchApiAuthConfig permitAll matchers). SKILL.md restricts the LLM to
    // emit attachment links only on /doc/ or /ecli/, so the rewriter never sees these
    // paths in practice. We don't add positive tests for non-existent routes.

    #[test]
    fn rewrites_publication_resolve_href() {
        let html = r#"<a href="cdx-nl://publication/stb-2024-123/resolve">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains("/NL/wetten/bwb/publication/stb-2024-123/resolve"));
    }

    #[test]
    fn publication_bare_id_in_href_passes_through_unchanged() {
        // The arm validates /resolve suffix; malformed publication URIs must NOT
        // be expanded into the protected bytes endpoint.
        let html = r#"<a href="cdx-nl://publication/stb-2024-123">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="cdx-nl://publication/stb-2024-123""#),
                "bare publication URI must pass through unchanged, got: {out}");
        assert!(!out.contains("/NL/wetten/bwb/publication/stb-2024-123\""),
                "must not expand to backend bytes endpoint");
    }

    #[test]
    fn publication_with_extra_segment_passes_through_unchanged() {
        let html = r#"<a href="cdx-nl://publication/stb-2024-123/bytes">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="cdx-nl://publication/stb-2024-123/bytes""#));
    }

    #[test]
    fn rewrites_resolve_href_to_global_endpoint() {
        let html = r#"<a href="cdx-nl://resolve/NLBWB1234">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="https://api.example.com/api/resolve/NLBWB1234""#));
    }

    #[test]
    fn malformed_cdx_nl_href_passes_through_unchanged() {
        let html = r#"<a href="cdx-nl://garbage/foo/bar">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert!(out.contains(r#"href="cdx-nl://garbage/foo/bar""#));
    }

    #[test]
    fn non_cdx_nl_href_passes_through_unchanged() {
        let html = r#"<a href="https://example.com/foo">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, None);
        assert_eq!(out, html);
    }

    #[test]
    fn signed_attachment_gets_sig_query_param() {
        // When a signing secret is provided, attachment URLs get a ?sig= param.
        let html = r#"<a href="cdx-nl://doc/NLBWB1/attachment/content_5.pdf">x</a>"#;
        let out = rewrite_cdx_nl_links(html, BASE, Some("test-secret"));
        assert!(out.contains("?sig="), "signed link should carry sig= param: {out}");
    }
}
