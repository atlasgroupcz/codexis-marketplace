//! A stub Datify on a real socket, for this crate's own tests.
//!
//! Both test tiers need one: the unit tests exercise `DatifyClient` in-process,
//! the integration tests run the binaries as processes. Behind a feature so the
//! integration tests can reach it too - the same shape `cdx_common::test_support`
//! uses - which keeps the request parser in one place rather than one per tier.

use std::io::{BufRead, BufReader, Read as _, Write as _};
use std::net::TcpListener;
use std::sync::mpsc;

/// One request as the stub received it.
pub struct Seen {
    pub method: String,
    /// Request target, i.e. path plus query string.
    pub target: String,
    /// The `Authorization` header, empty when absent.
    pub auth: String,
    pub body: String,
}

/// Serve exactly one request with the given reply, then stop.
///
/// Returns the base URL to point a client at and the channel the request
/// arrives on. Nothing is mocked: the request really crosses a socket, so the
/// method, URL, headers and body are the ones that would go to Datify.
pub fn stub(status: u16, body: &'static str) -> (String, mpsc::Receiver<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        // A test that fails before sending leaves the client gone; give up
        // rather than wedging the thread.
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        let mut reader = BufReader::new(stream);

        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            return;
        }
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_string();
        let target = parts.next().unwrap_or_default().to_string();

        let mut auth = String::new();
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() {
                return;
            }
            let header = header.trim_end();
            if header.is_empty() {
                break;
            }
            // Header names are case-insensitive on the wire, and hyper writes
            // them lowercased.
            let Some((name, value)) = header.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match name.to_ascii_lowercase().as_str() {
                "authorization" => auth = value.to_string(),
                "content-length" => content_length = value.parse().unwrap_or(0),
                _ => {}
            }
        }

        let mut payload = vec![0u8; content_length];
        if content_length > 0 && reader.read_exact(&mut payload).is_err() {
            return;
        }
        let _ = tx.send(Seen {
            method,
            target,
            auth,
            body: String::from_utf8_lossy(&payload).to_string(),
        });

        let mut stream = reader.into_inner();
        let _ = write!(
            stream,
            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
    });

    (format!("http://127.0.0.1:{port}"), rx)
}
