//! Chat sources for the Sources panel.
//!
//! Citations point at `or.justice.cz` - which is both the portal we query and
//! the official source of record.

use serde_json::{json, Value};

use crate::api::Detail;

const OFFICIAL_BASE: &str = "https://or.justice.cz/ias/ui/rejstrik-$firma";
const SNIPPET_MAX_CHARS: usize = 300;

/// Best-effort publish; no-ops outside a chat VM and never touches stdout.
pub(crate) fn try_publish(entries: &[Value]) {
    cdx_common::sources::try_publish("sbirka-listin-cli", entries);
}

/// One source for `listiny <ICO>` - the subject's official register page.
pub(crate) fn listiny_source(detail: &Detail) -> Vec<Value> {
    let title = if detail.nazev.is_empty() {
        format!("IČO {} - Sbírka listin", detail.ico)
    } else {
        format!("{} - Sbírka listin", detail.nazev)
    };
    let count = detail.pocet_listin;
    let snippet = match (detail.spisovaznacka.as_deref(), detail.sidlo.as_deref()) {
        (Some(spis), Some(sidlo)) => format!("{spis} · {sidlo} · {count} listin"),
        (Some(spis), None) => format!("{spis} · {count} listin"),
        (None, Some(sidlo)) => format!("{sidlo} · {count} listin"),
        (None, None) => format!("{count} listin"),
    };
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert(
        "url".into(),
        json!(format!("{OFFICIAL_BASE}?ico={}", detail.ico)),
    );
    entry.insert("title".into(), json!(title));
    entry.insert(
        "snippet".into(),
        json!(truncate_chars(&snippet, SNIPPET_MAX_CHARS)),
    );
    vec![Value::Object(entry)]
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alza() -> Detail {
        Detail {
            ico: "27082440".into(),
            nazev: "Alza.cz a.s.".into(),
            spisovaznacka: Some("B 8573/MSPH".into()),
            sidlo: Some("Jankovcova 1522/53, Holešovice, 170 00 Praha 7".into()),
            subjekt_id: "701502".into(),
            pocet_listin: 1,
            listiny: vec![],
        }
    }

    #[test]
    fn cites_the_official_register() {
        let entries = listiny_source(&alza());
        let url = entries[0]["url"].as_str().unwrap();
        assert!(url.starts_with("https://or.justice.cz/"), "got {url}");
        assert!(url.contains("ico=27082440"));
    }

    #[test]
    fn source_carries_name_and_count() {
        let entries = listiny_source(&alza());
        assert_eq!(entries[0]["title"], json!("Alza.cz a.s. - Sbírka listin"));
        assert_eq!(
            entries[0]["snippet"],
            json!("B 8573/MSPH · Jankovcova 1522/53, Holešovice, 170 00 Praha 7 · 1 listin")
        );
    }

    #[test]
    fn falls_back_to_ico_title_when_the_name_is_missing() {
        let mut d = alza();
        d.nazev = String::new();
        d.pocet_listin = 0;
        let entries = listiny_source(&d);
        assert_eq!(entries[0]["title"], json!("IČO 27082440 - Sbírka listin"));
    }

    #[test]
    fn snippet_truncates_on_char_boundary() {
        let mut d = alza();
        d.sidlo = Some("Šťěpánská ".repeat(50));
        let entries = listiny_source(&d);
        assert_eq!(entries[0]["snippet"].as_str().unwrap().chars().count(), 300);
    }
}
