use crate::client::GraphQLClient;
use crate::commands::project;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use clap::{Args, Subcommand};
use serde_json::{json, Value};

#[derive(Subcommand)]
pub enum ChatCommands {
    #[command(about = "Search chats using the platform's search")]
    Search(SearchArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    #[arg(help = "Search query", value_parser = nonblank)]
    query: String,
    #[arg(long, help = "Project name, UUID or Node ID", value_parser = nonblank)]
    project: Option<String>,
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u16).range(1..=100))]
    limit: u16,
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u32).range(0..=i32::MAX as i64))]
    offset: u32,
}

fn nonblank(value: &str) -> Result<String, String> {
    if value.trim().is_empty() {
        return Err("value must not be blank".into());
    }
    Ok(value.to_string())
}

pub fn execute(
    client: &GraphQLClient,
    command: ChatCommands,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let ChatCommands::Search(args) = command;
    let hits = search(client, &args)?;
    let output = match format {
        OutputFormat::Json => hits,
        OutputFormat::Table => table_rows(hits),
    };
    print_output(&output, format);
    Ok(())
}

fn search(client: &GraphQLClient, args: &SearchArgs) -> Result<Value, CdxctlError> {
    let mut variables = json!({
        "query": args.query,
        "limit": args.limit,
        "offset": args.offset,
    });
    if let Some(project) = &args.project {
        variables["projectId"] = json!(project::resolve_project_id(client, project)?);
    }
    let data = client.execute(graphql::SEARCH_CHATS, variables)?;
    data.get("search")
        .filter(|hits| hits.is_array())
        .cloned()
        .ok_or_else(|| CdxctlError::Parse("Response missing search results".into()))
}

