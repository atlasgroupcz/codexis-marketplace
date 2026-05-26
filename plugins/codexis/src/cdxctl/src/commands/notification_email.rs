use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::output::{print_output, OutputFormat};
use reqwest::blocking::multipart::{Form, Part};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::Path;

const CHANNELS_PATH: &str = "/rest/v1/channels/email";
const SEND_PATH: &str = "/rest/v1/channels/email/send";

#[derive(Serialize)]
struct SendPayload<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "channelId")]
    channel_id: Option<&'a str>,
    to: &'a [String],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<&'a str>,
    subject: &'a str,
    #[serde(rename = "bodyText")]
    body_text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bodyHtml")]
    body_html: Option<&'a str>,
}

pub fn channels_list(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.rest_get(CHANNELS_PATH)?;
    print_output(&data, format);
    Ok(())
}

pub fn channels_test(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let path = format!("{CHANNELS_PATH}/{id}/test");
    let data = client.rest_post_empty(&path)?;
    print_output(&data, format);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn send(
    client: &GraphQLClient,
    to: &[String],
    cc: &[String],
    bcc: &[String],
    subject: &str,
    body: Option<&str>,
    body_file: Option<&str>,
    body_html: Option<&str>,
    channel_id: Option<&str>,
    attachments: &[String],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    if to.is_empty() {
        return Err(CdxctlError::Parse(
            "at least one --to recipient is required".into(),
        ));
    }
    if body.is_some() && body_file.is_some() {
        return Err(CdxctlError::Parse(
            "use either --body or --body-file, not both".into(),
        ));
    }
    let body_owned = resolve_body(body, body_file)?;
    let cc_refs: Vec<&str> = cc.iter().map(String::as_str).collect();
    let bcc_refs: Vec<&str> = bcc.iter().map(String::as_str).collect();

    let payload = SendPayload {
        channel_id,
        to,
        cc: cc_refs,
        bcc: bcc_refs,
        subject,
        body_text: &body_owned,
        body_html,
    };
    let payload_json = serde_json::to_string(&payload)?;
    let payload_part = Part::text(payload_json)
        .mime_str("application/json")
        .map_err(|e| CdxctlError::Network(e.to_string()))?;
    let mut form = Form::new().part("payload", payload_part);
    for attachment_path in attachments {
        form = form.part("attachments", attachment_part(attachment_path)?);
    }
    let data = client.rest_post_multipart(SEND_PATH, form)?;
    print_output(&data, format);
    Ok(())
}

fn attachment_part(path_str: &str) -> Result<Part, CdxctlError> {
    let path = Path::new(path_str);
    let mut file = fs::File::open(path)
        .map_err(|e| CdxctlError::Parse(format!("Failed to open attachment {path_str}: {e}")))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment")
        .to_string();
    let part = Part::bytes(buffer).file_name(filename);
    Ok(part)
}

fn resolve_body(body: Option<&str>, body_file: Option<&str>) -> Result<String, CdxctlError> {
    if let Some(text) = body {
        return Ok(text.to_string());
    }
    if let Some(path) = body_file {
        if path == "-" {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            return Ok(buf);
        }
        return fs::read_to_string(path).map_err(CdxctlError::from);
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_body_uses_inline_body_when_provided() {
        let body = resolve_body(Some("inline body"), None).expect("inline body resolves");
        assert_eq!(body, "inline body");
    }

    #[test]
    fn resolve_body_returns_empty_string_when_neither_source_provided() {
        let body = resolve_body(None, None).expect("missing body is allowed at this layer");
        assert_eq!(body, "");
    }

    #[test]
    fn serializes_payload_without_optional_fields_when_unset() {
        let payload = SendPayload {
            channel_id: None,
            to: &vec!["a@b".to_string()],
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: "hi",
            body_text: "body",
            body_html: None,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(!json.contains("channelId"), "channelId should be omitted when None");
        assert!(!json.contains("cc"), "empty cc list should be omitted");
        assert!(!json.contains("bcc"), "empty bcc list should be omitted");
        assert!(!json.contains("bodyHtml"), "bodyHtml should be omitted when None");
        assert!(json.contains("\"to\":[\"a@b\"]"));
        assert!(json.contains("\"bodyText\":\"body\""));
    }
}
