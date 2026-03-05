use std::env;
use std::process::{Command, Stdio};

const CDX_SCHEME: &str = "cdx://";
const API_PREFIX: &str = "/rest/cdx-api";
const CDX_API_JWT_AUTH_ENV: &str = "CDX_API_JWT_AUTH";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return;
    }

    let base_url = match env::var("CODEXIS_API_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().trim_end_matches('/').to_string(),
        _ => {
            eprintln!("CODEXIS_API_URL must be set (e.g., export CODEXIS_API_URL=\"https://.../\")");
            std::process::exit(2);
        }
    };

    let mut resolved_args: Vec<String> = Vec::with_capacity(args.len());
    for arg in args {
        if let Some(rest) = arg.strip_prefix(CDX_SCHEME) {
            resolved_args.push(build_cdx_url(&base_url, rest));
        } else {
            resolved_args.push(arg);
        }
    }

    let jwt_auth = env::var(CDX_API_JWT_AUTH_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut curl_args: Vec<String> = Vec::with_capacity(resolved_args.len() + 2);
    if let Some(jwt_auth) = jwt_auth.as_deref() {
        curl_args.push("-H".to_string());
        curl_args.push(to_authorization_header(jwt_auth));
    }
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
    println!("Requires CODEXIS_API_URL to be set in the environment.");
    println!("Optional: if {CDX_API_JWT_AUTH_ENV} is set, it is used for the Authorization header.");
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
