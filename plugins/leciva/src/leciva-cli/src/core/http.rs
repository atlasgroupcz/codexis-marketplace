use crate::core::error::CliError;

const LECIVA_SCHEME: &str = "leciva://";

/// Display ID prefix -> API path (Czech SÚKL medicines domain).
const ID_PREFIXES: &[(&str, &str)] = &[("CZSUKL", "CZ/leciva")];

/// The plugin owns a single domain, so id-less routes (values) use this fixed path.
const LECIVA_API_PATH: &str = "CZ/leciva";

pub(crate) struct LecivaApi;

impl cdx_common::search_engine::SearchApi for LecivaApi {
    fn id_prefixes(&self) -> &'static [(&'static str, &'static str)] {
        ID_PREFIXES
    }
    fn cdx_scheme(&self) -> &'static str {
        LECIVA_SCHEME
    }
    fn plugin_name(&self) -> &'static str {
        "leciva"
    }
    fn resolve_cdx_url(&self, base_url: &str, cdx_path: &str) -> Result<String, String> {
        resolve_cdx_url(base_url, cdx_path)
    }
    // Metadata-only: PIL/SPC links in the record are absolute SÚKL URLs already.
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
        &LecivaApi,
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
    cdx_common::search_engine::execute_get_request(&LecivaApi, base_url, auth_header, resource, dry_run)
}

/// Resolve a leciva:// path (everything after "leciva://") into a full HTTP URL.
/// Routes:
///   doc/values/<field>       -> {base}/CZ/leciva/doc/values/<field>
///   doc/<DOC_ID>/<endpoint>  -> {base}/CZ/leciva/doc/<DOC_ID>/<endpoint>  (meta, cena; endpoint
///                               may carry a query string, e.g. meta?full=true)
///   resolve/<DISPLAY_ID>     -> {base}/resolve/<DISPLAY_ID>
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    // `doc/values/...` must be matched before the generic `doc/...` branch.
    if let Some(field) = cdx_path.strip_prefix("doc/values/") {
        if field.is_empty() {
            return Err("Missing field after doc/values/".to_string());
        }
        return Ok(format!("{base}/{LECIVA_API_PATH}/doc/values/{field}"));
    }

    if let Some(rest) = cdx_path.strip_prefix("doc/") {
        let (id, endpoint) = rest
            .split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = cdx_common::http::lookup_display_id_prefix(id, ID_PREFIXES)?;
        return Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}"));
    }

    if let Some(rest) = cdx_path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        return Ok(format!("{base}/resolve/{rest}"));
    }

    Err(format!("Unknown leciva:// path: {cdx_path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    #[test]
    fn doc_meta() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSUKL0000009/meta"),
            Ok(format!("{BASE}/CZ/leciva/doc/CZSUKL0000009/meta"))
        );
    }

    #[test]
    fn doc_meta_full_query_passes_through() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSUKL0000009/meta?full=true"),
            Ok(format!("{BASE}/CZ/leciva/doc/CZSUKL0000009/meta?full=true"))
        );
    }

    #[test]
    fn doc_cena() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSUKL0000009/cena"),
            Ok(format!("{BASE}/CZ/leciva/doc/CZSUKL0000009/cena"))
        );
    }

    #[test]
    fn doc_values() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/values/forma"),
            Ok(format!("{BASE}/CZ/leciva/doc/values/forma"))
        );
    }

    #[test]
    fn doc_values_empty_field_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/values/").is_err());
    }

    #[test]
    fn doc_unknown_prefix_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZRZP1/meta").is_err());
    }

    #[test]
    fn doc_missing_endpoint_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZSUKL0000009").is_err());
    }

    #[test]
    fn doc_empty_endpoint_after_slash_rejected() {
        assert!(resolve_cdx_url(BASE, "doc/CZSUKL0000009/").is_err());
    }

    #[test]
    fn resolve_empty_rejected() {
        assert!(resolve_cdx_url(BASE, "resolve/").is_err());
    }

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/CZSUKL0000009"),
            Ok(format!("{BASE}/resolve/CZSUKL0000009"))
        );
    }

    #[test]
    fn unknown_route_rejected() {
        assert!(resolve_cdx_url(BASE, "foo/bar").is_err());
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "doc/values/atc"),
            Ok("https://api.example.com/api/CZ/leciva/doc/values/atc".to_string())
        );
    }
}
