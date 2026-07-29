//! `cdx-cli preview` - CGI handler behind the daemon's plugin preview endpoint
//! (`/rest/v1/plugin-previews/…`, LM-207). The daemon passes the resource path of
//! a `cdx://preview/doc/<DOC_ID>[/<DOC_PART_ID>]` preview reference via
//! `PATH_INFO`; the handler downloads the document's native preview content from
//! the CODEXIS API into the directory the daemon names in `PREVIEW_DIR` and emits
//! a JSON manifest the frontend feeds to its file preview renderer.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::core::config::{CODEXIS_PLUGIN_API_URL_ENV, CODEXIS_USER_API_TOKEN_ENV};
use crate::core::error::CliError;
use cdx_common::config::{read_env, to_authorization_header};
use cdx_common::curl::{execute_curl, split_envelope_str};

const WEB_BASE_URL_ENV: &str = "CODEXIS_PLUGIN_BASE_URL";
const PREVIEW_DIR_ENV: &str = "PREVIEW_DIR";
const PREVIEW_ASSET_BASE_URL_ENV: &str = "PREVIEW_ASSET_BASE_URL";

pub(crate) fn execute_preview() -> Result<(), CliError> {
    let path_info = env::var("PATH_INFO").unwrap_or_default();
    let response = match render_preview(&path_info) {
        Ok(manifest) => CgiResponse::ok(manifest.to_string()),
        Err(PreviewError::UnsupportedPath) => {
            CgiResponse::error(404, "Not Found", "Náhled pro tento zdroj není k dispozici.")
        }
        Err(PreviewError::Cli(error)) => CgiResponse::error(
            502,
            "Bad Gateway",
            &format!("Náhled dokumentu se nepodařilo načíst: {error}"),
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
    content_type: &'static str,
    cache_control: &'static str,
    body: String,
}

impl CgiResponse {
    fn ok(body: String) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "application/json;charset=UTF-8",
            // Rendering re-executes the CLI on every request, so let the browser
            // reuse a successful manifest; `private` keeps it out of shared caches.
            cache_control: "private, max-age=3600",
            body,
        }
    }

    fn error(status: u16, reason: &'static str, message: &str) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain;charset=UTF-8",
            cache_control: "no-store",
            body: message.to_string(),
        }
    }

    fn render(&self) -> String {
        format!(
            "Status: {} {}\nContent-Type: {}\nCache-Control: {}\n\n{}",
            self.status, self.reason, self.content_type, self.cache_control, self.body
        )
    }
}

fn render_preview(path_info: &str) -> Result<Value, PreviewError> {
    let document = parse_document_path(path_info).ok_or(PreviewError::UnsupportedPath)?;
    let config = PreviewConfig::load()?;
    let previews_dir = previews_dir()?;
    fs::create_dir_all(&previews_dir)
        .map_err(|source| PreviewError::Cli(CliError::Io { context: "create previews dir".to_string(), source }))?;

    let title = fetch_title(&config, &document.doc_id).unwrap_or_else(|| document.doc_id.clone());
    let file = write_preview_file(&config, &document, &previews_dir, &title, &asset_base_url()?)?;
    let size = fs::metadata(&file.path).map(|m| m.len()).unwrap_or(0);

    Ok(json!({
        "path": file.path.to_string_lossy(),
        "mimeType": file.mime_type,
        "name": file.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        "size": size,
        "title": title,
        "anchor": document.part_id.clone(),
        "externalUrl": canonical_url(&document),
    }))
}

struct PreviewFile {
    path: PathBuf,
    mime_type: &'static str,
}

