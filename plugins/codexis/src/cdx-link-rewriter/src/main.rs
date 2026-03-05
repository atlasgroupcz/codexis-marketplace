use std::env;
use std::io::{self, Read, Write};

const HREF_PREFIX: &str = "href=\"cdx://";

fn main() {
    let base_url = match env::var("CODEXIS_BASE_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().trim_end_matches('/').to_string(),
        _ => {
            eprintln!("CODEXIS_BASE_URL must be set (e.g., export CODEXIS_BASE_URL=\"https://next.codexis.cz\")");
            std::process::exit(2);
        }
    };

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Failed to read stdin: {}", e);
        std::process::exit(1);
    }

    let output = rewrite_cdx_links(&input, &base_url);

    if let Err(e) = io::stdout().write_all(output.as_bytes()) {
        eprintln!("Failed to write stdout: {}", e);
        std::process::exit(1);
    }
}

/// Rewrites all `href="cdx://..."` attributes in the HTML to use the given base URL.
///
/// Only rewrites links inside href attributes (double-quoted). This avoids rewriting
/// cdx:// that might appear in text content or other contexts.
fn rewrite_cdx_links(html: &str, base_url: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start) = remaining.find(HREF_PREFIX) {
        // Copy everything before the match
        result.push_str(&remaining[..start]);

        // Write the new href prefix
        result.push_str("href=\"");
        result.push_str(base_url);
        result.push('/');

        // Skip past href="cdx://
        let after_scheme = &remaining[start + HREF_PREFIX.len()..];

        // Find the closing quote
        if let Some(end) = after_scheme.find('"') {
            let path = &after_scheme[..end];
            // Append path (strip leading slash if present to avoid double slash)
            let trimmed_path = path.trim_start_matches('/');
            result.push_str(trimmed_path);
            result.push('"');
            remaining = &after_scheme[end + 1..];
        } else {
            // Malformed: no closing quote, just pass through the rest
            result.push_str(after_scheme);
            remaining = "";
        }
    }

    // Append any remaining content
    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://next.codexis.cz";

    #[test]
    fn rewrites_single_cdx_link() {
        let input = r#"<p><a href="cdx://doc/CR10_2025_01_01">Document</a></p>"#;
        let expected = r#"<p><a href="https://next.codexis.cz/doc/CR10_2025_01_01">Document</a></p>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }

    #[test]
    fn rewrites_multiple_cdx_links() {
        let input = r#"<a href="cdx://doc/A">a</a> and <a href="cdx://doc/B">b</a>"#;
        let expected = r#"<a href="https://next.codexis.cz/doc/A">a</a> and <a href="https://next.codexis.cz/doc/B">b</a>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }

    #[test]
    fn preserves_non_cdx_links() {
        let input = r#"<a href="https://example.com">ext</a> <a href="cdx://doc/X">cdx</a>"#;
        let expected = r#"<a href="https://example.com">ext</a> <a href="https://next.codexis.cz/doc/X">cdx</a>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }

    #[test]
    fn handles_bare_cdx_scheme() {
        let input = r#"<a href="cdx://">root</a>"#;
        let expected = r#"<a href="https://next.codexis.cz/">root</a>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }

    #[test]
    fn handles_cdx_with_leading_slash() {
        let input = r#"<a href="cdx:///doc/X">doc</a>"#;
        let expected = r#"<a href="https://next.codexis.cz/doc/X">doc</a>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }

    #[test]
    fn passthrough_when_no_links() {
        let input = "<p>Hello world, no links here</p>";
        assert_eq!(rewrite_cdx_links(input, BASE), input);
    }

    #[test]
    fn does_not_rewrite_cdx_in_text_content() {
        let input = r#"<p>Visit cdx://doc/X for details</p>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), input);
    }

    #[test]
    fn handles_trailing_slash_on_base_url() {
        let base = "https://next.codexis.cz/";
        let input = r#"<a href="cdx://doc/X">doc</a>"#;
        // base_url is trimmed in main(), but rewrite_cdx_links always adds a slash
        let trimmed_base = base.trim_end_matches('/');
        let expected = r#"<a href="https://next.codexis.cz/doc/X">doc</a>"#;
        assert_eq!(rewrite_cdx_links(input, trimmed_base), expected);
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(rewrite_cdx_links("", BASE), "");
    }

    #[test]
    fn preserves_other_attributes() {
        let input = r#"<a href="cdx://doc/X" class="link" target="_blank">doc</a>"#;
        let expected = r#"<a href="https://next.codexis.cz/doc/X" class="link" target="_blank">doc</a>"#;
        assert_eq!(rewrite_cdx_links(input, BASE), expected);
    }
}
