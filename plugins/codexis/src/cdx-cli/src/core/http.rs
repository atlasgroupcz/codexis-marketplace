use std::process::{Command, Stdio};

use crate::core::error::CliError;

const API_PREFIX: &str = "/rest/cdx-api";

pub(crate) fn execute_search_request(
    base_url: &str,
    auth_header: &str,
    source_code: &str,
    payload: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    let curl_args = build_search_curl_args(base_url, auth_header, source_code, payload);

    if dry_run {
        println!("{}", format_command("curl", &redact_curl_args(&curl_args)));
        return Ok(());
    }

    let status = Command::new("curl")
        .args(&curl_args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(CliError::CurlSpawn)?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(CliError::CommandExited {
            command: "curl",
            code,
        }),
        None => Err(CliError::CommandTerminated { command: "curl" }),
    }
}

fn build_search_curl_args(
    base_url: &str,
    auth_header: &str,
    source_code: &str,
    payload: &str,
) -> Vec<String> {
    vec![
        "-sS".to_string(),
        "-H".to_string(),
        auth_header.to_string(),
        "-X".to_string(),
        "POST".to_string(),
        "-H".to_string(),
        "Content-Type: application/json".to_string(),
        build_api_url(base_url, &format!("search/{source_code}")),
        "-d".to_string(),
        payload.to_string(),
    ]
}

fn build_api_url(base_url: &str, rest: &str) -> String {
    let path = rest.trim_start_matches('/');
    if path.is_empty() {
        format!("{base_url}{API_PREFIX}")
    } else {
        format!("{base_url}{API_PREFIX}/{path}")
    }
}

fn format_command(program: &str, args: &[String]) -> String {
    let mut rendered = Vec::with_capacity(args.len() + 1);
    rendered.push(shell_escape(program));
    rendered.extend(args.iter().map(|arg| shell_escape(arg)));
    rendered.join(" ")
}

fn redact_curl_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            if is_authorization_header(arg) {
                redact_authorization_header(arg)
            } else {
                arg.clone()
            }
        })
        .collect()
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    if value.chars().all(is_shell_safe_char) {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn is_shell_safe_char(ch: char) -> bool {
    matches!(
        ch,
        'a'..='z'
            | 'A'..='Z'
            | '0'..='9'
            | '-'
            | '_'
            | '.'
            | '/'
            | ':'
            | '='
            | '@'
            | '%'
    )
}

fn is_authorization_header(value: &str) -> bool {
    value
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("authorization:")
}

fn redact_authorization_header(header: &str) -> String {
    let value = header
        .split_once(':')
        .map(|(_, value)| value.trim())
        .unwrap_or_default();

    if value.to_ascii_lowercase().starts_with("bearer ") {
        "Authorization: Bearer <redacted>".to_string()
    } else if let Some((scheme, _)) = value.split_once(' ') {
        format!("Authorization: {scheme} <redacted>")
    } else {
        "Authorization: <redacted>".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_curl_args_are_built_as_post_json_request() {
        let args = build_search_curl_args(
            "https://app.codexis.cz",
            "Authorization: Bearer token",
            "JD",
            r#"{"query":"náhrada škody","limit":5}"#,
        );

        assert_eq!(
            args,
            vec![
                "-sS",
                "-H",
                "Authorization: Bearer token",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "https://app.codexis.cz/rest/cdx-api/search/JD",
                "-d",
                r#"{"query":"náhrada škody","limit":5}"#,
            ]
        );
    }

    #[test]
    fn shell_escape_quotes_json_for_dry_run_output() {
        let rendered = format_command(
            "curl",
            &[
                "-d".to_string(),
                r#"{"query":"náhrada škody","limit":5}"#.to_string(),
            ],
        );

        assert_eq!(
            rendered,
            "curl -d '{\"query\":\"náhrada škody\",\"limit\":5}'"
        );
    }

    #[test]
    fn dry_run_output_redacts_authorization_header() {
        let rendered = format_command(
            "curl",
            &redact_curl_args(&[
                "-H".to_string(),
                "Authorization: Bearer super-secret-token".to_string(),
                "-d".to_string(),
                r#"{"query":"test"}"#.to_string(),
            ]),
        );

        assert_eq!(
            rendered,
            "curl -H 'Authorization: Bearer <redacted>' -d '{\"query\":\"test\"}'"
        );
    }
}
