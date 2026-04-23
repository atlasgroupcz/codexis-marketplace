const CDX_SCHEME: &str = "cdx-cz-spp://";

/// Display ID prefix -> API path (Czech sbirkapp domain)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("CZSB", "CZ/sbirkapp"),
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

fn main() {
    let base_url = match std::env::var("CDX_CZ_SPP_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CDX_CZ_SPP_API_URL must be set (e.g., export CDX_CZ_SPP_API_URL=\"https://search.example.com/api\")");
            std::process::exit(2);
        }
    };

    // Signing disabled — secret must not be on the VM (user has root).
    // When re-enabling, move signing to the daemon side (post-processing).
    let signing_secret: Option<String> = None;

    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .expect("failed to read stdin");

    let output = rewrite_cdx_cz_spp_links(&input, &base_url, signing_secret.as_deref());
    print!("{output}");
}

fn rewrite_cdx_cz_spp_links(html: &str, base_url: &str, signing_secret: Option<&str>) -> String {
    let base = base_url.trim_end_matches('/');
    let needle = "href=\"cdx-cz-spp://";
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
        let path = &cdx_url[CDX_SCHEME.len()..]; // everything after cdx-cz-spp://

        match resolve_cdx_cz_spp_path(path, base, signing_secret) {
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

fn resolve_cdx_cz_spp_path(path: &str, base: &str, signing_secret: Option<&str>) -> Option<String> {
    let suffix_pos = path.find(|c| c == '?' || c == '#');
    let (route, suffix) = match suffix_pos {
        Some(pos) => (&path[..pos], &path[pos..]),
        None => (path, ""),
    };

    let mut segments = route.splitn(2, '/');
    let first = segments.next().unwrap_or("");
    let rest = segments.next().unwrap_or("");

    let url = match first {
        "search" => {
            let api_path = lookup_prefix(rest)?;
            format!("{base}/{api_path}/search")
        }
        "doc" => {
            let (display_id, endpoint) = rest.split_once('/')?;
            if endpoint.is_empty() {
                return None;
            }
            let api_path = lookup_display_id_prefix(display_id)?;
            format!("{base}/{api_path}/doc/{display_id}/{endpoint}")
        }
        "resolve" => {
            format!("{base}/resolve/{rest}")
        }
        _ => return None,
    };

    if let Some(secret) = signing_secret {
        if url.contains("/attachment/") {
            let url_path = extract_url_path(&url);
            let sig = sign_path(secret.as_bytes(), url_path);
            return Some(format!("{url}?sig={sig}{suffix}"));
        }
    }

    Some(format!("{url}{suffix}"))
}

fn lookup_prefix(code: &str) -> Option<&'static str> {
    ID_PREFIXES.iter().find(|(k, _)| *k == code).map(|(_, v)| *v)
}

fn lookup_display_id_prefix(id: &str) -> Option<&'static str> {
    ID_PREFIXES.iter().find(|(k, _)| id.starts_with(k)).map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://search.example.com/api";

    // --- Basic rewriting ---

    #[test]
    fn rewrites_czsb_attachment_link() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/attachment/content_1.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1234/attachment/content_1.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Fragment preservation ---

    #[test]
    fn preserves_page_fragment() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/attachment/content_1.pdf#page=180">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1234/attachment/content_1.pdf#page=180" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Query string preservation ---

    #[test]
    fn preserves_query_string() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1/text?part=paragraf-1">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/text?part=paragraf-1" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Other route types ---

    #[test]
    fn rewrites_search_link() {
        let input = r#"<a href="cdx-cz-spp://search/CZSB">Search</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/search" target="_blank">Search</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    #[test]
    fn rewrites_resolve_link() {
        let input = r#"<a href="cdx-cz-spp://resolve/CZSB1234">Resolve</a>"#;
        let expected = r#"<a href="https://search.example.com/api/resolve/CZSB1234" target="_blank">Resolve</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Multiple links ---

    #[test]
    fn rewrites_multiple_links() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1/attachment/a.pdf">A</a> and <a href="cdx-cz-spp://doc/CZSB2/attachment/b.pdf">B</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/attachment/a.pdf" target="_blank">A</a> and <a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB2/attachment/b.pdf" target="_blank">B</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Passthrough / edge cases ---

    #[test]
    fn preserves_non_cdx_cz_spp_links() {
        let input = r#"<a href="https://example.com">ext</a> <a href="cdx-cz-spp://doc/CZSB1/attachment/a.pdf">cdx</a>"#;
        let expected = r#"<a href="https://example.com">ext</a> <a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/attachment/a.pdf" target="_blank">cdx</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    #[test]
    fn ignores_cdx_scheme_without_cz_spp() {
        let input = r#"<a href="cdx://doc/CR10">CODEXIS</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), input);
    }

    #[test]
    fn passthrough_when_no_links() {
        let input = "<p>Hello world, no links here</p>";
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), input);
    }

    #[test]
    fn does_not_rewrite_cdx_cz_spp_in_text_content() {
        let input = r#"<p>Visit cdx-cz-spp://doc/CZSB1/attachment/a.pdf for details</p>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), input);
    }

    #[test]
    fn unknown_prefix_passes_through() {
        let input = r#"<a href="cdx-cz-spp://doc/UNKNOWN123/meta">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), input);
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(rewrite_cdx_cz_spp_links("", BASE, None), "");
    }

    #[test]
    fn preserves_existing_target_blank() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1/attachment/a.pdf" target="_blank">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/attachment/a.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1/attachment/a.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/attachment/a.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, "https://search.example.com/api/", None), expected);
    }

    // --- query + fragment combined ---

    #[test]
    fn preserves_query_and_fragment() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1/text?part=paragraf-1#page=5">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/CZ/sbirkapp/doc/CZSB1/text?part=paragraf-1#page=5" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_cz_spp_links(input, BASE, None), expected);
    }

    // --- Signing tests ---

    const TEST_SECRET: &str = "test-secret-1234";

    #[test]
    fn sign_path_matches_test_vector_1() {
        let sig = sign_path(
            TEST_SECRET.as_bytes(),
            "/api/SK/ezbierka/doc/SKEZ1/attachment/content_1.pdf",
        );
        assert_eq!(sig, "JQHN6fVMHIAhH1lTFvbbctGHJerylRxSj3isHQStYl8");
    }

    #[test]
    fn attachment_link_gets_sig_when_secret_present() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/attachment/content_1.pdf">Doc</a>"#;
        let output = rewrite_cdx_cz_spp_links(input, BASE, Some(TEST_SECRET));
        assert!(output.contains("?sig="));
    }

    #[test]
    fn attachment_link_no_sig_when_secret_absent() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/attachment/content_1.pdf">Doc</a>"#;
        let output = rewrite_cdx_cz_spp_links(input, BASE, None);
        assert!(!output.contains("?sig="));
    }

    #[test]
    fn attachment_link_sig_before_fragment() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/attachment/content_1.pdf#page=180">Doc</a>"#;
        let output = rewrite_cdx_cz_spp_links(input, BASE, Some(TEST_SECRET));
        let sig_pos = output.find("?sig=").expect("should have sig");
        let frag_pos = output.find("#page=").expect("should have fragment");
        assert!(sig_pos < frag_pos);
    }

    #[test]
    fn non_attachment_doc_link_no_sig() {
        let input = r#"<a href="cdx-cz-spp://doc/CZSB1234/meta">Doc</a>"#;
        let output = rewrite_cdx_cz_spp_links(input, BASE, Some(TEST_SECRET));
        assert!(!output.contains("?sig="));
    }
}