/// Downloads the document's native preview content and stores it as a file the
/// frontend renderer understands: HTML documents become a standalone `.html`
/// page (stylesheet served by the daemon out of the plugin), binary documents keep
/// their native format (`.pdf`), and backends without a preview fall back to
/// the markdown text render stored as `.md`.
fn write_preview_file(
    config: &PreviewConfig,
    document: &DocumentPreviewPath,
    previews_dir: &Path,
    title: &str,
    asset_base_url: &str,
) -> Result<PreviewFile, PreviewError> {
    let base_name = file_base_name(document);
    let download_path = previews_dir.join(format!("{base_name}.download"));
    let downloaded = download_preview(config, document, &download_path);

    match downloaded.as_deref() {
        Some(content_type) if content_type.starts_with("text/html") => {
            let content = fs::read_to_string(&download_path)
                .map_err(|source| PreviewError::Cli(CliError::Io { context: "read downloaded preview".to_string(), source }))?;
            let _ = fs::remove_file(&download_path);
            let page = document_page(title, &content, &stylesheet_href(asset_base_url));
            let path = previews_dir.join(format!("{base_name}.html"));
            fs::write(&path, page)
                .map_err(|source| PreviewError::Cli(CliError::Io { context: "write html preview".to_string(), source }))?;
            Ok(PreviewFile { path, mime_type: "text/html" })
        }
        Some(content_type) if content_type.starts_with("application/pdf") => {
            let path = previews_dir.join(format!("{base_name}.pdf"));
            fs::rename(&download_path, &path)
                .map_err(|source| PreviewError::Cli(CliError::Io { context: "store pdf preview".to_string(), source }))?;
            Ok(PreviewFile { path, mime_type: "application/pdf" })
        }
        _ => {
            let _ = fs::remove_file(&download_path);
            let markdown = fetch_content(
                config,
                &format!("doc/{}/text", document.doc_id),
                &text_query(document),
            )?;
            let path = previews_dir.join(format!("{base_name}.md"));
            fs::write(&path, markdown)
                .map_err(|source| PreviewError::Cli(CliError::Io { context: "write markdown preview".to_string(), source }))?;
            Ok(PreviewFile { path, mime_type: "text/markdown" })
        }
    }
}

/// Downloads the preview endpoint straight to disk and reports the response
/// content type; `None` when the endpoint is unavailable or errors (fallback).
fn download_preview(
    config: &PreviewConfig,
    document: &DocumentPreviewPath,
    target: &Path,
) -> Option<String> {
    // Always download the whole law; the referenced part is surfaced as a scroll
    // anchor (see render_preview) so the preview shows the full document and jumps
    // to the paragraph instead of rendering it in isolation.
    let resource_path = format!("doc/{}/preview", document.doc_id);
    let url = format!("{}{}/{resource_path}", config.base_url, crate::core::http::API_PREFIX);
    let mut curl_args = vec![
        "-sS".to_string(),
        "-o".to_string(),
        target.to_string_lossy().into_owned(),
        "-w".to_string(),
        "%{content_type}|%{http_code}".to_string(),
    ];
    if let Some(header) = config.api_auth_header() {
        curl_args.push("-H".to_string());
        curl_args.push(header);
    }
    curl_args.push(url);
    let output = execute_curl(&curl_args).ok()?;
    if !output.status.success() {
        return None;
    }
    let written = String::from_utf8_lossy(&output.stdout);
    let (content_type, http_code) = written.trim().rsplit_once('|')?;
    if http_code != "200" {
        return None;
    }
    Some(content_type.to_string())
}

/// The daemon owns the filesystem layout and names the target directory in
/// `PREVIEW_DIR`; the plugin never derives a path of its own.
fn previews_dir() -> Result<PathBuf, PreviewError> {
    let dir = read_env(PREVIEW_DIR_ENV)
        .ok_or(PreviewError::Cli(CliError::MissingConfig(PREVIEW_DIR_ENV)))?;
    Ok(PathBuf::from(dir))
}

fn asset_base_url() -> Result<String, PreviewError> {
    read_env(PREVIEW_ASSET_BASE_URL_ENV)
        .ok_or(PreviewError::Cli(CliError::MissingConfig(PREVIEW_ASSET_BASE_URL_ENV)))
}

/// The generated HTML links the stylesheet straight out of the plugin's `previews/`
/// directory, which the daemon serves as a static asset under the preview endpoint.
fn stylesheet_href(asset_base_url: &str) -> String {
    format!("{}/preview.css", asset_base_url.trim_end_matches('/'))
}

