use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const CDX_SCHEME: &str = "cdx://";
const API_PREFIX: &str = "/rest/cdx-api";
const CODEXIS_API_URL_ENV: &str = "CODEXIS_API_URL";
const CDX_API_JWT_AUTH_ENV: &str = "CDX_API_JWT_AUTH";
const CDX_ENV_FILE_RELATIVE_PATH: &str = ".cdx/.env";
const CDX_ENV_FILE_DISPLAY_PATH: &str = "~/.cdx/.env";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return;
    }

    let env_file = load_env_file_from_home();
    let base_url = resolve_config_value(CODEXIS_API_URL_ENV, env_file.as_ref());
    let jwt_auth = resolve_config_value(CDX_API_JWT_AUTH_ENV, env_file.as_ref());

    let mut missing: Vec<String> = Vec::new();
    if base_url.is_none() {
        missing.push(CODEXIS_API_URL_ENV.to_string());
    }
    if jwt_auth.is_none() {
        missing.push(CDX_API_JWT_AUTH_ENV.to_string());
    }
    if !missing.is_empty() {
        eprintln!(
            "Missing required configuration: {}. Set them in the environment or {}.",
            missing.join(", "),
            CDX_ENV_FILE_DISPLAY_PATH
        );
        std::process::exit(2);
    }
    let base_url = base_url.unwrap().trim_end_matches('/').to_string();

    let mut resolved_args: Vec<String> = Vec::with_capacity(args.len());
    for arg in args {
        if let Some(rest) = arg.strip_prefix(CDX_SCHEME) {
            resolved_args.push(build_cdx_url(&base_url, rest));
        } else {
            resolved_args.push(arg);
        }
    }

    let mut curl_args: Vec<String> = Vec::with_capacity(resolved_args.len() + 2);
    curl_args.push("-H".to_string());
    curl_args.push(to_authorization_header(jwt_auth.as_deref().unwrap()));
    curl_args.extend(resolved_args);

    let status = Command::new("curl")
        .args(&curl_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("failed to run curl: {}", err);
            std::process::exit(1);
        }
    }
}

fn build_cdx_url(base_url: &str, rest: &str) -> String {
    let path = rest.trim_start_matches('/');
    if path.is_empty() {
        format!("{base_url}{API_PREFIX}")
    } else {
        format!("{base_url}{API_PREFIX}/{path}")
    }
}

fn load_env_file_from_home() -> Option<HashMap<String, String>> {
    let home = env::var_os("HOME")?;
    let path = PathBuf::from(home).join(CDX_ENV_FILE_RELATIVE_PATH);
    let content = fs::read_to_string(path).ok()?;
    Some(parse_env_file(&content))
}

fn parse_env_file(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(parse_env_line)
        .collect::<HashMap<_, _>>()
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let line = line.strip_prefix("export ").unwrap_or(line);
    let (key, raw_value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    Some((key.to_string(), parse_env_value(raw_value.trim())))
}

fn parse_env_value(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if first == last && (first == b'"' || first == b'\'') {
            return unescape_quoted_env_value(&value[1..value.len() - 1], first);
        }
    }
    value.to_string()
}

fn unescape_quoted_env_value(value: &str, quote: u8) -> String {
    if quote == b'\'' {
        return value.to_string();
    }

    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }
    result
}

fn resolve_config_value(name: &str, env_file: Option<&HashMap<String, String>>) -> Option<String> {
    env::var(name)
        .ok()
        .and_then(|value| normalize_config_value(value))
        .or_else(|| {
            env_file
                .and_then(|entries| entries.get(name))
                .cloned()
                .and_then(normalize_config_value)
        })
}

fn normalize_config_value(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn to_authorization_header(jwt_auth: &str) -> String {
    // Accept either:
    // - "Authorization: Bearer <jwt>" (passed through)
    // - "Bearer <jwt>" (prefixed with "Authorization: ")
    // - "<jwt>" (heuristic: treat as JWT and prepend "Authorization: Bearer ")
    // - anything else (treated as the value of Authorization header)
    if starts_with_ignore_ascii_case(jwt_auth, "authorization:") {
        return jwt_auth.trim().to_string();
    }
    if starts_with_ignore_ascii_case(jwt_auth, "bearer ") {
        return format!("Authorization: {}", jwt_auth.trim());
    }
    if looks_like_jwt(jwt_auth) {
        return format!("Authorization: Bearer {}", jwt_auth.trim());
    }
    format!("Authorization: {}", jwt_auth.trim())
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn looks_like_jwt(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.contains(' ') {
        return false;
    }
    let mut parts = value.split('.');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), Some(c), None) => !a.is_empty() && !b.is_empty() && !c.is_empty(),
        _ => false,
    }
}

