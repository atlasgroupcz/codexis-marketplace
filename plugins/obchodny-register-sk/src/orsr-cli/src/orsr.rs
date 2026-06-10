use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::http::RegistryClient;

const ORSR_BASE_URL: &str = "https://www.orsr.sk";

/// Resolve orsr.sk extract deep links for an IČO by scraping the link targets
/// (and nothing else) out of the windows-1250 search-result page. Only the
/// ASCII `vypis.asp?...` hrefs are read, so the legacy encoding is irrelevant.
pub fn resolve_links(client: &RegistryClient, ico: &str) -> Result<Value> {
    let search_url = format!("{ORSR_BASE_URL}/hladaj_ico.asp?ICO={ico}&SID=0");
    let body = client.get_bytes(&search_url)?;
    let page = String::from_utf8_lossy(&body);

    let mut subjects: Vec<(String, String)> = Vec::new();
    for fragment in page.split("vypis.asp?").skip(1) {
        let href: String = fragment
            .chars()
            .take_while(|c| *c != '"' && *c != '\'' && *c != '<' && *c != ' ')
            .collect();
        let href = href.replace("&amp;", "&");
        let (Some(id), Some(sid)) = (param(&href, "ID"), param(&href, "SID")) else {
            continue;
        };
        if !subjects.iter().any(|(i, s)| *i == id && *s == sid) {
            subjects.push((id, sid));
        }
    }

    if subjects.is_empty() {
        return Err(anyhow!(
            "no orsr.sk entry found for IČO {ico} (searched {search_url})"
        ));
    }

    let entries: Vec<Value> = subjects
        .into_iter()
        .map(|(id, sid)| {
            json!({
                "id": id,
                "sid": sid,
                "aktualnyVypis": format!("{ORSR_BASE_URL}/vypis.asp?ID={id}&SID={sid}&P=0"),
                "uplnyVypis": format!("{ORSR_BASE_URL}/vypis.asp?ID={id}&SID={sid}&P=1"),
            })
        })
        .collect();

    Ok(json!({ "ico": ico, "subjekty": entries }))
}

fn param(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name && value.chars().all(|c| c.is_ascii_digit()))
            .then(|| value.to_string())
    })
}
