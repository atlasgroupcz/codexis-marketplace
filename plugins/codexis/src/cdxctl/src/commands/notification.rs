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
    // The daemon's CreateNotificationInput exposes the key/value metadata as
    // `vars: [KeyValueInput!]` (KeyValueInput { key, value }). The CLI's
    // `--extra k=v` pairs map straight onto it. (This field used to be named
    // `extra`, which the current daemon schema rejects.)
    let vars: Vec<Value> = extra
        .iter()
        .map(|(k, v)| json!({ "key": k, "value": v }))
        .collect();

    let input = json!({
        "message": message,
        "action": action,
        "link": link,
        "vars": if vars.is_empty() { Value::Null } else { Value::Array(vars) },
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
                    .filter(|n| n.get("seen").map(|v| v.is_null()).unwrap_or(true))
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
    let resolved = resolve_notification_id(client, id)?;
    update_state(client, &resolved, json!({ "seen": true }), format)
}

pub fn confirm(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let resolved = resolve_notification_id(client, id)?;
    update_state(client, &resolved, json!({ "confirmed": true }), format)
}

/// Resolve a notification reference to its NodeId. Accepts either the id (a
/// Notification NodeId or raw inner id) OR the notification's message text.
/// Notifications are naturally referenced by message (unlike agents/skills,
/// whose inner id IS their name), but the daemon mutation needs the id. A unique
/// message match wins; otherwise the arg is treated as an id.
fn resolve_notification_id(client: &GraphQLClient, arg: &str) -> Result<String, CdxctlError> {
    let data = client.execute(graphql::NOTIFICATIONS_QUERY, json!({}))?;
    if let Some(items) = data
        .get("notifications")
        .and_then(|n| n.get("items"))
        .and_then(|i| i.as_array())
    {
        let ids: Vec<&str> = items
            .iter()
            .filter(|n| n.get("message").and_then(|m| m.as_str()) == Some(arg))
            .filter_map(|n| n.get("id").and_then(|i| i.as_str()))
            .collect();
        if let [id] = ids.as_slice() {
            return Ok((*id).to_string());
        }
    }
    Ok(resolve_node_id(arg, "Notification"))
}

fn update_state(
    client: &GraphQLClient,
    id: &str,
    state: Value,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Notification");
    let mut input = state;
    input["ids"] = json!([node_id]);

    let data = client.execute(
        graphql::UPDATE_NOTIFICATION_STATE_MUTATION,
        json!({ "input": input }),
    )?;
    let updated = data
        .get("updateNotificationState")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first().cloned())
        .unwrap_or(Value::Null);
    print_output(&updated, format);
    Ok(())
}
