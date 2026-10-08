//! Test-only helpers shared across command modules.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

pub fn stub_daemon(
    status_line: &'static str,
    body: &'static str,
) -> (String, mpsc::Receiver<String>) {
    stub_daemon_responses(vec![(status_line, body.to_string())])
}

pub fn stub_daemon_responses(
    responses: Vec<(&'static str, String)>,
) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for (status_line, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            sender.send(read_request(&mut stream)).unwrap();
            let response = format!(
            "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
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
