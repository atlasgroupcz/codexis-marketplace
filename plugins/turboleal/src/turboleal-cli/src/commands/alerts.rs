use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::api::ApiClient;
use crate::cli::AlertCommand;
use crate::commands::{collect, print_next_cursor, print_references};
use crate::model::Alert;
use crate::output::{Output, table, text};
use crate::sources;

pub fn run(client: &ApiClient, output: &Output, command: AlertCommand) -> Result<()> {
    match command {
        AlertCommand::List {
            watch_set,
            unacknowledged,
            page,
        } => {
            let acknowledged = unacknowledged.then_some(false);
            let mut alerts = collect(&page, |after, limit| {
                client.alerts(watch_set, acknowledged, after, limit)
            })?;
            alerts
                .items
                .iter_mut()
                .for_each(|alert| link(client, alert));
            publish_sources(client, &alerts.items);
            output.print(&alerts, |page| {
                print(&page.items);
                print_next_cursor(page.next_cursor);
            })
        }
        AlertCommand::Get { id } => {
            let mut alert = client.alert(id)?;
            link(client, &mut alert);
            publish_sources(client, std::slice::from_ref(&alert));
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

fn link(client: &ApiClient, alert: &mut Alert) {
    alert.turboleal_url = sources::alert_page_url(client.app_url(), alert.id);
}

/// Cites the alerts in the chat; the law names come from the watch sets'
/// documents, and a failed lookup only leaves a source without the name.
fn publish_sources(client: &ApiClient, alerts: &[Alert]) {
    let alerts = &alerts[..alerts.len().min(sources::SOURCES_CAP)];
    let watch_sets: HashSet<_> = alerts.iter().map(|alert| alert.watch_set_id).collect();
    let titles: HashMap<String, String> = watch_sets
        .into_iter()
        .filter_map(|watch_set| client.documents(watch_set).ok())
        .flatten()
        .map(|document| (document.document_id, document.title))
        .collect();
    sources::publish(&sources::alert_sources(
        alerts,
        &titles,
        client.app_url(),
        sources::SOURCES_CAP,
    ));
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
