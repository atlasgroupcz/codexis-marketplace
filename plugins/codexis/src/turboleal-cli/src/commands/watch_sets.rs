use std::fs;

use anyhow::{Context, Result};

use crate::api::ApiClient;
use crate::cli::WatchSetCommand;
use crate::commands::{collect, print_next_cursor};
use crate::model::{WatchSet, WatchSetInput};
use crate::output::{Output, table};

pub fn run(client: &ApiClient, output: &Output, command: WatchSetCommand) -> Result<()> {
    match command {
        WatchSetCommand::List(page) => {
            let watch_sets = collect(&page, |after, limit| client.watch_sets(after, limit))?;
            output.print(&watch_sets, |page| {
                print(&page.items);
                print_next_cursor(page.next_cursor);
            })
        }
        WatchSetCommand::Get { id } => {
            let watch_set = client.watch_set(id)?;
            output.print(&watch_set, |watch_set| {
                print(std::slice::from_ref(watch_set));
                println!("\n{}", watch_set.brief);
            })
        }
        WatchSetCommand::Create { name, brief } => {
            let input = WatchSetInput {
                name: Some(name),
                brief: Some(read_text(&brief)?),
            };
            let watch_set = client.create_watch_set(&input)?;
            output.print(&watch_set, |watch_set| {
                println!("Created watch set {}", watch_set.id)
            })
        }
        WatchSetCommand::Update { id, name, brief } => {
            let input = WatchSetInput {
                name,
                brief: brief.as_deref().map(read_text).transpose()?,
            };
            let watch_set = client.update_watch_set(id, &input)?;
            output.print(&watch_set, |watch_set| {
                print(std::slice::from_ref(watch_set))
            })
        }
        WatchSetCommand::Delete { id } => {
            client.delete_watch_set(id)?;
            output.message(&format!("Deleted watch set {id}"));
            Ok(())
        }
    }
}

fn print(watch_sets: &[WatchSet]) {
    println!(
        "{}",
        table(
            ["id", "name", "extraction", "created"],
            watch_sets.iter().map(|watch_set| {
                [
                    watch_set.id.to_string(),
                    watch_set.name.clone(),
                    watch_set.extraction_status.clone(),
                    watch_set.created_at.clone(),
                ]
            }),
        )
    );
}

/// `@path` reads the text from a file; anything else is the text itself.
fn read_text(value: &str) -> Result<String> {
    match value.strip_prefix('@') {
        Some(path) => fs::read_to_string(path).with_context(|| format!("cannot read {path}")),
        None => Ok(value.to_owned()),
    }
}
