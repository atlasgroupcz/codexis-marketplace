use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use uuid::Uuid;

pub fn create(
    client: &GraphQLClient,
    message: &str,
    action: Option<&str>,
    link: Option<&str>,
    extra: &[(String, String)],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let mut obj = json!({
        "message": message,
        "action": action,
        "link": link,
        "seen": null,
        "confirmed": null,
    });

    // Add extra key-value pairs
    if let Some(map) = obj.as_object_mut() {
        for (key, value) in extra {
            map.insert(key.clone(), json!(value));
        }
    }

    // Create directory ~/.cdx/notifications/YYYY/MM/DD/HH/
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/codexis".to_string());
    let now = Utc::now();
    let dir = format!(
        "{}/.cdx/notifications/{}/{:02}/{:02}/{:02}",
        home,
        now.format("%Y"),
        now.format("%m"),
        now.format("%d"),
        now.format("%H"),
    );
    fs::create_dir_all(&dir)?;

    // Write n_{timestamp_ms}_{uuid}.json
    let timestamp_ms = now.timestamp_millis();
    let uuid = Uuid::new_v4();
    let file_path = format!("{}/n_{}_{}.json", dir, timestamp_ms, uuid);
    let content = serde_json::to_string_pretty(&obj)?;
    fs::write(&file_path, content)?;

    // Call refreshNotifications mutation to trigger daemon pickup
    let data = client.execute(graphql::REFRESH_NOTIFICATIONS_MUTATION, json!({}))?;
    let result = data
        .get("refreshNotifications")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(
        &json!({ "created": file_path, "refreshed": result }),
        format,
    );
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
        .cloned()
        .unwrap_or(Value::Array(vec![]));

    let result = if unseen {
        match notifications {
            Value::Array(arr) => {
                let filtered: Vec<Value> = arr
                    .into_iter()
                    .filter(|n| {
                        n.get("seen")
                            .map(|v| v.is_null() || v == false)
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
    let data = client.execute(
        graphql::MARK_NOTIFICATIONS_SEEN_MUTATION,
        json!({ "ids": [id] }),
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
    let data = client.execute(
        graphql::MARK_NOTIFICATION_CONFIRMED_MUTATION,
        json!({ "id": id }),
    )?;
    let result = data
        .get("markNotificationConfirmed")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&json!({ "markNotificationConfirmed": result }), format);
    Ok(())
}
