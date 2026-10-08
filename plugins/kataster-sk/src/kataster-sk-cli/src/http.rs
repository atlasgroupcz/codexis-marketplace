use std::time::Duration;

use anyhow::{anyhow, Error, Result};
use cdx_common::http_client::call_with_retry;
use ureq::Agent;

const DEFAULT_ZBGIS_BASE_URL: &str = "https://zbgis.skgeodesy.sk/mapka/api/search/kataster";
const DEFAULT_ESKN_BASE_URL: &str = "https://kataster.skgeodesy.sk/eskn/rest/services/VRM";
const DEFAULT_PORTAL_URL: &str =
    "https://kataster.skgeodesy.sk/eskn-portal/services/info-stav-podania/info-stav-spracovania-ziadost";
// The cadastre WAF rejects non-browser user agents, so impersonate one.
const USER_AGENT_VALUE: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

pub struct KatasterClient {
    agent: Agent,
    zbgis_base_url: String,
    eskn_base_url: String,
    portal_url: String,
}

/// The status form as served: its HTML and the session cookies the submit needs.
pub struct PortalPage {
    pub body: String,
    pub cookie: String,
}

impl KatasterClient {
    pub fn new() -> Result<Self> {
        let zbgis_base_url =
            cdx_common::http::base_url_from_env("ZBGIS_BASE_URL", DEFAULT_ZBGIS_BASE_URL);
        let eskn_base_url =
            cdx_common::http::base_url_from_env("ESKN_BASE_URL", DEFAULT_ESKN_BASE_URL);
        let agent = cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(10),
            Duration::from_secs(30),
        );
        let portal_url = cdx_common::http::base_url_from_env("ESKN_PORTAL_URL", DEFAULT_PORTAL_URL);
        Ok(Self {
            agent,
            zbgis_base_url,
            eskn_base_url,
            portal_url,
        })
    }

    #[cfg(test)]
    pub fn with_portal_url(mut self, url: &str) -> Self {
        self.portal_url = url.to_string();
        self
    }

    pub fn portal_url(&self) -> String {
        self.portal_url.clone()
    }

    /// GET the status form, keeping the cookies the portal sets: the submit is
    /// only accepted inside the session that served the form.
    pub fn portal_get(&self) -> Result<PortalPage> {
        let url = &self.portal_url;
        let mut resp = call_with_retry(|| self.agent.get(url).header("Accept", "text/html").call())
            .into_response("GET", url)
            .map_err(Error::msg)?;
        let cookie = cookie_header(
            resp.headers()
                .get_all("set-cookie")
                .iter()
                .filter_map(|value| value.to_str().ok()),
        );
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| anyhow!("GET {url}: could not read the body: {e}"))?;
        Ok(PortalPage { body, cookie })
    }

    /// POST the status form as a browser would (urlencoded) inside `cookie`'s session.
    pub fn portal_post(&self, cookie: &str, form: &[(&str, &str)]) -> Result<String> {
        let url = &self.portal_url;
        call_with_retry(|| {
            self.agent
                .post(url)
                .header("Accept", "text/html")
                .header("Cookie", cookie)
                .send_form(form.iter().copied())
        })
        .into_text("POST", url)
        .map_err(Error::msg)
    }

    pub fn zbgis_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(
            &cdx_common::http::join_url(&self.zbgis_base_url, path),
            query,
        )
    }

    pub fn eskn_get(&self, path: &str, query: &[(&str, &str)]) -> Result<String> {
        self.get_json(
            &cdx_common::http::join_url(&self.eskn_base_url, path),
            query,
        )
    }

    fn get_json(&self, url: &str, query: &[(&str, &str)]) -> Result<String> {
        let body = call_with_retry(|| {
            self.agent
                .get(url)
                .header("Accept", "application/json")
                .query_pairs(query.iter().copied())
                .call()
        })
        .into_text("GET", url)
        .map_err(Error::msg)?;

        // A WAF block arrives as HTTP 200 + HTML - never retried, and it is the
        // one failure this service reports with a success status.
        if cdx_common::http::looks_like_html(&body) {
            return Err(anyhow!(
                "GET {url} returned HTML instead of JSON - the cadastre WAF blocked the request or the service is unavailable"
            ));
        }
        Ok(body)
    }
}

/// The `Cookie` request header answering a set of `Set-Cookie` response headers:
/// each cookie's `name=value`, attributes dropped.
fn cookie_header<'a>(set_cookies: impl Iterator<Item = &'a str>) -> String {
    set_cookies
        .filter_map(|value| value.split(';').next())
        .map(str::trim)
        .filter(|pair| pair.contains('='))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_header_keeps_name_value_pairs_only() {
        let header = cookie_header(
            [
                "JSESSIONID=abc!1; path=/; secure; HttpOnly",
                "TS01=lb9; Path=/; Domain=.skgeodesy.sk; Secure; SameSite=None",
                "garbage",
            ]
            .into_iter(),
        );
        assert_eq!(header, "JSESSIONID=abc!1; TS01=lb9");
    }

    #[test]
    fn cookie_header_of_nothing_is_empty() {
        assert_eq!(cookie_header(std::iter::empty()), "");
    }
}
