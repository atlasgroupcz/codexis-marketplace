const CDX_SCHEME: &str = "cdx-at://";

/// Display ID prefix -> API path (same as cdx-at binary)
const ID_PREFIXES: &[(&str, &str)] = &[
    ("ATBR", "AT/bundesrecht"),
    ("ATJD", "AT/judikatur"),
    ("ATLR", "AT/landesrecht"),
    ("ATSO", "AT/sonstige"),
    ("ATHI", "AT/history"),
];

fn main() {
    let base_url = match std::env::var("CDX_AT_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CDX_AT_API_URL must be set (e.g., export CDX_AT_API_URL=\"https://search.example.com/api\")");
            std::process::exit(2);
        }
    };

    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
        .expect("failed to read stdin");

    let output = rewrite_cdx_at_links(&input, &base_url);
    print!("{output}");
}

fn rewrite_cdx_at_links(html: &str, base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let needle = "href=\"cdx-at://";
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(href_start) = remaining.find(needle) {
        result.push_str(&remaining[..href_start]);

        let url_start = href_start + "href=\"".len();
        let after_needle = &remaining[url_start..];

        let closing_quote = match after_needle.find('"') {
            Some(pos) => pos,
            None => {
                result.push_str(&remaining[href_start..href_start + needle.len()]);
                remaining = &remaining[href_start + needle.len()..];
                continue;
            }
        };

        let cdx_url = &after_needle[..closing_quote];
        let path = &cdx_url[CDX_SCHEME.len()..];

        match resolve_cdx_at_path(path, base) {
            Some(resolved) => {
                result.push_str("href=\"");
                result.push_str(&resolved);
                result.push('"');

                let after_closing = &remaining[url_start + closing_quote + 1..];
                let tag_end = after_closing.find('>').unwrap_or(after_closing.len());
                let rest_of_tag = &after_closing[..tag_end];

                if !rest_of_tag.contains(" target=") {
                    result.push_str(" target=\"_blank\"");
                }

                remaining = after_closing;
            }
            None => {
                result.push_str(&remaining[href_start..url_start + closing_quote + 1]);
                remaining = &remaining[url_start + closing_quote + 1..];
            }
        }
    }

    result.push_str(remaining);
    result
}

fn resolve_cdx_at_path(path: &str, base: &str) -> Option<String> {
    let suffix_pos = path.find(|c| c == '?' || c == '#');
    let (route, suffix) = match suffix_pos {
        Some(pos) => (&path[..pos], &path[pos..]),
        None => (path, ""),
    };

    let mut segments = route.splitn(2, '/');
    let first = segments.next().unwrap_or("");
    let rest = segments.next().unwrap_or("");

    match first {
        "search" => {
            let api_path = lookup_prefix(rest)?;
            Some(format!("{base}/{api_path}/search{suffix}"))
        }
        "doc" => {
            let (display_id, endpoint) = rest.split_once('/')?;
            if endpoint.is_empty() {
                return None;
            }
            let api_path = lookup_display_id_prefix(display_id)?;
            Some(format!("{base}/{api_path}/doc/{display_id}/{endpoint}{suffix}"))
        }
        "resolve" => {
            Some(format!("{base}/resolve/{rest}{suffix}"))
        }
        _ => None,
    }
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

    #[test]
    fn rewrites_atbr_attachment_link() {
        let input = r#"<a href="cdx-at://doc/ATBR1234/attachment/content_1.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1234/attachment/content_1.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_atjd_attachment_link() {
        let input = r#"<a href="cdx-at://doc/ATJD5678/attachment/content_1.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/judikatur/doc/ATJD5678/attachment/content_1.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_athi_attachment_link() {
        let input = r#"<a href="cdx-at://doc/ATHI100/attachment/content_1.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/history/doc/ATHI100/attachment/content_1.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn preserves_page_fragment() {
        let input = r#"<a href="cdx-at://doc/ATBR1/attachment/content_1.pdf#page=42">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/attachment/content_1.pdf#page=42" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn preserves_query_string() {
        let input = r#"<a href="cdx-at://doc/ATBR1/text?part=paragraf-1">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/text?part=paragraf-1" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_search_link() {
        let input = r#"<a href="cdx-at://search/ATJD">Search</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/judikatur/search" target="_blank">Search</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_resolve_link() {
        let input = r#"<a href="cdx-at://resolve/ATBR1234">Resolve</a>"#;
        let expected = r#"<a href="https://search.example.com/api/resolve/ATBR1234" target="_blank">Resolve</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_multiple_links() {
        let input = r#"<a href="cdx-at://doc/ATBR1/attachment/a.pdf">A</a> and <a href="cdx-at://doc/ATJD2/attachment/b.pdf">B</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/attachment/a.pdf" target="_blank">A</a> and <a href="https://search.example.com/api/AT/judikatur/doc/ATJD2/attachment/b.pdf" target="_blank">B</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn preserves_non_cdx_at_links() {
        let input = r#"<a href="https://example.com">ext</a> <a href="cdx-at://doc/ATBR1/attachment/a.pdf">cdx</a>"#;
        let expected = r#"<a href="https://example.com">ext</a> <a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/attachment/a.pdf" target="_blank">cdx</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn ignores_cdx_sk_scheme() {
        let input = r#"<a href="cdx-sk://doc/SKEZ1/meta">SK</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), input);
    }

    #[test]
    fn passthrough_when_no_links() {
        let input = "<p>Hello world, no links here</p>";
        assert_eq!(rewrite_cdx_at_links(input, BASE), input);
    }

    #[test]
    fn unknown_prefix_passes_through() {
        let input = r#"<a href="cdx-at://doc/UNKNOWN123/meta">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), input);
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(rewrite_cdx_at_links("", BASE), "");
    }

    #[test]
    fn preserves_existing_target_blank() {
        let input = r#"<a href="cdx-at://doc/ATBR1/attachment/a.pdf" target="_blank">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/attachment/a.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }

    #[test]
    fn base_url_trailing_slash_stripped() {
        let input = r#"<a href="cdx-at://doc/ATBR1/attachment/a.pdf">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/attachment/a.pdf" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, "https://search.example.com/api/"), expected);
    }

    #[test]
    fn preserves_query_and_fragment() {
        let input = r#"<a href="cdx-at://doc/ATBR1/text?part=paragraf-1#page=5">Doc</a>"#;
        let expected = r#"<a href="https://search.example.com/api/AT/bundesrecht/doc/ATBR1/text?part=paragraf-1#page=5" target="_blank">Doc</a>"#;
        assert_eq!(rewrite_cdx_at_links(input, BASE), expected);
    }
}
