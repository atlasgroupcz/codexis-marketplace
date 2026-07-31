
use crate::core::error::CliError;

const REGISTR_SMLUV_SCHEME: &str = "registr-smluv://";

/// Display ID prefix -> API path (Czech smlouvy domain)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("CZRS", "CZ/smlouvy"),
];

pub(crate) struct RegistrSmluvApi;

impl cdx_common::search_engine::SearchApi for RegistrSmluvApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        REGISTR_SMLUV_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "registr-smluv"
    }
    fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String> {
        resolve_cdx_url(base_url, cdx_path)
    }
    // Registr smluv has no attachments to rewrite.
    fn resolve_attachment_links(&self, body: &str, _base_url: &str) -> String {
        body.to_string()
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
        &RegistrSmluvApi, base_url, auth_header, source_code, payload, dry_run, sort, order,
    )
}

pub(crate) fn execute_get_request(
    base_url: &str,
    auth_header: Option<&str>,
    resource: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    cdx_common::search_engine::execute_get_request(&RegistrSmluvApi, base_url, auth_header, resource, dry_run)
}

/// Resolve a registr-smluv:// path (everything after "registr-smluv://") into a full HTTP URL.
/// Supported routes: `doc/<DOC_ID>/meta` (contract record) and `resolve/<DISPLAY_ID>`.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    if let Some(rest) = cdx_path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = cdx_common::http::lookup_display_id_prefix(id, ID_PREFIXES)?;
        Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}"))
    } else if let Some(rest) = cdx_path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        Ok(format!("{base}/resolve/{rest}"))
    } else {
        Err(format!("Unknown registr-smluv:// path: {cdx_path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    // --- URL resolution tests ---

    #[test]
    fn doc_czrs_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZRS1234/meta"),
            Ok(format!("{BASE}/CZ/smlouvy/doc/CZRS1234/meta"))
        );
    }

    #[test]
    fn doc_unknown_prefix() {
        assert!(resolve_cdx_url(BASE, "doc/UNKNOWN123/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZRS1234").is_err());
    }

    #[test]
    fn doc_empty_endpoint() {
        assert!(resolve_cdx_url(BASE, "doc/CZRS1234/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/CZRS1234"),
            Ok(format!("{BASE}/resolve/CZRS1234"))
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
            resolve_cdx_url("https://api.example.com/api/", "resolve/CZRS1"),
            Ok("https://api.example.com/api/resolve/CZRS1".to_string())
        );
    }

    // --- envelope splitting ---

}
