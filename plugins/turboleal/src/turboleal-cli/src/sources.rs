use std::collections::{HashMap, HashSet};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::model::{Alert, Document};

pub(crate) const SOURCES_CAP: usize = 10;
const SNIPPET_MAX_CHARS: usize = 300;

/// Best-effort publish of chat sources; delegates to the shared publisher.
pub(crate) fn publish(entries: &[Value]) {
    cdx_common::sources::try_publish("turboleal-cli", entries);
}

/// The Turboleal page of every alert, capped. Turboleal never cites CODEXIS:
/// `titles` maps a CODEXIS document id to the law's name for the title only.
pub(crate) fn alert_sources(
    alerts: &[Alert],
    titles: &HashMap<String, String>,
    app_url: &str,
    cap: usize,
) -> Vec<Value> {
    let pages = alerts.iter().filter_map(|alert| {
        Some(entry(
            &alert_page_url(app_url, alert.id)?,
            &format!(
                "Upozornění v Turbolealu: {}",
                provision_title(
                    titles.get(&alert.document_id).map(String::as_str),
                    &alert.citation,
                    alert.label.as_deref(),
                )
            ),
            Some(&change_summary(alert)),
        ))
    });
    dedupe(pages, cap)
}

/// The watch set's page in Turboleal, with the watched laws as the snippet.
pub(crate) fn document_sources(
    watch_set: Uuid,
    documents: &[Document],
    app_url: &str,
) -> Vec<Value> {
    let Some(url) = watch_set_page_url(app_url, watch_set) else {
        return Vec::new();
    };
    let laws: Vec<String> = documents
        .iter()
        .map(|document| {
            let law = provision_title(Some(&document.title), &document.citation, None);
            match watched_summary(document).as_str() {
                "" => law,
                watched => format!("{law} - {watched}"),
            }
        })
        .collect();
    vec![entry(
        &url,
        "Sledovaná legislativa v Turbolealu",
        Some(&laws.join(" ")),
    )]
}

/// The watch set's detail page in Turboleal.
pub(crate) fn watch_set_page_url(app_url: &str, watch_set: Uuid) -> Option<String> {
    let url = Url::parse(&format!(
        "{}/watch-sets/{}",
        app_url.trim_end_matches('/'),
        node_id("WatchSet", watch_set)
    ))
    .ok()?;
    Some(url.into())
}

/// The Turboleal alerts page with the alert open, as the web app encodes it.
pub(crate) fn alert_page_url(app_url: &str, alert: Uuid) -> Option<String> {
    let mut url = Url::parse(&format!("{}/alerts", app_url.trim_end_matches('/'))).ok()?;
    let state = json!({ "acknowledged": "all", "alert": node_id("Alert", alert) });
    url.query_pairs_mut()
        .append_pair("alerts", &state.to_string());
    Some(url.into())
}

fn node_id(type_name: &str, id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(format!("{type_name}:{id}"))
}

fn dedupe(entries: impl Iterator<Item = Value>, cap: usize) -> Vec<Value> {
    let mut seen = HashSet::new();
    entries
        .filter(|entry| seen.insert(entry["url"].as_str().unwrap_or_default().to_owned()))
        .take(cap)
        .collect()
}

fn entry(url: &str, title: &str, snippet: Option<&str>) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert("url".into(), json!(url));
    entry.insert("title".into(), json!(title));
    if let Some(snippet) = snippet.filter(|snippet| !snippet.is_empty()) {
        entry.insert(
            "snippet".into(),
            json!(snippet.chars().take(SNIPPET_MAX_CHARS).collect::<String>()),
        );
    }
    Value::Object(entry)
}

fn provision_title(law: Option<&str>, citation: &str, label: Option<&str>) -> String {
    let law = match law.map(str::trim).filter(|law| !law.is_empty()) {
        Some(law) => format!("{law} ({citation})"),
        None => citation.to_owned(),
    };
    match label.map(str::trim).filter(|label| !label.is_empty()) {
        Some(label) => format!("{law}, {label}"),
        None => law,
    }
}

fn change_summary(alert: &Alert) -> String {
    let mut parts = vec![format!("{}.", change_type(&alert.change_type))];
    if let Some(date) = alert.effective_from.as_deref() {
        parts.push(format!("Účinnost od {}.", czech_date(date)));
    }
    let amendments: Vec<&str> = alert
        .amendments
        .iter()
        .map(|amendment| amendment.citation.as_str())
        .collect();
    if !amendments.is_empty() {
        parts.push(sentence(&format!("Novela č. {}", amendments.join(", "))));
    }
    parts.join(" ")
}

fn sentence(text: &str) -> String {
    if text.ends_with('.') {
        text.to_owned()
    } else {
        format!("{text}.")
    }
}

fn change_type(value: &str) -> &str {
    match value {
        "AMENDED" => "Změna znění (novela)",
        "UPCOMING_AMENDMENT" => "Připravovaná změna",
        "ADDED" => "Nové ustanovení",
        "REMOVED" => "Zrušené ustanovení",
        "REPEALED" => "Zrušený předpis",
        other => other,
    }
}

fn czech_date(value: &str) -> String {
    let parts: Vec<&str> = value.split('-').collect();
    match parts.as_slice() {
        [year, month, day] => match (month.parse::<u8>(), day.parse::<u8>()) {
            (Ok(month), Ok(day)) => format!("{day}. {month}. {year}"),
            _ => value.to_owned(),
        },
        _ => value.to_owned(),
    }
}

