//! `cdx-cli preview` — CGI handler behind the daemon's plugin preview endpoint
//! (`/rest/plugin-previews/…`, LM-207). The daemon passes the resource path of a
//! `codexis://doc/<DOC_ID>/text[?part=<PART>]` preview reference via `PATH_INFO`
//! / `QUERY_STRING`; the handler fetches the document markdown + metadata from
//! the CODEXIS API and emits a standalone HTML page (CGI response on stdout).

use std::env;

use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;

use crate::core::config::Config;
use crate::core::error::CliError;
use cdx_common::config::read_env;
use cdx_common::curl::{execute_curl, split_envelope_str};

const WEB_BASE_URL_ENV: &str = "CODEXIS_PLUGIN_BASE_URL";

pub(crate) fn execute_preview() -> Result<(), CliError> {
    let path_info = env::var("PATH_INFO").unwrap_or_default();
    let query_string = env::var("QUERY_STRING").unwrap_or_default();
    let stylesheet = stylesheet_href(&path_info);
    let response = match render_preview(&path_info, &query_string, &stylesheet) {
        Ok(page) => CgiResponse::ok(page),
        Err(PreviewError::UnsupportedPath) => CgiResponse::error(
            404,
            "Not Found",
            "Náhled pro tento zdroj není k dispozici.",
            &stylesheet,
        ),
        Err(PreviewError::Cli(error)) => CgiResponse::error(
            502,
            "Bad Gateway",
            &format!("Náhled dokumentu se nepodařilo načíst: {error}"),
            &stylesheet,
        ),
    };
    print!("{}", response.render());
    Ok(())
}

enum PreviewError {
    UnsupportedPath,
    Cli(CliError),
}

impl From<CliError> for PreviewError {
    fn from(error: CliError) -> Self {
        Self::Cli(error)
    }
}

struct CgiResponse {
    status: u16,
    reason: &'static str,
    body: String,
}

impl CgiResponse {
    fn ok(body: String) -> Self {
        Self { status: 200, reason: "OK", body }
    }

    fn error(status: u16, reason: &'static str, message: &str, stylesheet_href: &str) -> Self {
        Self {
            status,
            reason,
            body: error_page(message, stylesheet_href),
        }
    }

    fn render(&self) -> String {
        format!(
            "Status: {} {}\nContent-Type: text/html;charset=UTF-8\n\n{}",
            self.status, self.reason, self.body
        )
    }
}

fn render_preview(
    path_info: &str,
    query_string: &str,
    stylesheet_href: &str,
) -> Result<String, PreviewError> {
    let doc_id = parse_document_path(path_info).ok_or(PreviewError::UnsupportedPath)?;
    let config = Config::load()?;

    let markdown = fetch_content(&config, &format!("doc/{doc_id}/text"), query_string)?;
    let title = fetch_title(&config, &doc_id)
        .or_else(|| markdown_heading(&markdown))
        .unwrap_or_else(|| doc_id.clone());
    let canonical_url = canonical_url(&doc_id);

    Ok(document_page(
        &title,
        canonical_url.as_deref(),
        &render_markdown(&markdown),
        stylesheet_href,
    ))
}

/// The stylesheet lives at the previews root, while the page is served under the
/// resource path (`…/plugin-previews/<id>/doc/<DOC_ID>/text`), so the relative
/// href must climb back up one level per path segment above the first.
fn stylesheet_href(path_info: &str) -> String {
    let segments = path_info.trim_matches('/').split('/').filter(|s| !s.is_empty()).count();
    format!("{}preview.css", "../".repeat(segments.saturating_sub(1)))
}

/// Accepts `/doc/<DOC_ID>` and `/doc/<DOC_ID>/text`; anything else has no preview.
fn parse_document_path(path_info: &str) -> Option<String> {
    let mut segments = path_info.trim_matches('/').split('/');
    if segments.next() != Some("doc") {
        return None;
    }
    let doc_id = segments.next().filter(|segment| !segment.is_empty())?;
    match segments.next() {
        None | Some("text") => {}
        Some(_) => return None,
    }
    if segments.next().is_some() {
        return None;
    }
    Some(doc_id.to_string())
}

fn fetch_content(
    config: &Config,
    resource_path: &str,
    query_string: &str,
) -> Result<String, CliError> {
    let mut url = format!("{}{}/{resource_path}", config.base_url, crate::core::http::API_PREFIX);
    if !query_string.is_empty() {
        url.push('?');
        url.push_str(query_string);
    }
    let curl_args = vec![
        "-sS".to_string(),
        "--fail-with-body".to_string(),
        "-H".to_string(),
        config.auth_header.clone(),
        url,
    ];
    let output = execute_curl(&curl_args)?;
    let body = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        return Err(CliError::RequestFailed {
            code: output.status.code().unwrap_or(-1),
        });
    }
    let (content, _sources) = split_envelope_str(&body);
    Ok(content)
}

fn fetch_title(config: &Config, doc_id: &str) -> Option<String> {
    let meta = fetch_content(config, &format!("doc/{doc_id}/meta"), "").ok()?;
    let value: Value = serde_json::from_str(&meta).ok()?;
    first_title(&value)
}

/// Depth-first search for the first non-empty `title` field in the metadata
/// response (`DocumentMetadataResponse` nests per-source DTOs that all carry
/// `title`).
fn first_title(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(title)) = map.get("title") {
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            map.values().find_map(first_title)
        }
        Value::Array(items) => items.iter().find_map(first_title),
        _ => None,
    }
}

