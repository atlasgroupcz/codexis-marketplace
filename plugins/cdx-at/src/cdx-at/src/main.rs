use std::env;
use std::process::{Command, Stdio};

const CDX_SCHEME: &str = "cdx-at://";

/// Display ID prefix -> API path (Austrian RIS domains)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("ATBR", "AT/bundesrecht"),
    ("ATJD", "AT/judikatur"),
    ("ATLR", "AT/landesrecht"),
    ("ATSO", "AT/sonstige"),
    ("ATHI", "AT/history"),
];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return;
    }

    let base_url = match env::var("CDX_AT_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CDX_AT_API_URL must be set (e.g., export CDX_AT_API_URL=\"https://search.example.com/api\")");
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
    println!("cdx-at [curl args] <cdx-at://...>");
    println!();
    println!("Resolves cdx-at:// URLs to the ai-scraper search API and delegates to curl.");
    println!("Requires CDX_AT_API_URL to be set in the environment.");
    println!();
    println!("Supported URL patterns:");
    println!("  cdx-at://search/{{ATBR|ATJD|ATLR|ATSO|ATHI}}  Search documents");
    println!("  cdx-at://doc/{{displayId}}/{{endpoint}}           Document operations");
    println!("  cdx-at://resolve/{{id}}                           Resolve display ID");
}

/// Resolve a cdx-at:// path (everything after "cdx-at://") into a full HTTP URL.
fn resolve_cdx_url(base_url: &str, cdx_path: &str) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');

    let (path, query) = match cdx_path.find('?') {
        Some(idx) => (&cdx_path[..idx], &cdx_path[idx..]),
        None => (cdx_path, ""),
    };

    let encoded_query = encode_query_values(query);

    if let Some(domain) = path.strip_prefix("search/") {
        let api_path = lookup_search_prefix(domain)?;
        Ok(format!("{base}/{api_path}/search{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("doc/") {
        let (id, endpoint) = rest.split_once('/')
            .ok_or_else(|| format!("Missing endpoint after doc ID in: doc/{rest}"))?;
        if endpoint.is_empty() {
            return Err(format!("Empty endpoint after doc/{id}/"));
        }
        let api_path = lookup_display_id_prefix(id)?;
        Ok(format!("{base}/{api_path}/doc/{id}/{endpoint}{encoded_query}"))
    } else if let Some(rest) = path.strip_prefix("resolve/") {
        if rest.is_empty() {
            return Err("Missing ID after resolve/".to_string());
        }
        Ok(format!("{base}/resolve/{rest}{encoded_query}"))
    } else {
        Err(format!("Unknown cdx-at:// path: {path}"))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.example.com/api";

    // --- search/ routes ---

    #[test]
    fn search_bundesrecht() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATBR"),
            Ok(format!("{BASE}/AT/bundesrecht/search"))
        );
    }

    #[test]
    fn search_judikatur() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATJD"),
            Ok(format!("{BASE}/AT/judikatur/search"))
        );
    }

    #[test]
    fn search_landesrecht() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATLR"),
            Ok(format!("{BASE}/AT/landesrecht/search"))
        );
    }

    #[test]
    fn search_sonstige() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATSO"),
            Ok(format!("{BASE}/AT/sonstige/search"))
        );
    }

    #[test]
    fn search_history() {
        assert_eq!(
            resolve_cdx_url(BASE, "search/ATHI"),
            Ok(format!("{BASE}/AT/history/search"))
        );
    }

    #[test]
    fn search_unknown_domain() {
        assert!(resolve_cdx_url(BASE, "search/UNKNOWN").is_err());
    }

    // --- doc/ routes ---

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
            Ok(format!("{BASE}/AT/history/doc/ATHI100/attachment/content_1.pdf"))
        );
    }

    #[test]
    fn doc_with_query_string() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATBR1/text?part=paragraf-1"),
            Ok(format!("{BASE}/AT/bundesrecht/doc/ATBR1/text?part=paragraf-1"))
        );
    }

    #[test]
    fn doc_with_german_diacritics_in_query() {
        assert_eq!(
            resolve_cdx_url(BASE, "doc/ATJD1/text?part=begründung"),
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

    // --- resolve/ routes ---

    #[test]
    fn resolve_basic() {
        assert_eq!(
            resolve_cdx_url(BASE, "resolve/ATBR1234"),
            Ok(format!("{BASE}/resolve/ATBR1234"))
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

    // --- base URL handling ---

    #[test]
    fn base_url_trailing_slash_stripped() {
        assert_eq!(
            resolve_cdx_url("https://api.example.com/api/", "resolve/ATBR1"),
            Ok("https://api.example.com/api/resolve/ATBR1".to_string())
        );
    }
}
