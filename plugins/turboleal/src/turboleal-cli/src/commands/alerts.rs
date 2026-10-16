use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::api::ApiClient;
use crate::cli::{AlertCommand, PageArgs};
use crate::commands::{PAGE_LIMIT, collect, documents_of, print_next_cursor, print_references};
use crate::format::prague_today;
use crate::model::Alert;
use crate::output::{Output, table, text};
use crate::sources;
use crate::summary::{AlertFilter, Summary, sort_by_effect, summarize};

pub fn run(client: &ApiClient, output: &Output, command: AlertCommand) -> Result<()> {
    match command {
        AlertCommand::List {
            watch_set,
            unacknowledged,
            filter,
            summary,
            page,
        } => {
            let acknowledged = unacknowledged.then_some(false);
            let filter = AlertFilter::new(
                filter.law.as_deref(),
                filter.section.as_deref(),
                (filter.effective_from, filter.effective_to),
                (filter.detected_from, filter.detected_to),
                filter.types,
            );
            let every = summary || !filter.is_empty();
            let page = if every {
                PageArgs {
                    limit: page.limit.or(Some(PAGE_LIMIT)),
                    after: page.after,
                    all: true,
                }
            } else {
                page
            };
            let mut alerts = collect(&page, |after, limit| {
                client.alerts(watch_set, acknowledged, after, limit)
            })?;
            name_and_link(client, &mut alerts.items);
            if every {
                alerts.items.retain(|alert| filter.matches(alert));
                sort_by_effect(&mut alerts.items);
            }
            publish_sources(client, &alerts.items);
            if summary {
                let today = prague_today();
                let summaries: Vec<Summary> = alerts
                    .items
                    .iter()
                    .map(|alert| summarize(alert, &today))
                    .collect();
                return output.print(&summaries, |summaries| print_summaries(summaries));
            }
            output.print(&alerts, |page| {
                print(&page.items);
                print_next_cursor(page.next_cursor);
            })
        }
        AlertCommand::Get { id, summary } => {
            let mut alert = client.alert(id)?;
            name_and_link(client, std::slice::from_mut(&mut alert));
            publish_sources(client, std::slice::from_ref(&alert));
            if summary {
                let summary = summarize(&alert, &prague_today());
                return output.print(&summary, |summary| {
                    print_summaries(std::slice::from_ref(summary))
                });
            }
            output.print(&alert, print_detail)
        }
        AlertCommand::Ack { id } => {
            let alert = client.acknowledge_alert(id)?;
            output.print(&alert, |alert| {
                println!(
                    "Acknowledged alert {} at {}",
                    alert.id,
                    text(&alert.acknowledged_at)
                );
            })
        }
    }
}

/// Adds the law's name from the watch sets' documents and the Turboleal link;
/// a failed lookup only leaves the name out.
fn name_and_link(client: &ApiClient, alerts: &mut [Alert]) {
    let watch_sets: HashSet<_> = alerts.iter().map(|alert| alert.watch_set_id).collect();
    let watch_sets: Vec<_> = watch_sets.into_iter().collect();
    let titles: HashMap<String, String> = documents_of(client, &watch_sets)
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .map(|document| (document.document_id, document.title))
        .collect();
    for alert in alerts {
        alert.law_title = titles.get(&alert.document_id).cloned();
        alert.turboleal_url = sources::alert_page_url(client.app_url(), alert.id);
    }
}

fn publish_sources(client: &ApiClient, alerts: &[Alert]) {
    sources::publish(&sources::alert_sources(
        alerts,
        client.app_url(),
        sources::SOURCES_CAP,
    ));
}

fn print_summaries(summaries: &[Summary]) {
    println!(
        "{}",
        table(
            [
                "type",
                "provision",
                "effective",
                "in force",
                "detected",
                "link"
            ],
            summaries.iter().map(|summary| {
                let law = match &summary.law {
                    Some(law) => format!("{law} ({})", summary.citation),
                    None => summary.citation.clone(),
                };
                [
                    summary.change_type.clone(),
                    match &summary.section {
                        Some(section) => format!("{law}, {section}"),
                        None => law,
                    },
                    text(&summary.effective_from),
                    match summary.in_force {
                        Some(true) => "yes".to_owned(),
                        Some(false) => "no".to_owned(),
                        None => "-".to_owned(),
                    },
                    summary.detected_at.clone(),
                    text(&summary.turboleal_url),
                ]
            }),
        )
    );
}

fn print(alerts: &[Alert]) {
    println!(
        "{}",
        table(
            [
                "id",
                "type",
                "provision",
                "effective",
                "detected",
                "acknowledged"
            ],
            alerts.iter().map(|alert| {
                [
                    alert.id.to_string(),
                    alert.change_type.clone(),
                    provision(alert),
                    text(&alert.effective_from),
                    alert.detected_at.clone(),
                    text(&alert.acknowledged_at),
                ]
            }),
        )
    );
}

fn print_detail(alert: &Alert) {
    print(std::slice::from_ref(alert));
    if !alert.amendments.is_empty() {
        let amendments: Vec<_> = alert
            .amendments
            .iter()
            .map(|amendment| amendment.citation.as_str())
            .collect();
        println!("Amended by: {}", amendments.join(", "));
    }
    if let Some(diff_markdown) = &alert.diff_markdown {
        println!("\n{}", diff_markdown.trim_end());
    } else if let Some(diff) = &alert.diff {
        println!(
            "\n{}",
            diff.trim_start_matches("```diff\n")
                .trim_end_matches("```")
                .trim_end()
        );
    }
    if !alert.references.is_empty() {
        println!("\nWhy it is watched:");
        print_references(&alert.references);
    }
}

fn provision(alert: &Alert) -> String {
    match &alert.label {
        Some(label) => format!("{} {label}", alert.citation),
        None => alert.citation.clone(),
    }
}
