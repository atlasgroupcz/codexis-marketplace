use std::io::Read;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::Url;
use cdx_common::http::{backoff, base_url_from_env, format_status, join_url, retryable_status};

const DEFAULT_RPO_BASE_URL: &str = "https://api.statistics.sk/rpo/v1";
const DEFAULT_RUZ_BASE_URL: &str = "https://www.registeruz.sk/cruz-public/api";
const DEFAULT_CACHE_TTL_SECS: u64 = 24 * 60 * 60;
const MAX_ATTEMPTS: usize = 3;
const MAX_URL_LEN: usize = 2000;
const USER_AGENT_VALUE: &str = concat!("orsr-cli/", env!("CARGO_PKG_VERSION"));

pub struct RegistryClient {
    client: Client,
    rpo_base_url: String,
    ruz_base_url: String,
    cache_enabled: bool,
    cache_ttl: Duration,
}

impl RegistryClient {
    pub fn new(cache_enabled: bool) -> Result<Self> {
        let rpo_base_url = base_url_from_env("RPO_BASE_URL", DEFAULT_RPO_BASE_URL);
        let ruz_base_url = base_url_from_env("RUZ_BASE_URL", DEFAULT_RUZ_BASE_URL);
        let client = Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
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
            let mut resp = match self.client.get(url).send() {
                Ok(resp) => resp,
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(error.into());
                }
            };
            let status = resp.status();
            if !status.is_success() {
                if retryable_status(status) && attempt + 1 < MAX_ATTEMPTS {
                    sleep(backoff(attempt));
                    continue;
                }
                return Err(anyhow!("HTTP {} on GET {url}", format_status(status)));
            }
            let mut body = Vec::new();
            // TODO: drop once orsr.sk sends TLS close_notify — its legacy server
            // closes EOF-delimited responses abruptly, which strict rustls reads
            // report as an error even though the body already arrived.
            if let Err(e) = resp.read_to_end(&mut body) {
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
                .client
                .get(url)
                .header(ACCEPT, "application/json")
                .send()
            {
                Ok(resp) => resp,
                Err(error) => {
                    if attempt + 1 < MAX_ATTEMPTS {
                        sleep(backoff(attempt));
                        continue;
                    }
                    return Err(error.into());
                }
            };
            let status = resp.status();
            let final_url = resp.url().to_string();
            let body = resp.text().unwrap_or_default();
            if !status.is_success() {
                if retryable_status(status) && attempt + 1 < MAX_ATTEMPTS {
                    sleep(backoff(attempt));
                    continue;
                }
                return Err(anyhow!(
                    "HTTP {status} on GET {final_url}\n{body}",
                    status = format_status(status),
                    body = body.trim()
                ));
            }
            if body.trim_start().starts_with('<') {
                return Err(anyhow!(
                    "GET {final_url} returned HTML instead of JSON — the service is unavailable or the request was blocked"
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
    let mut url = Url::parse(&join_url(base, path))
        .map_err(|error| anyhow!("invalid API URL: {error}"))?;
    if !query.is_empty() {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in query {
            pairs.append_pair(key, value);
        }
    }
    let url = url.to_string();
    if url.len() > MAX_URL_LEN {
        return Err(anyhow!(
            "API URL is {} characters; URLs must stay under {} characters",
            url.len(),
            MAX_URL_LEN
        ));
    }
    Ok(url)
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
