use std::env;
use std::process::{Command, Stdio};

const CDX_SCHEME: &str = "cdx-cz-spp://";

/// Display ID prefix -> API path (Czech sbirkapp domain)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("CZSB", "CZ/sbirkapp"),
];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return;
    }

    let base_url = match env::var("CDX_CZ_SPP_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CDX_CZ_SPP_API_URL must be set (e.g., export CDX_CZ_SPP_API_URL=\"https://search.example.com/api\")");
            std::process::exit(2);
        }
    };

    let mut resolved_args: Vec<String> = Vec::with_capacity(args.len());
    for arg in &args {
        if let Some(rest) = arg.strip_prefix(CDX_SCHEME) {
            match resolve_cdx_url(&base_url, rest) {
                Ok(url) => resolved_args.push(url),
                Err(msg) => {
                    eprintln!("Error: {msg}");
                    std::process::exit(1);
                }
            }
        } else {
            resolved_args.push(arg.clone());
        }
    }

    let status = Command::new("curl")
        .args(&resolved_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("failed to run curl: {err}");
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("cdx-cz-spp [curl args] <cdx-cz-spp://...>");
    println!();
    println!("Resolves cdx-cz-spp:// URLs to the ai-scraper search API and delegates to curl.");
    println!("Requires CDX_CZ_SPP_API_URL to be set in the environment.");
    println!();
    println!("Supported URL patterns:");
    println!("  cdx-cz-spp://search/CZSB              Search documents");
    println!("  cdx-cz-spp://doc/{{displayId}}/{{endpoint}}  Document operations");
    println!("  cdx-cz-spp://resolve/{{id}}                  Resolve display ID");
}

/// Resolve a cdx-cz-spp:// path (everything after "cdx-cz-spp://") into a full HTTP URL.
/// Returns Err with an error message on unknown routes.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    // Split path and query string
    let (path, query) = match cdx_path.find('?') {
        Some(idx) => (&cdx_path[..idx], &cdx_path[idx..]),
        None => (cdx_path, ""),
    };

    if let Some(domain) = path.strip_prefix("search/") {
        let api_path = lookup_search_prefix(domain)?;
        Ok(format!("{base}/{api_path}/search{query}"))
    } else if let Some(rest) = path.strip_prefix("doc/") {
        let (id, endpoint) = rest.split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = lookup_display_id_prefix(id)?;
        Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}{query}"))
    } else if let Some(rest) = path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        Ok(format!("{base}/resolve/{rest}{query}"))
    } else {
        Err(format!("Unknown cdx-cz-spp:// path: {path}"))
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

    // --- search/ routes ---

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

    // --- doc/ routes ---

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
            Ok(format!("{BASE}/CZ/sbirkapp/doc/CZSB5678/related"))
        );
    }

    #[test]
    fn doc_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB1/text?part=paragraf-1"),
            Ok(format!("{BASE}/CZ/sbirkapp/doc/CZSB1/text?part=paragraf-1"))
        );
    }

    #[test]
    fn doc_attachment() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/CZSB100/attachment/content_1.pdf"),
            Ok(format!("{BASE}/CZ/sbirkapp/doc/CZSB100/attachment/content_1.pdf"))
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

    // --- resolve/ routes ---

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

    // --- unknown routes ---

    #[test]
    fn unknown_route() {
        assert!(resolve_cdx_url(BASE, "foo/bar").is_err());
    }

    // --- base URL trailing slash handling ---

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "resolve/CZSB1"),
            Ok("https://api.example.com/api/resolve/CZSB1".to_string())
        );
    }
}
