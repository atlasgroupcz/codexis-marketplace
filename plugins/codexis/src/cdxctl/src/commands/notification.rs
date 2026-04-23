use super::resolve_node_id;
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn create(
    client: &GraphQLClient,
    message: &str,
    action: Option<&str>,
    link: Option<&str>,
    extra: &[(String, String)],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let extra_kv: Vec<Value> = extra
        .iter()
        .map(|(k, v)| json!({ "key": k, "value": v }))
        .collect();

    let input = json!({
        "message": message,
        "action": action,
        "link": link,
        "extra": if extra_kv.is_empty() { Value::Null } else { Value::Array(extra_kv) },
    });

    let data = client.execute(graphql::CREATE_NOTIFICATION_MUTATION, json!({ "input": input }))?;
    let result = data
        .get("createNotification")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn list(
    client: &GraphQLClient,
    _days: u32,
    unseen: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::NOTIFICATIONS_QUERY, json!({}))?;
    let notifications = data
        .get("notifications")
        .and_then(|n| n.get("items"))
        .cloned()
        .unwrap_or(Value::Array(vec![]));

    let result = if unseen {
        match notifications {
            Value::Array(arr) => {
                let filtered: Vec<Value> = arr
                    .into_iter()
                    .filter(|n| {
                        n.get("seen")
                            .map(|v| v.is_null())
                            .unwrap_or(true)
                    })
                    .collect();
                Value::Array(filtered)
            }
            other => other,
        }
    } else {
        notifications
    };

    print_output(&result, format);
    Ok(())
}

pub fn seen(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Notification");
    let data = client.execute(
        graphql::MARK_NOTIFICATIONS_SEEN_MUTATION,
        json!({ "ids": [node_id] }),
    )?;
    let result = data
        .get("markNotificationsSeen")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&json!({ "markNotificationsSeen": result }), format);
    Ok(())
}

pub fn confirm(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Notification");
    let data = client.execute(
        graphql::MARK_NOTIFICATION_CONFIRMED_MUTATION,
        json!({ "id": node_id }),
    )?;
    let result = data
        .get("markNotificationConfirmed")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&json!({ "markNotificationConfirmed": result }), format);
    Ok(())
}
