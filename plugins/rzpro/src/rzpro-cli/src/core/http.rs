use crate::core::error::CliError;

const RZPRO_SCHEME: &str = "rzpro://";

/// Search source code -> API path (used by the `SearchApi` trait to route
/// `/search`). Both device (`CZRZP`) and person (`CZRZP-OS`) searches belong here.
const ID_PREFIXES: &[(&str, &str)] = &[("CZRZP", "CZ/rzpro"), ("CZRZP-OS", "CZ/rzpro/osoby")];

/// Doc-id prefix -> API path (used to resolve `doc/<ID>/...` GET routes). Only
/// sources with real per-document records belong here; person search (`CZRZP-OS`)
/// is fetched via `osoba/<key>`, never `doc/<id>`, so it is intentionally absent.
const DOC_ID_PREFIXES: &[(&str, &str)] = &[("CZRZP", "CZ/rzpro")];

/// The plugin owns a single domain, so id-less routes (values, osoba) use this
/// fixed path.
const RZPRO_API_PATH: &str = "CZ/rzpro";

pub(crate) struct RzproApi;

impl cdx_common::search_engine::SearchApi for RzproApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        RZPRO_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "rzpro"
    }
    fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String> {
        resolve_cdx_url(base_url, cdx_path)
    }
    // Metadata-only: the record carries an absolute KSRZIS portal URL already.
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
        &RzproApi,
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
        &RzproApi,
        base_url,
        auth_header,
        resource,
        dry_run,
    )
}

/// Resolve a rzpro:// path into a full HTTP URL.
/// Routes:
///   doc/values/<field>       -> {base}/CZ/rzpro/doc/values/<field>
///   osoba/<key>              -> {base}/CZ/rzpro/osoba/<key>
///   doc/<DOC_ID>/<endpoint>  -> {base}/CZ/rzpro/doc/<DOC_ID>/<endpoint>  (meta)
///   resolve/<DISPLAY_ID>     -> {base}/resolve/<DISPLAY_ID>
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    // `doc/values/...` must be matched before the generic `doc/...` branch.
    if let Some(field) = cdx_path.strip_prefix("doc/values/") {
        if field.is_empty() {
            return Err("Missing field after doc/values/".to_string());
        }
        return Ok(format!("{base}/{RZPRO_API_PATH}/doc/values/{field}"));
    }

    if let Some(key) = cdx_path.strip_prefix("osoba/") {
        if key.is_empty() {
            return Err("Missing key after osoba/".to_string());
        }
        return Ok(format!("{base}/{RZPRO_API_PATH}/osoba/{key}"));
    }

    if let Some(rest) = cdx_path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = cdx_common::http::lookup_display_id_prefix(id, DOC_ID_PREFIXES)?;
        return Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}"));
    }

    if let Some(rest) = cdx_path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        return Ok(format!("{base}/resolve/{rest}"));
    }

    Err(format!("Unknown rzpro:// path: {cdx_path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    #[test]
    fn doc_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZRZP00000019/meta"),
            Ok(format!("{BASE}/CZ/rzpro/doc/CZRZP00000019/meta"))
        );
    }

    #[test]
    fn osoba_by_key() {
        assert_eq!(
            resolve_cdx_url(BASE, "osoba/002276"),
            Ok(format!("{BASE}/CZ/rzpro/osoba/002276"))
        );
    }

    #[test]
    fn osoba_empty_key_rejected() {
        assert!(resolve_cdx_url(BASE, "osoba/").is_err());
    }

    #[test]
    fn doc_values() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/values/riziko"),
            Ok(format!("{BASE}/CZ/rzpro/doc/values/riziko"))
        );
    }

    #[test]
    fn doc_unknown_prefix_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZSUKL1/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZRZP00000019").is_err());
    }

    #[test]
    fn doc_empty_endpoint_after_slash_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZRZP00000019/").is_err());
    }

    #[test]
    fn resolve_empty_rejected() {
        assert!(resolve_cdx_url(BASE, "resolve/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/CZRZP00000019"),
            Ok(format!("{BASE}/resolve/CZRZP00000019"))
        );
    }

    #[test]
    fn unknown_route_rejected() {
        assert!(resolve_cdx_url(BASE, "foo/bar").is_err());
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "osoba/1"),
            Ok("https://api.example.com/api/CZ/rzpro/osoba/1".to_string())
        );
    }
}
