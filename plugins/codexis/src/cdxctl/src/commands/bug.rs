use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

const REPORT_PATH: &str = "/rest/v1/plugin/bug-report";
const SEVERITIES: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];

/// File a bug report. Append-only and fire and forget: one POST, print what the
/// daemon created, no follow-up read and no way to edit or delete it afterwards.
pub fn report(
    client: &GraphQLClient,
    title: &str,
    description: &str,
    severity: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let payload = build_payload(title, description, severity)?;
    let created = client.rest_post_json(REPORT_PATH, &payload)?;
    print_output(&created, format);
    Ok(())
}

/// Severity is normalized here so `--severity high` and `--severity HIGH` produce
/// the same request, and a typo fails locally instead of as an opaque HTTP 400.
fn build_payload(title: &str, description: &str, severity: &str) -> Result<Value, CdxctlError> {
    Ok(json!({
        "title": title,
        "description": description,
        "severity": normalize_severity(severity)?,
    }))
}

fn normalize_severity(raw: &str) -> Result<String, CdxctlError> {
    let upper = raw.trim().to_uppercase();
    if SEVERITIES.contains(&upper.as_str()) {
        return Ok(upper);
    }
    Err(CdxctlError::Parse(format!(
        "unknown severity '{raw}', expected one of {}",
        SEVERITIES.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn builds_the_payload_the_daemon_expects() {
        let payload = build_payload("Chat hangs", "Steps: open chat, wait", "HIGH").unwrap();
        assert_eq!(payload["title"], json!("Chat hangs"));
        assert_eq!(payload["description"], json!("Steps: open chat, wait"));
        assert_eq!(payload["severity"], json!("HIGH"));
        assert_eq!(payload.as_object().unwrap().len(), 3);
    }

    #[test]
    fn accepts_a_lowercase_severity() {
        let payload = build_payload("t", "d", " critical ").unwrap();
        assert_eq!(payload["severity"], json!("CRITICAL"));
    }

    #[test]
    fn rejects_an_unknown_severity() {
        let error = build_payload("t", "d", "urgent").unwrap_err();
        match error {
            CdxctlError::Parse(message) => {
                assert!(message.contains("urgent"));
                assert!(message.contains("CRITICAL"));
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn posts_the_report_to_the_daemon_bug_report_endpoint() {
        let (address, requests) = stub_daemon("HTTP/1.1 201 Created", "{\"id\":\"bug-1\"}");
        let client = GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".to_string(),
        );

        report(&client, "Chat hangs", "Steps: open chat", "high", OutputFormat::Json).unwrap();

        let request = requests.recv().unwrap();
        assert!(request.starts_with("POST /rest/v1/plugin/bug-report HTTP/1.1"));
        assert!(request.to_lowercase().contains("authorization: bearer test-token"));
        assert!(request.to_lowercase().contains("content-type: application/json"));
        let body = request.split("\r\n\r\n").nth(1).unwrap();
        let parsed: Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["severity"], json!("HIGH"));
        assert_eq!(parsed["title"], json!("Chat hangs"));
    }

    #[test]
    fn surfaces_a_daemon_rejection_as_a_network_error() {
        let (address, _requests) = stub_daemon("HTTP/1.1 400 Bad Request", "{\"code\":\"BAD_REQUEST\"}");
        let client = GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".to_string(),
        );

        let error = report(&client, "t", "d", "LOW", OutputFormat::Json).unwrap_err();
        match error {
            CdxctlError::Network(message) => assert!(message.contains("400")),
            other => panic!("expected a network error, got {other:?}"),
        }
    }

    fn stub_daemon(status_line: &'static str, body: &'static str) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            sender.send(read_request(&mut stream)).unwrap();
            let response = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });
        (address, receiver)
    }

    /// Read until the headers and the whole declared body have arrived. A single
    /// `read()` can return the headers without the body when TCP splits them,
    /// which made the assertions on the payload intermittently fail.
    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut raw = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&raw).to_string();
            let Some(header_end) = text.find("\r\n\r\n") else {
                continue;
            };
            let expected = content_length(&text[..header_end]);
            if raw.len() >= header_end + 4 + expected {
                return text;
            }
        }
        String::from_utf8_lossy(&raw).to_string()
    }

    fn content_length(headers: &str) -> usize {
        headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())?
            })
            .unwrap_or(0)
    }
}
