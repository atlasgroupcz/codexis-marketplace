use std::io::Read;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};

use anyhow::{anyhow, Result};
use ureq::Agent;

const DEFAULT_RPO_BASE_URL: &str = "https://api.statistics.sk/rpo/v1";
const DEFAULT_RUZ_BASE_URL: &str = "https://www.registeruz.sk/cruz-public/api";
const DEFAULT_CACHE_TTL_SECS: u64 = 24 * 60 * 60;
const MAX_ATTEMPTS: usize = 3;
const MAX_URL_LEN: usize = 2000;
const USER_AGENT_VALUE: &str = concat!("orsr-cli/", env!("CARGO_PKG_VERSION"));

pub struct RegistryClient {
    agent: Agent,
    rpo_base_url: String,
    ruz_base_url: String,
    cache_enabled: bool,
    cache_ttl: Duration,
}

impl RegistryClient {
    pub fn new(cache_enabled: bool) -> Result<Self> {
        let rpo_base_url = base_url_from_env("RPO_BASE_URL", DEFAULT_RPO_BASE_URL);
        let ruz_base_url = base_url_from_env("RUZ_BASE_URL", DEFAULT_RUZ_BASE_URL);
        // ureq keeps a connection pool on the Agent -> the retry loops reuse the
        // TCP+TLS connection to the same host.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .build();
        Ok(Self {
            agent,
            rpo_base_url,
            ruz_base_url,
            cache_enabled,
            cache_ttl: configured_cache_ttl(),
        })
    }

    pub fn rpo_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&build_url(&self.rpo_base_url, path, query)?)
    }

    pub fn ruz_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(&build_url(&self.ruz_base_url, path, query)?)
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        for attempt in 0..MAX_ATTEMPTS {
            let resp = match self
                .agent
                .get(url)
                .set("User-Agent", USER_AGENT_VALUE)
                .call()
            {
                Ok(resp) => resp,
                Err(ureq::Error::Status(code, resp)) => {
                    if retryable_status(code) && attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(anyhow!(
                        "HTTP {} on GET {url}",
                        format_status(code, resp.status_text())
                    ));
                }
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(anyhow!("GET {url} failed: {error}"));
                }
            };
            let mut body = Vec::new();
            // TODO: drop once orsr.sk sends TLS close_notify - its legacy server
            // closes EOF-delimited responses abruptly, which strict rustls reads
            // report as an error even though the body already arrived.
            if let Err(e) = resp.into_reader().read_to_end(&mut body) {
                if body.is_empty() {
                    return Err(anyhow!("error reading GET {url}: {e}"));
                }
            }
            return Ok(body);
        }
        Err(anyhow!("GET {url} failed after {MAX_ATTEMPTS} attempts"))
    }

    fn get_json(&self, url: &str) -> Result<String> {
        if let Some(cached) = self.read_cache(url) {
            return Ok(cached);
        }
        for attempt in 0..MAX_ATTEMPTS {
            let resp = match self
                .agent
                .get(url)
                .set("Accept", "application/json")
                .set("User-Agent", USER_AGENT_VALUE)
                .call()
            {
                Ok(resp) => resp,
                Err(ureq::Error::Status(code, resp)) => {
                    if retryable_status(code) && attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    let final_url = resp.get_url().to_string();
                    let status = format_status(code, resp.status_text());
                    let body = resp.into_string().unwrap_or_default();
                    return Err(anyhow!(
                        "HTTP {status} on GET {final_url}\n{body}",
                        status = status,
                        body = body.trim()
                    ));
                }
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(anyhow!("GET {url} failed: {error}"));
                }
            };
            let final_url = resp.get_url().to_string();
            let body = resp.into_string().unwrap_or_default();
            if body.trim_start().starts_with('<') {
                return Err(anyhow!(
                    "GET {final_url} returned HTML instead of JSON - the service is unavailable or the request was blocked"
                ));
            }
            self.write_cache(url, &body);
            return Ok(body);
        }
        Err(anyhow!("GET {url} failed after {MAX_ATTEMPTS} attempts"))
    }

    fn read_cache(&self, url: &str) -> Option<String> {
        if !self.cache_enabled || self.cache_ttl.is_zero() {
            return None;
        }
        let path = cache_file_for(url)?;
        let metadata = fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        if modified.elapsed().ok()? > self.cache_ttl {
            return None;
        }
        fs::read_to_string(path).ok()
    }

    fn write_cache(&self, url: &str, body: &str) {
        if !self.cache_enabled || self.cache_ttl.is_zero() {
            return;
        }
        let Some(path) = cache_file_for(url) else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_ok() {
            let _ = fs::write(path, body);
        }
    }
}

