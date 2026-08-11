use crate::core::error::CliError;
use cdx_common::curl::encode_query_values;

const CDX_AT_SCHEME: &str = "cdx-at://";

/// Display ID prefix -> API path (Austrian RIS domains)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("ATBR", "AT/bundesrecht"),
    ("ATJD", "AT/judikatur"),
    ("ATLR", "AT/landesrecht"),
    ("ATSO", "AT/sonstige"),
    ("ATHI", "AT/history"),
];

pub(crate) struct CdxAtApi;

impl cdx_common::search_engine::SearchApi for CdxAtApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        CDX_AT_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "cdx-at"
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
        &CdxAtApi,
        cdx_common::search_engine::SearchRequest {
            base_url,
            auth_header,
            source_code,
            payload,
            dry_run,
            sort,
            order,
        },
    )
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    cdx_common::search_engine::execute_get_request(
        &CdxAtApi,
        base_url,
        auth_header,
        resource,
        dry_run,
    )
}

/// Resolve a cdx-at:// path (everything after "cdx-at://") into a full HTTP URL.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    // Split path and query string
    let (path, query) = match cdx_path.find('?') {
        Some(idx) => (&cdx_path[..idx], &cdx_path[idx..]),
        None => (cdx_path, ""),
    };

    let encoded_query = encode_query_values(query);

    if let Some(domain) = path.strip_prefix("search/") {
        let api_path = cdx_common::http::lookup_search_prefix(domain, ID_PREFIXES)?;
        Ok(format!("{base}/{api_path}/search{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("law/") {
        // Consolidated-law / point-in-time surface lives on the History domain.
        // lawKey is a single token (all-digits Gesetzesnummer or eli~<stem>);
        // the rest (/at, /versions, /toc, /parts, /paragraph/<para>/versions, /text)
        // is forwarded verbatim.
        if rest.is_empty() {
            return Err("Missing law key after law/".to_string());
        }
        Ok(format!("{base}/AT/history/law/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("bgbl/") {
        // BGBl gazette resolver (Bundesrecht only): bgbl/<TYPE>/<YEAR>/<NR>
        if rest.is_empty() {
            return Err("Missing BGBl path after bgbl/".to_string());
        }
        Ok(format!("{base}/AT/bundesrecht/bgbl/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("lgbl/") {
        // LGBl gazette resolver (Landesrecht only): lgbl/<STATE>/<YEAR>/<NR>
        if rest.is_empty() {
            return Err("Missing LGBl path after lgbl/".to_string());
        }
        Ok(format!("{base}/AT/landesrecht/lgbl/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("by-ecli/") {
        // ECLI resolver (Judikatur only): by-ecli/<ECLI>
        if rest.is_empty() {
            return Err("Missing ECLI after by-ecli/".to_string());
        }
        Ok(format!("{base}/AT/judikatur/by-ecli/{rest}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("by-document-number/") {
        // Document-number resolver: by-document-number/<DOMAIN>/<DN>.
        // A NOR number is not unique across domains, so the domain is required.
        let (domain, dn) = rest.split_once('/').ok_or_else(|| {
            format!("Expected by-document-number/<DOMAIN>/<DN>, got: by-document-number/{rest}")
        })?;
        let api_path = lookup_domain_name(domain)?;
        if dn.is_empty() {
            return Err("Missing document number".to_string());
        }
        Ok(format!(
            "{base}/{api_path}/by-document-number/{dn}{encoded_query}"
        ))
    } else if let Some(rest) = path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = cdx_common::http::lookup_display_id_prefix(id, ID_PREFIXES)?;
        Ok(format!(
            "{base}/{api_path}/doc/{id}/{endpoint}{encoded_query}"
        ))
    } else {
        Err(format!("Unknown cdx-at:// path: {path}"))
    }
}

/// Rewrite ONLY `cdx-at://…/attachment/…` links in `body` to absolute https URLs,
/// preserving any `#fragment` / `?query`. Non-attachment cdx:// links are left as-is.
/// Pure string scan (no JSON parse, no regex) - the matched token ends at the first
/// JSON/markdown delimiter.
pub(crate) fn resolve_attachment_links(body: &str, base_url: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(pos) = rest.find(CDX_AT_SCHEME) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + CDX_AT_SCHEME.len()..];
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
        let token = &after[..end]; // e.g. doc/ATBR12/attachment/content_1.pdf#page=3
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
                    out.push_str(CDX_AT_SCHEME);
                    out.push_str(token);
                }
            }
        } else {
            out.push_str(CDX_AT_SCHEME);
            out.push_str(token);
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Domain URL segment -> API path, for the by-document-number resolver.
/// Only the domains that expose a by-document-number endpoint are listed
/// (Sonstige has none).
const DOMAIN_NAMES: &[(&str, &str)] = &[
    ("history", "AT/history"),
    ("judikatur", "AT/judikatur"),
    ("bundesrecht", "AT/bundesrecht"),
    ("landesrecht", "AT/landesrecht"),
];

fn lookup_domain_name(name: &str) -> Result<&'static str, String> {
    for &(domain, api_path) in DOMAIN_NAMES {
        if domain == name {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown domain for by-document-number: {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    // --- URL resolution tests ---

    #[test]
    fn search_atbr() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATBR"),
            Ok(format!("{BASE}/AT/bundesrecht/search"))
        );
    }

    #[test]
    fn search_atjd() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATJD"),
            Ok(format!("{BASE}/AT/judikatur/search"))
        );
    }

    #[test]
    fn search_atlr() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATLR"),
            Ok(format!("{BASE}/AT/landesrecht/search"))
        );
    }

    #[test]
    fn search_atso() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATSO"),
            Ok(format!("{BASE}/AT/sonstige/search"))
        );
    }

    #[test]
    fn search_athi() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATHI"),
            Ok(format!("{BASE}/AT/history/search"))
        );
    }

    #[test]
    fn search_unknown_domain() {
        assert!(resolve_cdx_url(BASE, "search/UNKNOWN").is_err());
    }

    #[test]
    fn doc_atbr_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATBR1234/meta"),
            Ok(format!("{BASE}/AT/bundesrecht/doc/ATBR1234/meta"))
        );
    }

    #[test]
    fn doc_atjd_text() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATJD5678/text"),
            Ok(format!("{BASE}/AT/judikatur/doc/ATJD5678/text"))
        );
    }

    #[test]
    fn doc_athi_attachment() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATHI100/attachment/content_1.pdf"),
            Ok(format!(
                "{BASE}/AT/history/doc/ATHI100/attachment/content_1.pdf"
            ))
        );
    }

    #[test]
    fn doc_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATBR1/text?part=paragraf-1"),
            Ok(format!(
                "{BASE}/AT/bundesrecht/doc/ATBR1/text?part=paragraf-1"
            ))
        );
    }

    #[test]
    fn doc_with_german_diacritics_in_query() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATJD1/text?part=begr\u{00FC}ndung"),
            Ok(format!(
                "{BASE}/AT/judikatur/doc/ATJD1/text?part=begr%C3%BCndung"
            ))
        );
    }

    #[test]
    fn doc_unknown_prefix() {
        assert!(resolve_cdx_url(BASE, "doc/UNKNOWN123/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/ATBR1234").is_err());
    }

    #[test]
    fn doc_empty_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/ATBR1234/").is_err());
    }

    #[test]
    fn law_summary() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/10008147"),
            Ok(format!("{BASE}/AT/history/law/10008147"))
        );
    }

    #[test]
    fn law_point_in_time_with_query() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/10008147/at?date=2024-06-01"),
            Ok(format!("{BASE}/AT/history/law/10008147/at?date=2024-06-01"))
        );
    }

    #[test]
    fn law_eli_token_toc_all() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/eli~jgs~1811~946/toc?all=true"),
            Ok(format!(
                "{BASE}/AT/history/law/eli~jgs~1811~946/toc?all=true"
            ))
        );
    }

    #[test]
    fn law_paragraph_versions() {
        assert_eq!(
            resolve_cdx_url(BASE, "law/10008147/paragraph/Art.%207/versions"),
            Ok(format!(
                "{BASE}/AT/history/law/10008147/paragraph/Art.%207/versions"
            ))
        );
    }

    #[test]
    fn law_empty_key() {
        assert!(resolve_cdx_url(BASE, "law/").is_err());
    }

    #[test]
    fn resolver_bgbl() {
        assert_eq!(
            resolve_cdx_url(BASE, "bgbl/I/2026/62"),
            Ok(format!("{BASE}/AT/bundesrecht/bgbl/I/2026/62"))
        );
    }

    #[test]
    fn resolver_lgbl() {
        assert_eq!(
            resolve_cdx_url(BASE, "lgbl/WI/2026/14"),
            Ok(format!("{BASE}/AT/landesrecht/lgbl/WI/2026/14"))
        );
    }

    #[test]
    fn resolver_by_ecli() {
        assert_eq!(
            resolve_cdx_url(BASE, "by-ecli/ECLI:AT:LVWGNI:2018:LVwG.AV.72.001.2018"),
            Ok(format!(
                "{BASE}/AT/judikatur/by-ecli/ECLI:AT:LVWGNI:2018:LVwG.AV.72.001.2018"
            ))
        );
    }

    #[test]
    fn resolver_by_document_number_history() {
        assert_eq!(
            resolve_cdx_url(BASE, "by-document-number/history/NOR40277843"),
            Ok(format!("{BASE}/AT/history/by-document-number/NOR40277843"))
        );
    }

    #[test]
    fn resolver_by_document_number_bundesrecht() {
        assert_eq!(
            resolve_cdx_url(BASE, "by-document-number/bundesrecht/NOR12345"),
            Ok(format!("{BASE}/AT/bundesrecht/by-document-number/NOR12345"))
        );
    }

    #[test]
    fn resolver_by_document_number_unknown_domain() {
        // sonstige has no by-document-number resolver -> rejected locally
        assert!(resolve_cdx_url(BASE, "by-document-number/sonstige/NOR1").is_err());
    }

    #[test]
    fn resolver_by_document_number_missing_domain() {
        assert!(resolve_cdx_url(BASE, "by-document-number/NOR1").is_err());
    }

    #[test]
    fn resolver_bgbl_empty() {
        assert!(resolve_cdx_url(BASE, "bgbl/").is_err());
    }

    #[test]
    fn unknown_route() {
        assert!(resolve_cdx_url(BASE, "foo/bar").is_err());
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "doc/ATBR1234/meta"),
            Ok("https://api.example.com/api/AT/bundesrecht/doc/ATBR1234/meta".to_string())
        );
    }

    // --- attachment link resolution + envelope splitting ---

    #[test]
    fn resolves_only_attachment_links_keeping_fragment() {
        let base = "https://example.com/api";
        let body = r#"{"a":"cdx-at://doc/ATBR12/attachment/content_1.pdf#page=3","b":"cdx-at://doc/ATBR12/meta","c":"plain"}"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains(
            "https://example.com/api/AT/bundesrecht/doc/ATBR12/attachment/content_1.pdf#page=3"
        ));
        assert!(
            out.contains("\"cdx-at://doc/ATBR12/meta\""),
            "non-attachment cdx:// must be left untouched: {out}"
        );
        assert!(out.contains("\"plain\""));
    }

    #[test]
    fn resolves_attachment_with_query_and_fragment() {
        let base = "https://example.com/api";
        let body = r#""cdx-at://doc/ATBR12/attachment/content_1.pdf?v=2#page=7""#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains(
            "https://example.com/api/AT/bundesrecht/doc/ATBR12/attachment/content_1.pdf"
        ));
        assert!(
            out.ends_with("#page=7\""),
            "fragment must be preserved verbatim: {out}"
        );
    }

    #[test]
    fn non_attachment_url_in_sources_stays_cdx() {
        let base = "https://example.com/api";
        let body = r#"{"content":1,"sources":[{"__kind":"url","url":"cdx-at://doc/ATBR12/meta"}]}"#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("\"cdx-at://doc/ATBR12/meta\""),
            "non-attachment cdx:// in sources must stay unresolved: {out}"
        );
    }

    #[test]
    fn resolves_attachment_amid_multibyte_german_text() {
        let base = "https://example.com/api";
        // Multi-byte chars (Ö, ü, ß) IMMEDIATELY adjacent to the token guard the char-boundary
        // slicing - this resolver runs on Austrian legal content where a mis-sliced index panics.
        let body = "Verordnung über…cdx-at://doc/ATBR12/attachment/content_1.pdf#page=3,Änderung ü";
        let out = resolve_attachment_links(body, base);
        assert!(out.contains(
            "https://example.com/api/AT/bundesrecht/doc/ATBR12/attachment/content_1.pdf#page=3"
        ));
        assert!(
            out.starts_with("Verordnung über…https://"),
            "multi-byte text immediately before token preserved: {out}"
        );
        assert!(
            out.ends_with(",Änderung ü"),
            "multi-byte text after the delimiter preserved: {out}"
        );
    }

    #[test]
    fn resolves_multiple_attachment_links_in_one_body() {
        let base = "https://example.com/api";
        let body = r#"["cdx-at://doc/ATBR1/attachment/a.pdf","cdx-at://doc/ATJD2/attachment/b.pdf#page=2"]"#;
        let out = resolve_attachment_links(body, base);
        assert!(out.contains("https://example.com/api/AT/bundesrecht/doc/ATBR1/attachment/a.pdf"));
        assert!(
            out.contains("https://example.com/api/AT/judikatur/doc/ATJD2/attachment/b.pdf#page=2")
        );
        assert!(
            !out.contains("cdx-at://"),
            "all attachment links resolved: {out}"
        );
    }

    #[test]
    fn unknown_prefix_attachment_falls_back_to_cdx() {
        let base = "https://example.com/api";
        // Unknown ID prefix → resolve_cdx_url errors → original cdx:// link re-emitted unchanged.
        let body = r#""cdx-at://doc/XXYY9/attachment/c.pdf#page=1""#;
        let out = resolve_attachment_links(body, base);
        assert!(
            out.contains("\"cdx-at://doc/XXYY9/attachment/c.pdf#page=1\""),
            "unknown-prefix attachment link must be left as cdx://: {out}"
        );
    }
}
