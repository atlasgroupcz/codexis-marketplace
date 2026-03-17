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

#[derive(Debug, Default, PartialEq, Eq)]
struct ArgFeatures {
    has_output_mode_override: bool,
    has_explicit_method: bool,
    has_get_override: bool,
    has_json_body: bool,
    has_content_type_header: bool,
    uses_curl_json: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RequestDefaults {
    add_silent_defaults: bool,
    add_post: bool,
    add_json_content_type: bool,
}

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

    let curl_args = build_curl_args(&base_url, jwt_auth.as_deref().unwrap(), &args);

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

fn build_curl_args(base_url: &str, jwt_auth: &str, args: &[String]) -> Vec<String> {
    let resolved_args = resolve_args(base_url, args);
    let defaults = infer_request_defaults(&resolved_args);

    let mut curl_args: Vec<String> = Vec::with_capacity(resolved_args.len() + 7);
    if defaults.add_silent_defaults {
        curl_args.push("-sS".to_string());
    }
    curl_args.push("-H".to_string());
    curl_args.push(to_authorization_header(jwt_auth));
    if defaults.add_post {
        curl_args.push("-X".to_string());
        curl_args.push("POST".to_string());
    }
    if defaults.add_json_content_type {
        curl_args.push("-H".to_string());
        curl_args.push("Content-Type: application/json".to_string());
    }
    curl_args.extend(resolved_args);
    curl_args
}

fn resolve_args(base_url: &str, args: &[String]) -> Vec<String> {
    let mut resolved_args: Vec<String> = Vec::with_capacity(args.len());
    for arg in args {
        if let Some(rest) = arg.strip_prefix(CDX_SCHEME) {
            resolved_args.push(build_cdx_url(base_url, rest));
        } else {
            resolved_args.push(arg.clone());
        }
    }
    resolved_args
}

fn infer_request_defaults(args: &[String]) -> RequestDefaults {
    let features = inspect_args(args);
    let should_infer_json_request =
        features.has_json_body && !features.has_get_override && !features.uses_curl_json;

    RequestDefaults {
        add_silent_defaults: !features.has_output_mode_override,
        add_post: should_infer_json_request && !features.has_explicit_method,
        add_json_content_type: should_infer_json_request && !features.has_content_type_header,
    }
}

fn inspect_args(args: &[String]) -> ArgFeatures {
    let mut features = ArgFeatures::default();
    let mut expects_header_value = false;
    let mut expects_request_value = false;
    let mut expects_data_value = false;

    for arg in args {
        if expects_header_value {
            if is_content_type_header(arg) {
                features.has_content_type_header = true;
            }
            expects_header_value = false;
            continue;
        }

        if expects_request_value {
            features.has_explicit_method = true;
            expects_request_value = false;
            continue;
        }

        if expects_data_value {
            features.has_json_body = true;
            expects_data_value = false;
            continue;
        }

        match arg.as_str() {
            "-H" | "--header" => {
                expects_header_value = true;
                continue;
            }
            "-X" | "--request" => {
                features.has_explicit_method = true;
                expects_request_value = true;
                continue;
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                features.has_json_body = true;
                expects_data_value = true;
                continue;
            }
            "--json" => {
                features.uses_curl_json = true;
                continue;
            }
            "-G" | "--get" => {
                features.has_get_override = true;
                continue;
            }
            "--silent" | "--no-silent" | "--show-error" | "--verbose" | "--trace"
            | "--trace-ascii" | "--trace-config" | "--progress-bar" => {
                features.has_output_mode_override = true;
                continue;
            }
            _ => {}
        }

        if let Some(value) = arg.strip_prefix("--header=") {
            if is_content_type_header(value) {
                features.has_content_type_header = true;
            }
            continue;
        }
        if arg.starts_with("--request=") {
            features.has_explicit_method = true;
            continue;
        }
        if arg.starts_with("--json=") {
            features.uses_curl_json = true;
            continue;
        }
        if arg.starts_with("--data=")
            || arg.starts_with("--data-raw=")
            || arg.starts_with("--data-binary=")
            || arg.starts_with("--data-ascii=")
        {
            features.has_json_body = true;
            continue;
        }
        if let Some(value) = short_option_attached_value(arg, 'H') {
            if is_content_type_header(value) {
                features.has_content_type_header = true;
            }
            continue;
        }
        if short_option_attached_value(arg, 'X').is_some() {
            features.has_explicit_method = true;
            continue;
        }
        if short_option_attached_value(arg, 'd').is_some() {
            features.has_json_body = true;
            continue;
        }
        if short_flag_cluster_contains(arg, 's')
            || short_flag_cluster_contains(arg, 'S')
            || short_flag_cluster_contains(arg, 'v')
            || short_flag_cluster_contains(arg, '#')
        {
            features.has_output_mode_override = true;
        }
    }

    features
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

fn is_content_type_header(value: &str) -> bool {
    value
        .trim_start()
        .get(..13)
        .map(|prefix| prefix.eq_ignore_ascii_case("content-type:"))
        .unwrap_or(false)
}

fn short_option_attached_value<'a>(arg: &'a str, flag: char) -> Option<&'a str> {
    let short = arg.strip_prefix('-')?;
    if arg.starts_with("--") || !short.starts_with(flag) || short.len() <= 1 {
        return None;
    }
    Some(&short[1..])
}