fn watched_summary(document: &Document) -> String {
    if document.whole_document {
        return "Sledovaný celý předpis.".to_owned();
    }
    let labels: Vec<&str> = document
        .sections
        .iter()
        .filter(|section| section.part_id.is_some())
        .filter_map(|section| section.label.as_deref())
        .collect();
    match labels.as_slice() {
        [] => String::new(),
        labels => format!("Sledováno: {}.", labels.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Amendment, Section};

    const APP: &str = "https://turboleal.example/";

    fn alert(id: u128, label: Option<&str>) -> Alert {
        Alert {
            id: Uuid::from_u128(id),
            watch_set_id: Uuid::from_u128(100),
            change_type: "UPCOMING_AMENDMENT".into(),
            document_id: "CR1".into(),
            citation: "586/1992 Sb.".into(),
            section_id: Uuid::from_u128(200),
            part_id: Some("paragraf38g".into()),
            label: label.map(str::to_owned),
            effective_from: Some("2027-01-01".into()),
            detected_at: "2026-10-15T12:00:00Z".into(),
            amendments: vec![Amendment {
                document_id: "CR2".into(),
                citation: "120/2026 Sb.".into(),
            }],
            diff: None,
            diff_markdown: None,
            turboleal_url: None,
            references: vec![],
            acknowledged_at: None,
        }
    }

    fn titles() -> HashMap<String, String> {
        HashMap::from([("CR1".to_owned(), "Zákon o daních z příjmů".to_owned())])
    }

    #[test]
    fn an_alert_cites_only_its_turboleal_page() {
        let entries = alert_sources(&[alert(1, Some("§ 38g"))], &titles(), APP, SOURCES_CAP);

        assert_eq!(
            entries,
            vec![json!({
                "__kind": "url",
                "url": "https://turboleal.example/alerts?alerts=%7B%22acknowledged%22%3A%22all%22%2C%22alert%22%3A%22QWxlcnQ6MDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDAx%22%7D",
                "title": "Upozornění v Turbolealu: Zákon o daních z příjmů (586/1992 Sb.), § 38g",
                "snippet": "Připravovaná změna. Účinnost od 1. 1. 2027. Novela č. 120/2026 Sb."
            })]
        );
    }

    #[test]
    fn the_alert_page_carries_the_graphql_node_id() {
        let url = Url::parse(&alert_page_url(APP, Uuid::from_u128(1)).unwrap()).unwrap();
        let (_, state) = url.query_pairs().next().unwrap();
        let state: Value = serde_json::from_str(&state).unwrap();
        let node = URL_SAFE_NO_PAD
            .decode(state["alert"].as_str().unwrap())
            .unwrap();

        assert_eq!(
            String::from_utf8(node).unwrap(),
            "Alert:00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(state["acknowledged"], json!("all"));
    }

    #[test]
    fn alerts_respect_the_cap() {
        let alerts: Vec<Alert> = (1..=12).map(|id| alert(id, Some("§ 38g"))).collect();

        let entries = alert_sources(&alerts, &titles(), APP, SOURCES_CAP);

        assert_eq!(entries.len(), SOURCES_CAP);
        assert!(entries.iter().all(|entry| {
            entry["url"]
                .as_str()
                .unwrap()
                .starts_with("https://turboleal.example/alerts?")
        }));
    }

    #[test]
    fn an_alert_without_a_law_name_cites_the_citation() {
        let entries = alert_sources(&[alert(1, None)], &HashMap::new(), APP, SOURCES_CAP);

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0]["title"],
            json!("Upozornění v Turbolealu: 586/1992 Sb.")
        );
    }

    #[test]
    fn watched_laws_cite_the_watch_set_page() {
        let documents = [
            Document {
                document_id: "CR3".into(),
                citation: "262/2006 Sb.".into(),
                title: "Zákoník práce".into(),
                status: "VALID".into(),
                whole_document: false,
                sections: vec![
                    Section {
                        id: Uuid::from_u128(1),
                        part_id: Some("paragraf35".into()),
                        label: Some("§ 35".into()),
                    },
                    Section {
                        id: Uuid::from_u128(2),
                        part_id: None,
                        label: None,
                    },
                ],
            },
            Document {
                document_id: "CR4".into(),
                citation: "89/2012 Sb.".into(),
                title: "Občanský zákoník".into(),
                status: "VALID".into(),
                whole_document: true,
                sections: vec![],
            },
        ];

        let entries = document_sources(Uuid::from_u128(7), &documents, APP);

        assert_eq!(
            entries,
            vec![json!({
                "__kind": "url",
                "url": "https://turboleal.example/watch-sets/V2F0Y2hTZXQ6MDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDA3",
                "title": "Sledovaná legislativa v Turbolealu",
                "snippet": "Zákoník práce (262/2006 Sb.) - Sledováno: § 35. Občanský zákoník (89/2012 Sb.) - Sledovaný celý předpis."
            })]
        );
    }

    #[test]
    fn change_summaries_name_the_change_in_czech() {
        let mut repealed = alert(1, None);
        repealed.change_type = "REPEALED".into();
        repealed.effective_from = None;
        repealed.amendments.clear();

        assert_eq!(change_summary(&repealed), "Zrušený předpis.");
        assert_eq!(czech_date("2027-03-09"), "9. 3. 2027");
        assert_eq!(czech_date("soon"), "soon");
    }
}
