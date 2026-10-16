mod alerts;
mod files;
mod watch_sets;

use anyhow::Result;
use uuid::Uuid;

use crate::api::{ApiClient, ApiResult};
use crate::cli::{Command, PageArgs, ReferenceCommand};
use crate::model::{Page, Reference};
use crate::output::{Output, table, text};

pub fn dispatch(client: &ApiClient, output: &Output, command: Command) -> Result<()> {
    match command {
        Command::Me => {
            let me = client.me()?;
            output.print(&me, |me| {
                println!(
                    "{}",
                    table(
                        ["id", "name", "email"],
                        [[me.id.to_string(), text(&me.name), text(&me.email)]]
                    )
                );
            })
        }
        Command::WatchSets(command) => watch_sets::run(client, output, command),
        Command::Files(command) => files::run(client, output, command),
        Command::Documents { watch_set } => documents(client, output, watch_set),
        Command::References(ReferenceCommand::List {
            watch_set,
            unlinked,
            page,
        }) => {
            let linked = unlinked.then_some(false);
            let references = collect(&page, |after, limit| {
                client.references(watch_set, linked, after, limit)
            })?;
            output.print(&references, |references| {
                print_references(&references.items)
            })
        }
        Command::Alerts(command) => alerts::run(client, output, command),
        Command::Auth(_) | Command::Completions { .. } => {
            unreachable!("auth and completions run without an API client")
        }
    }
}

fn documents(client: &ApiClient, output: &Output, watch_set: Uuid) -> Result<()> {
    let documents = client.documents(watch_set)?;
    output.print(&documents, |documents| {
        if documents.is_empty() {
            println!("No watched legislation yet.");
        }
        for document in documents {
            println!(
                "{}  {}  [{}]",
                document.citation, document.title, document.status
            );
            if document.whole_document {
                println!("  └ whole document");
            }
            for section in document
                .sections
                .iter()
                .filter(|section| section.part_id.is_some())
            {
                println!("  └ {}  ({})", text(&section.label), section.id);
            }
        }
    })
}

pub(crate) fn print_references(references: &[Reference]) {
    println!(
        "{}",
        table(
            ["id", "kind", "text", "points to", "found in", "reason"],
            references.iter().map(|reference| {
                [
                    reference.id.to_string(),
                    reference.kind.clone(),
                    reference.text.clone(),
                    match (&reference.document_id, &reference.part_id) {
                        (Some(document), Some(part)) => format!("{document}#{part}"),
                        (Some(document), None) => document.clone(),
                        _ => "not linked".to_owned(),
                    },
                    match (&reference.file_name, &reference.source_part_id) {
                        (Some(file), Some(part)) => format!(
                            "{}{file} ({part})",
                            reference.path.clone().unwrap_or_default()
                        ),
                        (Some(file), None) => {
                            format!("{}{file}", reference.path.clone().unwrap_or_default())
                        }
                        _ => "brief".to_owned(),
                    },
                    text(&reference.reason),
                ]
            }),
        )
    );
}

/// Fetches one page, or every page when `--all` is given, following `nextCursor`.
pub(crate) fn collect<T>(
    page: &PageArgs,
    mut fetch: impl FnMut(Option<Uuid>, Option<u16>) -> ApiResult<Page<T>>,
) -> ApiResult<Page<T>> {
    let first = fetch(page.after, page.limit)?;
    if !page.all {
        return Ok(first);
    }
    let mut items = first.items;
    let mut cursor = first.next_cursor;
    while let Some(after) = cursor {
        let next = fetch(Some(after), page.limit)?;
        items.extend(next.items);
        cursor = next.next_cursor;
    }
    Ok(Page {
        items,
        next_cursor: None,
    })
}

pub(crate) fn print_next_cursor(next_cursor: Option<Uuid>) {
    if let Some(cursor) = next_cursor {
        println!("More results: --after {cursor} (or --all)");
    }
}