fn short_flag_cluster_contains(arg: &str, flag: char) -> bool {
    let short = match arg.strip_prefix('-') {
        Some(value) if !arg.starts_with("--") => value,
        _ => return false,
    };
    short
        .chars()
        .all(|ch| ch.is_ascii_alphabetic() || ch == '#')
        && short.contains(flag)
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
    println!("cdx <url> [curl overrides]");
    println!("Opinionated defaults:");
    println!("  - adds -sS unless you explicitly choose curl output/verbosity flags");
    println!("  - with -d/--data*, adds POST and Content-Type: application/json unless overridden");
    println!("Supports cdx:// URLs.");
    println!("Examples:");
    println!("  cdx \"cdx://doc/CR10_2025_01_01/text\"");
    println!("  cdx \"cdx://search/CR\" -d '{{\"query\":\"občanský zákoník\",\"limit\":5}}'");
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
    fn build_curl_args_adds_silent_defaults_for_get_requests() {
        let args = vec!["cdx://doc/CR10_2026_01_01/text".to_string()];
        let curl_args = build_curl_args("https://app.codexis.cz", "a.b.c", &args);

        assert_eq!(
            curl_args,
            vec![
                "-sS",
                "-H",
                "Authorization: Bearer a.b.c",
                "https://app.codexis.cz/rest/cdx-api/doc/CR10_2026_01_01/text",
            ]
        );
    }

    #[test]
    fn build_curl_args_infers_post_and_json_header_from_data() {
        let args = vec![
            "cdx://search/CR".to_string(),
            "-d".to_string(),
            "{\"query\":\"test\"}".to_string(),
        ];
        let curl_args = build_curl_args("https://app.codexis.cz", "Bearer abc", &args);

        assert_eq!(
            curl_args,
            vec![
                "-sS",
                "-H",
                "Authorization: Bearer abc",
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "https://app.codexis.cz/rest/cdx-api/search/CR",
                "-d",
                "{\"query\":\"test\"}",
            ]
        );
    }

    #[test]
    fn build_curl_args_does_not_duplicate_content_type_or_method() {
        let args = vec![
            "-X".to_string(),
            "PUT".to_string(),
            "-H".to_string(),
            "Content-Type: application/merge-patch+json".to_string(),
            "cdx://doc/CR10_2026_01_01".to_string(),
            "-d".to_string(),
            "{\"note\":\"x\"}".to_string(),
        ];
        let curl_args = build_curl_args("https://app.codexis.cz", "a.b.c", &args);

        assert_eq!(
            curl_args,
            vec![
                "-sS",
                "-H",
                "Authorization: Bearer a.b.c",
                "-X",
                "PUT",
                "-H",
                "Content-Type: application/merge-patch+json",
                "https://app.codexis.cz/rest/cdx-api/doc/CR10_2026_01_01",
                "-d",
                "{\"note\":\"x\"}",
            ]
        );
    }

    #[test]
    fn build_curl_args_respects_get_override_for_query_encoding() {
        let args = vec![
            "-G".to_string(),
            "cdx://search/CR".to_string(),
            "-d".to_string(),
            "query=test".to_string(),
        ];
        let curl_args = build_curl_args("https://app.codexis.cz", "a.b.c", &args);

        assert_eq!(
            curl_args,
            vec![
                "-sS",
                "-H",
                "Authorization: Bearer a.b.c",
                "-G",
                "https://app.codexis.cz/rest/cdx-api/search/CR",
                "-d",
                "query=test",
            ]
        );
    }

    #[test]
    fn build_curl_args_keeps_user_selected_output_mode() {
        let args = vec![
            "-v".to_string(),
            "cdx://doc/CR10_2026_01_01/text".to_string(),
        ];
        let curl_args = build_curl_args("https://app.codexis.cz", "a.b.c", &args);

        assert_eq!(
            curl_args,
            vec![
                "-H",
                "Authorization: Bearer a.b.c",
                "-v",
                "https://app.codexis.cz/rest/cdx-api/doc/CR10_2026_01_01/text",
            ]
        );
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
