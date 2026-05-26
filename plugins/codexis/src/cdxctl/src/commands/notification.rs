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
    _extra: &[(String, String)],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    // The daemon's CreateNotificationInput accepts message/action/link/type/vars
    // but NOT `extra`; sending `extra` makes the daemon reject the whole mutation
    // ("field name 'extra' is not defined for input object CreateNotificationInput").
    // Drop it until/unless the daemon schema gains an `extra` field.
    let input = json!({
        "message": message,
        "action": action,
        "link": link,
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
    update_state(client, id, json!({ "seen": true }), format)
}

pub fn confirm(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    update_state(client, id, json!({ "confirmed": true }), format)
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
