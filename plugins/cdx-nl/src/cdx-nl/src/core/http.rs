
use crate::core::error::CliError;
use cdx_common::curl::encode_query_values;

const CDX_NL_SCHEME: &str = "cdx-nl://";

/// Display ID prefix -> API path (only Dutch domains)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("NLBWB", "NL/wetten/bwb"),
    ("NLUIT", "NL/rechtspraak/uitspraken"),
    ("NLCVDR", "NL/lokaleregelgeving/cvdr"),
];

pub(crate) struct CdxNlApi;

impl cdx_common::search_engine::SearchApi for CdxNlApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        CDX_NL_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "cdx-nl"
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
        &CdxNlApi, base_url, auth_header, source_code, payload, dry_run, sort, order,
    )
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    cdx_common::search_engine::execute_get_request(&CdxNlApi, base_url, auth_header, resource, dry_run)
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
/// Pure string scan (no JSON parse, no regex) - the matched token ends at the first
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
        // slicing - this resolver runs on NL content where a mis-sliced index panics.
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

