use anyhow::Result;

use crate::api::ApiClient;
use crate::cli::PageArgs;
use crate::commands::{PAGE_LIMIT, collect, documents_of};
use crate::output::{Output, table};
use crate::sources;
use crate::summary::{Watched, watched_in};

pub fn run(client: &ApiClient, output: &Output, law: &str, section: Option<&str>) -> Result<()> {
    let every = PageArgs {
        limit: Some(PAGE_LIMIT),
        after: None,
        all: true,
    };
    let watch_sets = collect(&every, |after, limit| client.watch_sets(after, limit))?.items;
    let ids: Vec<_> = watch_sets.iter().map(|watch_set| watch_set.id).collect();
    let mut watched = Vec::new();
    for (watch_set, documents) in watch_sets.iter().zip(documents_of(client, &ids)) {
        let url = sources::watch_set_page_url(client.app_url(), watch_set.id);
        watched.extend(watched_in(
            watch_set.id,
            &watch_set.name,
            url.as_deref(),
            &documents?,
            law,
            section,
        ));
    }
    output.print(&watched, |watched| print(watched))
}

fn print(watched: &[Watched]) {
    if watched.is_empty() {
        println!("Not watched in any watch set.");
        return;
    }
    println!(
        "{}",
        table(
            ["watch set", "law", "watched"],
            watched.iter().map(|item| {
                [
                    item.watch_set_name.clone(),
                    format!("{} {}", item.citation, item.title),
                    if item.whole_document {
                        "whole document".to_owned()
                    } else {
                        item.sections.join(", ")
                    },
                ]
            }),
        )
    );
}