fn build_url(base: &str, path: &str, query: &[(&str, &str)]) -> Result<String> {
    let mut url = join_url(base, path);
    if !query.is_empty() {
        url.push('?');
        for (i, (key, value)) in query.iter().enumerate() {
            if i > 0 {
                url.push('&');
            }
            form_urlencode_into(&mut url, key);
            url.push('=');
            form_urlencode_into(&mut url, value);
        }
    }
    if url.len() > MAX_URL_LEN {
        return Err(anyhow!(
            "API URL is {} characters; URLs must stay under {} characters",
            url.len(),
            MAX_URL_LEN
        ));
    }
    Ok(url)
}

/// Percent-encodes `value` into `out` using `application/x-www-form-urlencoded`
/// rules (space -> `+`, uppercase hex), matching what reqwest's `Url` query
/// serializer produced. Keeps built URLs - and therefore cache keys - stable.
fn form_urlencode_into(out: &mut String, value: &str) {
    for &byte in value.as_bytes() {
        match byte {
            b'*' | b'-' | b'.' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
}

/// Reads an env var (trimmed of a trailing slash), falling back to `default`.
/// (Was `cdx_common::http::base_url_from_env`.)
fn base_url_from_env(var: &str, default: &str) -> String {
    env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Joins a base URL and a path; an empty path yields the base unchanged.
/// (Was `cdx_common::http::join_url`.)
fn join_url(base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

/// `"<code> <reason>"`, or just the code when the reason phrase is empty.
/// (Was `cdx_common::http::format_status`, adapted to ureq's `u16` status.)
fn format_status(code: u16, reason: &str) -> String {
    if reason.is_empty() {
        code.to_string()
    } else {
        format!("{code} {reason}")
    }
}

/// Whether a status is worth retrying (429 or any 5xx).
/// (Was `cdx_common::http::retryable_status`, adapted to ureq's `u16` status.)
fn retryable_status(code: u16) -> bool {
    code == 429 || (500..600).contains(&code)
}

/// Exponential backoff: 250ms . 2^attempt.
/// (Was `cdx_common::http::backoff`.)
fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(250 * 2_u64.pow(attempt as u32))
}

fn configured_cache_ttl() -> Duration {
    env::var("ORSR_CLI_CACHE_TTL_SECONDS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_CACHE_TTL_SECS))
}

fn cache_file_for(url: &str) -> Option<PathBuf> {
    let cache_root = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(
        cache_root
            .join("orsr-cli")
            .join(format!("{}.json", cache_key(url))),
    )
}

fn cache_key(url: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in url.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_stable_for_same_url() {
        assert_eq!(
            cache_key("https://api.statistics.sk/rpo/v1/search?identifier=31333532"),
            cache_key("https://api.statistics.sk/rpo/v1/search?identifier=31333532")
        );
    }

    #[test]
    fn cache_key_changes_with_query() {
        assert_ne!(
            cache_key("https://api.statistics.sk/rpo/v1/search?identifier=31333532"),
            cache_key("https://api.statistics.sk/rpo/v1/search?identifier=45274649")
        );
    }

    #[test]
    fn build_url_appends_query_pairs() {
        let url = build_url("https://api.statistics.sk/rpo/v1", "search", &[("identifier", "00031333532")]).unwrap();
        assert_eq!(
            url,
            "https://api.statistics.sk/rpo/v1/search?identifier=00031333532"
        );
    }

    #[test]
    fn build_url_rejects_overlong_urls() {
        let long_value = "x".repeat(MAX_URL_LEN);
        assert!(build_url("https://api.statistics.sk/rpo/v1", "search", &[("fullName", &long_value)]).is_err());
    }

}