fn markdown_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let heading = line.trim_start().strip_prefix('#')?;
        let heading = heading.trim_start_matches('#').trim();
        (!heading.is_empty()).then(|| heading.to_string())
    })
}

fn canonical_url(doc_id: &str) -> Option<String> {
    let base = read_env(WEB_BASE_URL_ENV)?;
    Some(format!(
        "{}/document/{doc_id}/text",
        base.trim_end_matches('/')
    ))
}

fn render_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, options);
    let mut rendered = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut rendered, parser);
    rendered
}

fn document_page(
    title: &str,
    canonical_url: Option<&str>,
    content_html: &str,
    stylesheet_href: &str,
) -> String {
    let canonical_link = canonical_url
        .map(|href| format!("\n    <link rel=\"canonical\" href=\"{}\">", escape_html(href)))
        .unwrap_or_default();
    format!(
        "<!doctype html>\n<html lang=\"cs\">\n  <head>\n    <meta charset=\"utf-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n    <title>{}</title>{}\n    <link rel=\"stylesheet\" href=\"{}\">\n  </head>\n  <body>\n    <article class=\"document\">\n{}\n    </article>\n  </body>\n</html>\n",
        escape_html(title),
        canonical_link,
        escape_html(stylesheet_href),
        content_html
    )
}

fn error_page(message: &str, stylesheet_href: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"cs\">\n  <head>\n    <meta charset=\"utf-8\">\n    <title>Náhled není k dispozici</title>\n    <link rel=\"stylesheet\" href=\"{}\">\n  </head>\n  <body>\n    <article class=\"document document-error\">\n      <p>{}</p>\n    </article>\n  </body>\n</html>\n",
        escape_html(stylesheet_href),
        escape_html(message)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_document_paths_with_and_without_text_suffix() {
        assert_eq!(
            parse_document_path("/doc/CR10_2026_04_01/text"),
            Some("CR10_2026_04_01".to_string())
        );
        assert_eq!(parse_document_path("doc/JD1"), Some("JD1".to_string()));
    }

    #[test]
    fn rejects_paths_outside_the_doc_namespace() {
        assert_eq!(parse_document_path("/"), None);
        assert_eq!(parse_document_path("/doc"), None);
        assert_eq!(parse_document_path("/doc//text"), None);
        assert_eq!(parse_document_path("/doc/CR10/toc"), None);
        assert_eq!(parse_document_path("/doc/CR10/text/extra"), None);
        assert_eq!(parse_document_path("/search/CR"), None);
    }

    #[test]
    fn finds_the_first_title_in_nested_metadata() {
        let meta: Value = serde_json::from_str(
            r#"{"docId":"CR10","source":"CR","cr":{"main":{"docId":"CR10","title":"Zákon č. 586/1992 Sb."}}}"#,
        )
        .unwrap();
        assert_eq!(first_title(&meta), Some("Zákon č. 586/1992 Sb.".to_string()));
    }

    #[test]
    fn falls_back_to_the_first_markdown_heading() {
        assert_eq!(
            markdown_heading("intro\n## 586/1992 Sb.\ntext"),
            Some("586/1992 Sb.".to_string())
        );
        assert_eq!(markdown_heading("no heading"), None);
    }

    #[test]
    fn renders_a_full_document_page_with_title_canonical_and_stylesheet() {
        let page = document_page(
            "Zákon č. 586/1992 Sb.",
            Some("https://app.codexis.cz/document/CR10/text"),
            "<h1>586/1992 Sb.</h1>",
            "../../preview.css",
        );

        assert!(page.contains("<title>Zákon č. 586/1992 Sb.</title>"));
        assert!(page.contains(
            "<link rel=\"canonical\" href=\"https://app.codexis.cz/document/CR10/text\">"
        ));
        assert!(page.contains("<link rel=\"stylesheet\" href=\"../../preview.css\">"));
        assert!(page.contains("<h1>586/1992 Sb.</h1>"));
    }

    #[test]
    fn omits_the_canonical_link_without_a_web_base_url() {
        let page = document_page("Doc", None, "<p>x</p>", "preview.css");
        assert!(!page.contains("rel=\"canonical\""));
    }

    #[test]
    fn escapes_html_in_titles() {
        let page = document_page("A <b>&\"", None, "<p>x</p>", "preview.css");
        assert!(page.contains("<title>A &lt;b&gt;&amp;&quot;</title>"));
    }

    #[test]
    fn stylesheet_href_climbs_one_level_per_extra_path_segment() {
        assert_eq!(stylesheet_href("/doc/CR10_2026_04_01/text"), "../../preview.css");
        assert_eq!(stylesheet_href("/doc/CR10"), "../preview.css");
        assert_eq!(stylesheet_href("/doc"), "preview.css");
        assert_eq!(stylesheet_href("/"), "preview.css");
    }

    #[test]
    fn renders_markdown_tables_and_strikethrough() {
        let rendered = render_markdown("| a | b |\n|---|---|\n| 1 | 2 |\n\n~~gone~~");
        assert!(rendered.contains("<table>"));
        assert!(rendered.contains("<del>gone</del>"));
    }

    #[test]
    fn cgi_response_renders_status_and_content_type_headers() {
        let response = CgiResponse::error(404, "Not Found", "chybí", "preview.css");
        let rendered = response.render();
        assert!(rendered.starts_with("Status: 404 Not Found\nContent-Type: text/html;charset=UTF-8\n\n"));
        assert!(rendered.contains("chybí"));
    }
}