fn file_base_name(document: &DocumentPreviewPath) -> String {
    let raw = match &document.part_id {
        Some(part) => format!("{}-{part}", document.doc_id),
        None => document.doc_id.clone(),
    };
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Preview needs the API base; the user token is optional so a missing token
/// degrades to an error page instead of failing configuration outright.
struct PreviewConfig {
    base_url: String,
    user_auth_header: Option<String>,
}

impl PreviewConfig {
    fn load() -> Result<Self, CliError> {
        let base_url = read_env(CODEXIS_PLUGIN_API_URL_ENV)
            .ok_or_else(|| CliError::MissingConfigVars(vec![CODEXIS_PLUGIN_API_URL_ENV]))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            user_auth_header: read_env(CODEXIS_USER_API_TOKEN_ENV)
                .map(|token| to_authorization_header(&token)),
        })
    }

    /// Authorization for cdx-api requests - the user's Bearer token.
    fn api_auth_header(&self) -> Option<String> {
        self.user_auth_header.clone()
    }
}

struct DocumentPreviewPath {
    doc_id: String,
    part_id: Option<String>,
}

/// Accepts `/preview/doc/<DOC_ID>` and `/preview/doc/<DOC_ID>/<DOC_PART_ID>`;
/// anything else has no preview.
fn parse_document_path(path_info: &str) -> Option<DocumentPreviewPath> {
    let mut segments = path_info.trim_matches('/').split('/');
    if segments.next() != Some("preview") || segments.next() != Some("doc") {
        return None;
    }
    let doc_id = segments.next().filter(|segment| !segment.is_empty())?;
    let part_id = segments.next().filter(|segment| !segment.is_empty());
    if segments.next().is_some() {
        return None;
    }
    Some(DocumentPreviewPath {
        doc_id: doc_id.to_string(),
        part_id: part_id.map(str::to_string),
    })
}

fn text_query(document: &DocumentPreviewPath) -> String {
    match &document.part_id {
        Some(part) => format!("part={part}"),
        None => String::new(),
    }
}

