use serde::Serialize;
use uuid::Uuid;

use crate::diff::{DiffOverview, diff_overview};
use crate::format::{czech_date, fold, prague_date, section_key, type_label};
use crate::model::{Alert, Document};

/// A compact, user-facing view of an alert: Czech type and dates, whether it is
/// already in force, the Turboleal link, and the diff read paragraph by paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub change_type: String,
    pub law: Option<String>,
    pub citation: String,
    pub section: Option<String>,
    pub effective_from: Option<String>,
    pub in_force: Option<bool>,
    pub detected_at: String,
    pub amendments: Vec<String>,
    pub acknowledged: bool,
    pub turboleal_url: Option<String>,
    pub overview: Option<DiffOverview>,
    pub diff_markdown: Option<String>,
}

/// `today` is a `YYYY-MM-DD` day in Prague.
pub fn summarize(alert: &Alert, today: &str) -> Summary {
    Summary {
        id: alert.id,
        change_type: type_label(&alert.change_type).to_owned(),
        law: alert
            .law_title
            .as_deref()
            .map(str::trim)
            .filter(|law| !law.is_empty())
            .map(str::to_owned),
        citation: alert.citation.clone(),
        section: alert.label.clone(),
        effective_from: alert.effective_from.as_deref().map(czech_date),
        in_force: alert
            .effective_from
            .as_deref()
            .map(|effective| effective <= today),
        detected_at: czech_date(&prague_date(&alert.detected_at)),
        amendments: alert
            .amendments
            .iter()
            .map(|amendment| amendment.citation.clone())
            .collect(),
        acknowledged: alert.acknowledged_at.is_some(),
        turboleal_url: alert.turboleal_url.clone(),
        overview: diff_overview(alert.diff_markdown.as_deref()),
        diff_markdown: alert.diff_markdown.clone().or_else(|| alert.diff.clone()),
    }
}

/// Client-side alert filters; every bound is inclusive and dates are `YYYY-MM-DD`.
#[derive(Debug, Default)]
pub struct AlertFilter {
    law: Option<String>,
    section: Option<String>,
    effective: (Option<String>, Option<String>),
    detected: (Option<String>, Option<String>),
    types: Vec<String>,
}

impl AlertFilter {
    pub fn new(
        law: Option<&str>,
        section: Option<&str>,
        effective: (Option<String>, Option<String>),
        detected: (Option<String>, Option<String>),
        types: Vec<String>,
    ) -> Self {
        Self {
            law: law.map(fold),
            section: section.map(section_key),
            effective,
            detected,
            types,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.law.is_none()
            && self.section.is_none()
            && self.effective == (None, None)
            && self.detected == (None, None)
            && self.types.is_empty()
    }

    pub fn matches(&self, alert: &Alert) -> bool {
        self.law
            .as_deref()
            .is_none_or(|law| matches_law(law, &alert.citation, alert.law_title.as_deref()))
            && self.section.as_deref().is_none_or(|section| {
                alert
                    .label
                    .as_deref()
                    .is_some_and(|label| section_key(label) == section)
            })
            && (self.types.is_empty() || self.types.contains(&alert.change_type))
            && in_range(alert.effective_from.as_deref(), &self.effective)
            && in_range(Some(&prague_date(&alert.detected_at)), &self.detected)
    }
}

/// Sorts by the day a change takes effect, undated ones last.
pub fn sort_by_effect(alerts: &mut [Alert]) {
    alerts.sort_by(|left, right| {
        let day = |alert: &Alert| {
            alert
                .effective_from
                .clone()
                .unwrap_or_else(|| "9999".into())
        };
        day(left).cmp(&day(right))
    });
}

fn matches_law(law: &str, citation: &str, title: Option<&str>) -> bool {
    fold(citation).contains(law) || title.is_some_and(|title| fold(title).contains(law))
}

fn in_range(date: Option<&str>, (from, to): &(Option<String>, Option<String>)) -> bool {
    if from.is_none() && to.is_none() {
        return true;
    }
    date.is_some_and(|date| {
        from.as_deref().is_none_or(|from| date >= from) && to.as_deref().is_none_or(|to| date <= to)
    })
}

/// A watch set that watches a law, with the watched sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Watched {
    pub watch_set_id: Uuid,
    pub watch_set_name: String,
    pub citation: String,
    pub title: String,
    pub whole_document: bool,
    pub sections: Vec<String>,
    pub turboleal_url: Option<String>,
}