fn table_rows(mut hits: Value) -> Value {
    if let Some(rows) = hits.as_array_mut() {
        for hit in rows {
            let name = hit.pointer("/project/name").cloned().unwrap_or(Value::Null);
            hit["project"] = name;
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::encode_node_id;
    use crate::test_support::stub_daemon_responses;
    use clap::Parser;
    use std::sync::mpsc::Receiver;
    use std::time::Duration;

    const PROJECT_UUID: &str = "9fb16d74-5551-4235-a806-d5f3cd3a089c";

    fn daemon(responses: Vec<Value>) -> (GraphQLClient, Receiver<String>) {
        let (address, requests) = stub_daemon_responses(
            responses
                .into_iter()
                .map(|value| ("HTTP/1.1 200 OK", value.to_string()))
                .collect(),
        );
        (
            GraphQLClient::with_endpoint(
                format!("http://{address}/graphql"),
                "Bearer test-token".into(),
            ),
            requests,
        )
    }

    fn args(project: Option<&str>) -> SearchArgs {
        SearchArgs {
            query: "  nájem OR výpověď  ".into(),
            project: project.map(str::to_string),
            limit: 5,
            offset: 10,
        }
    }

    fn request(requests: &Receiver<String>) -> Value {
        let raw = requests.recv_timeout(Duration::from_secs(3)).unwrap();
        assert!(raw
            .to_lowercase()
            .contains("authorization: bearer test-token"));
        serde_json::from_str(raw.split_once("\r\n\r\n").unwrap().1).unwrap()
    }

    #[test]
    fn passes_query_and_pagination_and_preserves_results() {
        let hits = json!([
            {"id":"chat-b", "title":"Nájem", "snippet":"<mark>nájem</mark>", "modifiedAt":"2026-10-08T10:00:00Z", "automation":true, "project":{"id":"project-id", "name":"Právní agenda"}},
            {"id":"chat-a", "title":null, "snippet":null, "modifiedAt":null, "automation":false, "project":null}
        ]);
        let (client, requests) = daemon(vec![json!({"data":{"search":hits}})]);
        assert_eq!(search(&client, &args(None)).unwrap(), hits);
        let sent = request(&requests);
        assert_eq!(sent["query"], graphql::SEARCH_CHATS);
        assert_eq!(
            sent["variables"],
            json!({"query":"  nájem OR výpověď  ", "limit":5, "offset":10})
        );
    }

    #[test]
    fn defaults_to_twenty_results_without_a_project_filter() {
        let cli = crate::Cli::try_parse_from(["cdxctl", "chat", "search", "nájem"]).unwrap();
        let crate::Commands::Chat { command } = cli.command else {
            panic!("expected chat command");
        };
        let (client, requests) = daemon(vec![json!({"data":{"search":[]}})]);
        execute(&client, command, OutputFormat::Json).unwrap();
        assert_eq!(
            request(&requests)["variables"],
            json!({"query":"nájem", "limit":20, "offset":0})
        );
    }

    #[test]
    fn executes_table_output_and_propagates_search_failure() {
        let (client, _requests) = daemon(vec![
            json!({"data":{"search":[{"id":"chat-a","project":{"id":"project-a","name":"Právní agenda"}}]}}),
        ]);
        execute(
            &client,
            ChatCommands::Search(args(None)),
            OutputFormat::Table,
        )
        .unwrap();
        let (client, _requests) =
            daemon(vec![json!({"errors":[{"message":"Search unavailable"}]})]);
        assert!(execute(
            &client,
            ChatCommands::Search(args(None)),
            OutputFormat::Json
        )
        .is_err());
    }

    #[test]
    fn resolves_project_name_uuid_and_node_id_before_searching() {
        let id = encode_node_id("Project", PROJECT_UUID);
        for identifier in ["Právní agenda", PROJECT_UUID, id.as_str()] {
            let (client, requests) = daemon(vec![
                json!({"data":{"projects":[{"id":id,"name":"Právní agenda"}]}}),
                json!({"data":{"search":[]}}),
            ]);
            assert_eq!(search(&client, &args(Some(identifier))).unwrap(), json!([]));
            assert_eq!(request(&requests)["query"], graphql::GET_PROJECTS);
            assert_eq!(request(&requests)["variables"]["projectId"], id);
        }
    }

    #[test]
    fn rejects_unknown_project_without_searching() {
        let (client, requests) = daemon(vec![json!({"data":{"projects":[]}})]);
        let error = search(&client, &args(Some("Missing"))).unwrap_err();
        assert!(error.to_string().contains("No such project: Missing"));
        request(&requests);
        assert!(requests.recv_timeout(Duration::from_millis(50)).is_err());
    }

    #[test]
    fn rejects_ambiguous_project_names_and_lists_ids() {
        let first = encode_node_id("Project", PROJECT_UUID);
        let second = encode_node_id("Project", "b4e798df-34ee-43a9-b1e8-8ea3b45e9668");
        let (client, _requests) = daemon(vec![json!({"data":{"projects":[
            {"id":first,"name":"Duplicate"}, {"id":second,"name":"Duplicate"}
        ]}})]);
        let message = search(&client, &args(Some("Duplicate")))
            .unwrap_err()
            .to_string();
        assert!(message.contains("Ambiguous project"));
        assert!(message.contains(&first));
        assert!(message.contains(&second));
    }

    #[test]
    fn project_id_takes_precedence_over_a_matching_name() {
        let first = encode_node_id("Project", PROJECT_UUID);
        let second = encode_node_id("Project", "b4e798df-34ee-43a9-b1e8-8ea3b45e9668");
        let (client, requests) = daemon(vec![
            json!({"data":{"projects":[
                {"id":second,"name":first}, {"id":first,"name":"Právní agenda"}
            ]}}),
            json!({"data":{"search":[]}}),
        ]);
        assert!(search(&client, &args(Some(&first))).is_ok());
        request(&requests);
        assert_eq!(request(&requests)["variables"]["projectId"], first);
    }

    #[test]
    fn propagates_graphql_and_http_errors() {
        let (client, _requests) =
            daemon(vec![json!({"errors":[{"message":"Search unavailable"}]})]);
        assert!(search(&client, &args(None))
            .unwrap_err()
            .to_string()
            .contains("Search unavailable"));
        let (address, _requests) =
            stub_daemon_responses(vec![("HTTP/1.1 503 Service Unavailable", "offline".into())]);
        let client = GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".into(),
        );
        assert!(search(&client, &args(None))
            .unwrap_err()
            .to_string()
            .contains("503"));
    }

    #[test]
    fn rejects_missing_or_invalid_results() {
        for data in [json!({}), json!({"search":null}), json!({"search":{}})] {
            let (client, _requests) = daemon(vec![json!({"data":data})]);
            assert!(search(&client, &args(None))
                .unwrap_err()
                .to_string()
                .contains("Response missing search results"));
        }
    }

    #[test]
    fn table_shows_project_names_and_handles_chats_without_projects() {
        assert_eq!(
            table_rows(json!([
                {"id":"a","project":{"id":"p","name":"Právní agenda"}},
                {"id":"b","project":null}
            ])),
            json!([
                {"id":"a","project":"Právní agenda"}, {"id":"b","project":null}
            ])
        );
    }
}
