use serde_json::{json, Value};

const RUZ_ENTITY_WEB_URL: &str =
    "https://www.registeruz.sk/cruz-public/domain/accountingentity/show";
pub(crate) const LINKS_SOURCES_CAP: usize = 5;

/// Best-effort publish of chat sources; delegates to the shared publisher.
pub(crate) fn try_publish(entries: &[Value]) {
    cdx_common::sources::try_publish("orsr-cli", entries);
}

/// Sources for `links` - one orsr.sk aktuálny výpis per resolved subject, capped.
pub(crate) fn links_sources(links: &Value, ico8: &str) -> Vec<Value> {
    let Some(subjects) = links.get("subjekty").and_then(Value::as_array) else {
        return Vec::new();
    };
    subjects
        .iter()
        .filter_map(|s| s.get("aktualnyVypis").and_then(Value::as_str))
        .take(LINKS_SOURCES_CAP)
        .map(|url| url_entry(url, &vypis_title(ico8)))
        .collect()
}

/// Source for `detail` - the entity's orsr.sk výpis titled by its current name.
pub(crate) fn detail_source(detail_body: &str, links: &Value, ico8: &str) -> Vec<Value> {
    let Some(url) = links
        .pointer("/subjekty/0/aktualnyVypis")
        .and_then(Value::as_str)
    else {
        return Vec::new();
    };
    let detail: Value = serde_json::from_str(detail_body).unwrap_or(Value::Null);
    let title = current_value(detail.get("fullNames")).unwrap_or_else(|| vypis_title(ico8));
    vec![url_entry(url, &title)]
}

/// Source for `ruz` - the registeruz.sk accounting-entity page.
pub(crate) fn ruz_source(record_body: &str, ico8: &str, entity_id: i64) -> Vec<Value> {
    let record: Value = serde_json::from_str(record_body).unwrap_or(Value::Null);
    let title = record
        .get("nazovUJ")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("Register účtovných závierok - IČO {ico8}"));
    vec![url_entry(
        &format!("{RUZ_ENTITY_WEB_URL}/{entity_id}"),
        &title,
    )]
}

/// Current (not yet terminated) IČO from an RPO entity detail.
pub(crate) fn current_ico(detail_body: &str) -> Option<String> {
    let detail: Value = serde_json::from_str(detail_body).ok()?;
    current_value(detail.get("identifiers"))
}

fn current_value(history: Option<&Value>) -> Option<String> {
    let entries = history?.as_array()?;
    entries
        .iter()
        .find(|e| e.get("validTo").is_none())
        .or_else(|| entries.last())
        .and_then(|e| e.get("value"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn vypis_title(ico8: &str) -> String {
    format!("Výpis z obchodného registra SR - IČO {ico8}")
}

fn url_entry(url: &str, title: &str) -> Value {
    json!({ "__kind": "url", "url": url, "title": title })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_links() -> Value {
        json!({
            "ico": "31322832",
            "subjekty": [
                {
                    "id": "19667",
                    "sid": "2",
                    "aktualnyVypis": "https://www.orsr.sk/vypis.asp?ID=19667&SID=2&P=0",
                    "uplnyVypis": "https://www.orsr.sk/vypis.asp?ID=19667&SID=2&P=1"
                }
            ]
        })
    }

    #[test]
    fn links_sources_uses_aktualny_vypis_and_ico_title() {
        let entries = links_sources(&sample_links(), "31322832");
        assert_eq!(
            entries,
            vec![json!({
                "__kind": "url",
                "url": "https://www.orsr.sk/vypis.asp?ID=19667&SID=2&P=0",
                "title": "Výpis z obchodného registra SR - IČO 31322832"
            })]
        );
    }

    #[test]
    fn links_sources_caps_subjects() {
        let subjects: Vec<Value> = (0..8)
            .map(|i| json!({ "aktualnyVypis": format!("https://www.orsr.sk/vypis.asp?ID={i}&SID=2&P=0") }))
            .collect();
        let links = json!({ "subjekty": subjects });
        assert_eq!(links_sources(&links, "00000001").len(), LINKS_SOURCES_CAP);
    }

    #[test]
    fn detail_source_prefers_current_full_name() {
        let detail_body = r#"{
            "identifiers": [{"value": "31322832", "validFrom": "1992-04-29"}],
            "fullNames": [
                {"value": "SLOVNAFT a.s.", "validFrom": "1992-04-29", "validTo": "1998-11-29"},
                {"value": "SLOVNAFT, a.s.", "validFrom": "2008-05-30"}
            ]
        }"#;
        let entries = detail_source(detail_body, &sample_links(), "31322832");
        assert_eq!(entries[0]["title"], json!("SLOVNAFT, a.s."));
        assert_eq!(
            entries[0]["url"],
            json!("https://www.orsr.sk/vypis.asp?ID=19667&SID=2&P=0")
        );
    }

    #[test]
    fn detail_source_without_links_is_empty() {
        assert!(detail_source("{}", &json!({"subjekty": []}), "31322832").is_empty());
    }

    #[test]
    fn ruz_source_links_entity_page() {
        let entries = ruz_source(r#"{"nazovUJ": "SLOVNAFT, a.s."}"#, "31322832", 449752);
        assert_eq!(
            entries,
            vec![json!({
                "__kind": "url",
                "url": "https://www.registeruz.sk/cruz-public/domain/accountingentity/show/449752",
                "title": "SLOVNAFT, a.s."
            })]
        );
    }

    #[test]
    fn current_ico_reads_open_ended_identifier() {
        let body = r#"{"identifiers": [
            {"value": "11111111", "validFrom": "1990-01-01", "validTo": "1995-01-01"},
            {"value": "31322832", "validFrom": "1995-01-02"}
        ]}"#;
        assert_eq!(current_ico(body), Some("31322832".to_string()));
    }
}