/// The watched documents of one watch set that match `law` and, when given,
/// `section`; a whole watched document matches every section.
pub fn watched_in(
    watch_set_id: Uuid,
    watch_set_name: &str,
    turboleal_url: Option<&str>,
    documents: &[Document],
    law: &str,
    section: Option<&str>,
) -> Vec<Watched> {
    let law = fold(law);
    let section = section.map(section_key);
    documents
        .iter()
        .filter(|document| matches_law(&law, &document.citation, Some(&document.title)))
        .map(|document| Watched {
            watch_set_id,
            watch_set_name: watch_set_name.to_owned(),
            citation: document.citation.clone(),
            title: document.title.clone(),
            whole_document: document.whole_document,
            sections: document
                .sections
                .iter()
                .filter_map(|section| section.label.clone())
                .collect(),
            turboleal_url: turboleal_url.map(str::to_owned),
        })
        .filter(|watched| {
            section.as_deref().is_none_or(|section| {
                watched.whole_document
                    || watched
                        .sections
                        .iter()
                        .any(|label| section_key(label) == section)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::{Amendment, Section};

    fn alert(id: u128) -> Alert {
        Alert {
            id: Uuid::from_u128(id),
            watch_set_id: Uuid::from_u128(100),
            change_type: "UPCOMING_AMENDMENT".into(),
            document_id: "CR1".into(),
            citation: "586/1992 Sb.".into(),
            section_id: Uuid::from_u128(200),
            part_id: Some("paragraf38g".into()),
            label: Some("§ 38g".into()),
            effective_from: Some("2027-01-01".into()),
            detected_at: "2026-10-15T12:00:00Z".into(),
            amendments: vec![Amendment {
                document_id: "CR2".into(),
                citation: "120/2026 Sb.".into(),
            }],
            diff: None,
            diff_markdown: Some("{--old--}{++new++}".into()),
            law_title: Some("Zákon o daních z příjmů".into()),
            turboleal_url: Some("https://turboleal.example/alerts?x".into()),
            references: vec![],
            acknowledged_at: None,
        }
    }

    fn ids(alerts: &[Alert], filter: &AlertFilter) -> Vec<u128> {
        let mut kept: Vec<Alert> = alerts
            .iter()
            .filter(|alert| filter.matches(alert))
            .cloned()
            .collect();
        sort_by_effect(&mut kept);
        kept.iter().map(|alert| alert.id.as_u128()).collect()
    }

    #[test]
    fn a_summary_speaks_czech_and_keeps_the_link_and_diff() {
        let mut alert = alert(1);
        alert.diff_markdown =
            Some("### § 38g (CHANGED)\n\n(1) Text.\n\n{++(6) Nový odstavec.++}".into());

        let summary = summarize(&alert, "2026-10-16");

        assert_eq!(
            serde_json::to_value(summary).unwrap(),
            json!({
                "id": "00000000-0000-0000-0000-000000000001",
                "type": "připravovaná změna, zatím nenabyla účinnosti",
                "law": "Zákon o daních z příjmů",
                "citation": "586/1992 Sb.",
                "section": "§ 38g",
                "effectiveFrom": "1. 1. 2027",
                "inForce": false,
                "detectedAt": "15. 10. 2026",
                "amendments": ["120/2026 Sb."],
                "acknowledged": false,
                "turbolealUrl": "https://turboleal.example/alerts?x",
                "overview": {
                    "items": [{ "kind": "inserted", "where": "odst. 6", "text": "(6) Nový odstavec." }],
                    "renumberingOnly": false
                },
                "diffMarkdown": "### § 38g (CHANGED)\n\n(1) Text.\n\n{++(6) Nový odstavec.++}"
            })
        );
    }

    #[test]
    fn a_summary_dates_detection_in_prague_and_tells_what_is_in_force() {
        let mut alert = alert(1);
        alert.effective_from = Some("2026-01-01".into());
        alert.detected_at = "2026-10-15T23:30:00Z".into();

        let summary = summarize(&alert, "2026-10-16");

        assert_eq!(summary.in_force, Some(true));
        assert_eq!(summary.detected_at, "16. 10. 2026");
    }

    #[test]
    fn a_summary_falls_back_to_the_legacy_diff_without_an_overview() {
        let mut alert = alert(1);
        alert.diff_markdown = None;
        alert.diff = Some("```diff\n-a\n+b\n```".into());
        alert.law_title = Some("  ".into());

        let summary = summarize(&alert, "2026-10-16");

        assert_eq!(summary.overview, None);
        assert_eq!(
            summary.diff_markdown.as_deref(),
            Some("```diff\n-a\n+b\n```")
        );
        assert_eq!(summary.law, None);
    }

    #[test]
    fn alerts_filter_by_law_name_section_dates_and_type() {
        let mut alerts = vec![alert(1), alert(2), alert(3), alert(4)];
        alerts[1].label = Some("§ 38ga".into());
        alerts[2].effective_from = Some("2026-07-01".into());
        alerts[2].change_type = "AMENDED".into();
        alerts[3].document_id = "CR3".into();
        alerts[3].citation = "262/2006 Sb.".into();
        alerts[3].label = Some("§ 35".into());
        alerts[3].law_title = Some("Zákoník práce".into());
        let none = || (None, None);

        let by_law = AlertFilter::new(Some("danich z prijmu"), Some("38g"), none(), none(), vec![]);
        let by_date = AlertFilter::new(
            None,
            None,
            (Some("2026-01-01".into()), Some("2026-12-31".into())),
            none(),
            vec![],
        );
        let by_type = AlertFilter::new(
            Some("262/2006"),
            None,
            none(),
            none(),
            vec!["UPCOMING_AMENDMENT".into()],
        );
        let by_detection = AlertFilter::new(
            None,
            None,
            none(),
            (Some("2026-10-15".into()), Some("2026-10-15".into())),
            vec![],
        );

        assert_eq!(ids(&alerts, &by_law), vec![3, 1]);
        assert_eq!(ids(&alerts, &by_date), vec![3]);
        assert_eq!(ids(&alerts, &by_type), vec![4]);
        assert_eq!(ids(&alerts, &by_detection), vec![3, 1, 2, 4]);
        assert!(AlertFilter::default().is_empty());
        assert!(!by_detection.is_empty());
    }

    #[test]
    fn an_undated_alert_is_outside_every_date_range() {
        let mut undated = alert(1);
        undated.effective_from = None;
        let filter = AlertFilter::new(
            None,
            None,
            (None, Some("2027-12-31".into())),
            (None, None),
            vec![],
        );

        assert!(!filter.matches(&undated));
        assert!(AlertFilter::default().matches(&undated));
    }

    #[test]
    fn watched_finds_the_law_and_the_section_or_nothing() {
        let documents = vec![
            Document {
                document_id: "CR1".into(),
                citation: "586/1992 Sb.".into(),
                title: "Zákon o daních z příjmů".into(),
                status: "VALID".into(),
                whole_document: false,
                sections: vec![Section {
                    id: Uuid::from_u128(1),
                    part_id: Some("paragraf38g".into()),
                    label: Some("§ 38g".into()),
                }],
            },
            Document {
                document_id: "CR3".into(),
                citation: "262/2006 Sb.".into(),
                title: "Zákoník práce".into(),
                status: "VALID".into(),
                whole_document: true,
                sections: vec![],
            },
        ];
        let set = Uuid::from_u128(10);
        let find = |law, section| {
            watched_in(
                set,
                "Pekárna",
                Some("https://t/ws"),
                &documents,
                law,
                section,
            )
        };

        assert_eq!(
            find("586/1992", Some("38g")),
            vec![Watched {
                watch_set_id: set,
                watch_set_name: "Pekárna".into(),
                citation: "586/1992 Sb.".into(),
                title: "Zákon o daních z příjmů".into(),
                whole_document: false,
                sections: vec!["§ 38g".into()],
                turboleal_url: Some("https://t/ws".into()),
            }]
        );
        assert_eq!(find("zakonik prace", Some("§ 52")).len(), 1);
        assert_eq!(find("586/1992", Some("§ 7a")), vec![]);
        assert_eq!(find("89/2012", None), vec![]);
    }
}
