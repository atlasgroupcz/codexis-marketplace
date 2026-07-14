
use crate::core::error::CliError;
use cdx_common::curl::encode_query_values;

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

pub(crate) struct CdxSkApi;

impl cdx_common::search_engine::SearchApi for CdxSkApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        CDX_SK_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "cdx-sk"
    }
    fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String> {
        resolve_cdx_url(base_url, cdx_path)
    }
    fn resolve_attachment_links(&self, body: &str, base_url: &str) -> String {
        resolve_attachment_links(body, base_url)
    }
}

pub(crate) fn execute_search_request(
    base_url: &str,
    auth_header: Option<&str>,
    source_code: &str,
    payload: &str,
    dry_run: bool,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<(), CliError> {
    cdx_common::search_engine::execute_search_request(
        &CdxSkApi, base_url, auth_header, source_code, payload, dry_run, sort, order,
    )
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    cdx_common::search_engine::execute_get_request(&CdxSkApi, base_url, auth_header, resource, dry_run)
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

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

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
