use anyhow::Result;

use crate::api::ApiClient;
use crate::cli::AlertCommand;
use crate::commands::{collect, print_next_cursor, print_references};
use crate::model::Alert;
use crate::output::{Output, table, text};

pub fn run(client: &ApiClient, output: &Output, command: AlertCommand) -> Result<()> {
    match command {
        AlertCommand::List {
            watch_set,
            unacknowledged,
            page,
        } => {
            let acknowledged = unacknowledged.then_some(false);
            let alerts = collect(&page, |after, limit| {
                client.alerts(watch_set, acknowledged, after, limit)
            })?;
            output.print(&alerts, |page| {
                print(&page.items);
                print_next_cursor(page.next_cursor);
            })
        }
        AlertCommand::Get { id } => {
            let alert = client.alert(id)?;
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