fn print_usage() {
    println!("cdx [curl args] <url>");
    println!("Supports cdx:// URLs.");
    println!(
        "Requires {CODEXIS_API_URL_ENV} and {CDX_API_JWT_AUTH_ENV} in the environment or {CDX_ENV_FILE_DISPLAY_PATH}."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn authorization_header_passes_through_full_header() {
        let header = to_authorization_header("Authorization: Bearer abc");
        assert_eq!(header, "Authorization: Bearer abc");
    }

    #[test]
    fn authorization_header_wraps_bearer_value() {
        let header = to_authorization_header("Bearer abc");
        assert_eq!(header, "Authorization: Bearer abc");
    }

    #[test]
    fn authorization_header_wraps_raw_jwt() {
        let header = to_authorization_header("a.b.c");
        assert_eq!(header, "Authorization: Bearer a.b.c");
    }

    #[test]
    fn authorization_header_wraps_other_scheme() {
        let header = to_authorization_header("Basic xyz");
        assert_eq!(header, "Authorization: Basic xyz");
    }

    #[test]
    fn env_parser_supports_plain_and_exported_values() {
        let parsed = parse_env_file(
            r#"
CODEXIS_API_URL=https://app.codexis.cz/
export CDX_API_JWT_AUTH="Bearer abc"
# comment
"#,
        );

        assert_eq!(
            parsed.get(CODEXIS_API_URL_ENV),
            Some(&"https://app.codexis.cz/".to_string())
        );
        assert_eq!(
            parsed.get(CDX_API_JWT_AUTH_ENV),
            Some(&"Bearer abc".to_string())
        );
    }

    #[test]
    fn env_parser_unescapes_double_quoted_values() {
        let parsed = parse_env_file(r#"CDX_API_JWT_AUTH="Bearer \"abc\"""#);
        assert_eq!(
            parsed.get(CDX_API_JWT_AUTH_ENV),
            Some(&"Bearer \"abc\"".to_string())
        );
    }

    #[test]
    fn resolve_config_value_prefers_process_env_and_rejects_empty_values() {
        let mut env_file = HashMap::new();
        env_file.insert(
            CDX_API_JWT_AUTH_ENV.to_string(),
            "Bearer from-file".to_string(),
        );

        let value = resolve_config_value_for_test(
            Some("Bearer from-env"),
            CDX_API_JWT_AUTH_ENV,
            Some(&env_file),
        );
        assert_eq!(value, Some("Bearer from-env".to_string()));

        let value =
            resolve_config_value_for_test(Some("   "), CDX_API_JWT_AUTH_ENV, Some(&env_file));
        assert_eq!(value, Some("Bearer from-file".to_string()));
    }

    #[test]
    fn resolve_config_value_uses_env_file_when_process_env_is_missing() {
        let mut env_file = HashMap::new();
        env_file.insert(
            CODEXIS_API_URL_ENV.to_string(),
            "https://file.codexis.test".to_string(),
        );

        let value = resolve_config_value_for_test(None, CODEXIS_API_URL_ENV, Some(&env_file));
        assert_eq!(value, Some("https://file.codexis.test".to_string()));
    }

    #[test]
    fn resolve_config_value_accepts_raw_jwt_from_env_file() {
        let mut env_file = HashMap::new();
        env_file.insert(CDX_API_JWT_AUTH_ENV.to_string(), "a.b.c".to_string());

        let value = resolve_config_value_for_test(None, CDX_API_JWT_AUTH_ENV, Some(&env_file));
        assert_eq!(value, Some("a.b.c".to_string()));
        assert_eq!(
            to_authorization_header(value.as_deref().unwrap()),
            "Authorization: Bearer a.b.c"
        );
    }

    fn resolve_config_value_for_test(
        direct_value: Option<&str>,
        name: &str,
        env_file: Option<&HashMap<String, String>>,
    ) -> Option<String> {
        direct_value
            .map(str::to_string)
            .and_then(normalize_config_value)
            .or_else(|| {
                env_file
                    .and_then(|entries| entries.get(name))
                    .cloned()
                    .and_then(normalize_config_value)
            })
    }
}