fn fetch_content(
    config: &PreviewConfig,
    resource_path: &str,
    query_string: &str,
) -> Result<String, CliError> {
    let mut url = format!("{}{}/{resource_path}", config.base_url, crate::core::http::API_PREFIX);
    if !query_string.is_empty() {
        url.push('?');
        url.push_str(query_string);
    }
    let mut curl_args = vec!["-sS".to_string(), "--fail-with-body".to_string()];
    if let Some(header) = config.api_auth_header() {
        curl_args.push("-H".to_string());
        curl_args.push(header);
    }
    curl_args.push(url);
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

fn fetch_title(config: &PreviewConfig, doc_id: &str) -> Option<String> {
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

fn canonical_url(document: &DocumentPreviewPath) -> Option<String> {
    let base = read_env(WEB_BASE_URL_ENV)?;
    Some(canonical_url_for_base(&base, document))
}

/// The web app's document route is `/doc/<DOC_ID>`, with the referenced part as a
/// URL fragment (`#paragrafN`) so "open in new tab" lands on the whole law scrolled
/// to the paragraph - matching the backend `CdxSourceUrlBuilder.forDocIdAndPart`.
fn canonical_url_for_base(base: &str, document: &DocumentPreviewPath) -> String {
    let fragment = document
        .part_id
        .as_ref()
        .map(|part| format!("#{part}"))
        .unwrap_or_default();
    format!("{}/doc/{}{fragment}", base.trim_end_matches('/'), document.doc_id)
}

fn document_page(title: &str, content_html: &str, stylesheet_href: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"cs\">\n  <head>\n    <meta charset=\"utf-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n    <title>{}</title>\n    <link rel=\"stylesheet\" href=\"{}\">\n  </head>\n  <body>\n    <article class=\"document\">\n{}\n    </article>\n  </body>\n</html>\n",
        escape_html(title),
        escape_html(stylesheet_href),
        content_html
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
    fn parses_document_paths_with_and_without_a_part() {
        let plain = parse_document_path("/preview/doc/CR10_2026_04_01").unwrap();
        assert_eq!(plain.doc_id, "CR10_2026_04_01");
        assert_eq!(plain.part_id, None);

        let with_part = parse_document_path("preview/doc/CR10_2026_04_01/paragraf10").unwrap();
        assert_eq!(with_part.doc_id, "CR10_2026_04_01");
        assert_eq!(with_part.part_id.as_deref(), Some("paragraf10"));
    }

    #[test]
    fn rejects_paths_outside_the_preview_doc_namespace() {
        assert!(parse_document_path("/").is_none());
        assert!(parse_document_path("/preview").is_none());
        assert!(parse_document_path("/preview/doc").is_none());
        assert!(parse_document_path("/doc/CR10").is_none());
        assert!(parse_document_path("/preview/doc/CR10/part/extra").is_none());
        assert!(parse_document_path("/preview/search/CR").is_none());
    }

    #[test]
    fn builds_the_text_query_from_the_part() {
        let with_part = DocumentPreviewPath {
            doc_id: "CR10".to_string(),
            part_id: Some("paragraf10".to_string()),
        };
        let without_part = DocumentPreviewPath { doc_id: "CR10".to_string(), part_id: None };

        assert_eq!(text_query(&with_part), "part=paragraf10");
        assert_eq!(text_query(&without_part), "");
    }

    #[test]
    fn canonical_url_points_at_the_doc_route_with_the_part_as_a_fragment() {
        let with_part = DocumentPreviewPath {
            doc_id: "CR10_2026_04_01".to_string(),
            part_id: Some("paragraf10".to_string()),
        };
        let without_part = DocumentPreviewPath {
            doc_id: "CR10_2026_04_01".to_string(),
            part_id: None,
        };

        assert_eq!(
            canonical_url_for_base("https://next.codexis.cz/", &with_part),
            "https://next.codexis.cz/doc/CR10_2026_04_01#paragraf10"
        );
        assert_eq!(
            canonical_url_for_base("https://next.codexis.cz", &without_part),
            "https://next.codexis.cz/doc/CR10_2026_04_01"
        );
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
    fn derives_safe_file_names_from_document_paths() {
        let with_part = DocumentPreviewPath {
            doc_id: "CR10_2026_04_01".to_string(),
            part_id: Some("paragraf10".to_string()),
        };
        let odd = DocumentPreviewPath { doc_id: "CR10/../x".to_string(), part_id: None };

        assert_eq!(file_base_name(&with_part), "CR10_2026_04_01-paragraf10");
        assert_eq!(file_base_name(&odd), "CR10____x");
    }

    #[test]
    fn links_the_stylesheet_under_the_preview_asset_base_url() {
        assert_eq!(
            stylesheet_href("/rest/v1/plugin-previews/UGx1Z2lu"),
            "/rest/v1/plugin-previews/UGx1Z2lu/preview.css"
        );
        assert_eq!(
            stylesheet_href("/rest/v1/plugin-previews/UGx1Z2lu/"),
            "/rest/v1/plugin-previews/UGx1Z2lu/preview.css"
        );
    }

    #[test]
    fn renders_a_full_document_page_with_title_and_stylesheet() {
        let page = document_page(
            "Zákon č. 586/1992 Sb.",
            "<h1>586/1992 Sb.</h1>",
            "/rest/v1/plugin-previews/UGx1Z2lu/preview.css",
        );

        assert!(page.contains("<title>Zákon č. 586/1992 Sb.</title>"));
        assert!(page.contains(
            "<link rel=\"stylesheet\" href=\"/rest/v1/plugin-previews/UGx1Z2lu/preview.css\">"
        ));
        assert!(page.contains("<h1>586/1992 Sb.</h1>"));
    }

    #[test]
    fn escapes_html_in_titles() {
        let page = document_page("A <b>&\"", "<p>x</p>", "preview.css");
        assert!(page.contains("<title>A &lt;b&gt;&amp;&quot;</title>"));
    }

    #[test]
    fn cgi_response_renders_status_and_content_type_headers() {
        let response = CgiResponse::error(404, "Not Found", "chybí");
        let rendered = response.render();
        assert!(rendered.starts_with(
            "Status: 404 Not Found\nContent-Type: text/plain;charset=UTF-8\nCache-Control: no-store\n\n"
        ));
        assert!(rendered.contains("chybí"));
    }

    #[test]
    fn successful_manifests_are_browser_cacheable_json() {
        let rendered = CgiResponse::ok("{}".to_string()).render();
        assert!(rendered.contains("\nContent-Type: application/json;charset=UTF-8\n"));
        assert!(rendered.contains("\nCache-Control: private, max-age=3600\n"));
    }
}
