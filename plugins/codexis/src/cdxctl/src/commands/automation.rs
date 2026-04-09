use super::resolve_node_id;
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn list(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_AUTOMATIONS, json!({}))?;
    let automations = data
        .get("automations")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    print_output(&automations, format);
    Ok(())
}

pub fn create(
    client: &GraphQLClient,
    title: &str,
    cron: &str,
    prompt: &str,
    description: Option<&str>,
    agent: Option<&str>,
    skills: &[String],
    max_turns: Option<u32>,
    work_dir: Option<&str>,
    disabled: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let mut input = json!({
        "title": title,
        "cron": cron,
        "prompt": prompt,
        "enabled": !disabled,
    });

    if let Some(desc) = description {
        input["description"] = json!(desc);
    }
    if let Some(a) = agent {
        input["agentFullName"] = json!(a);
    }
    if !skills.is_empty() {
        input["skillFullNames"] = json!(skills);
    }
    if let Some(turns) = max_turns {
        input["maxTurns"] = json!(turns);
    }
    if let Some(dir) = work_dir {
        input["workDir"] = json!(dir);
    }

    let data = client.execute(graphql::CREATE_AUTOMATION, json!({ "input": input }))?;
    let result = data.get("createAutomation").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn create_command(
    client: &GraphQLClient,
    title: &str,
    cron: &str,
    command: &str,
    description: Option<&str>,
    disabled: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let mut input = json!({
        "type": "COMMAND",
        "title": title,
        "cron": cron,
        "command": command,
        "enabled": !disabled,
    });

    if let Some(desc) = description {
        input["description"] = json!(desc);
    }

    let data = client.execute(graphql::CREATE_AUTOMATION, json!({ "input": input }))?;
    let result = data.get("createAutomation").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    title: Option<&str>,
    cron: Option<&str>,
    prompt: Option<&str>,
    description: Option<&str>,
    agent: Option<&str>,
    skills: Option<&[String]>,
    max_turns: Option<u32>,
    work_dir: Option<&str>,
    enabled: Option<bool>,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Automation");

    // Fetch current automation to merge with
    let data = client.execute(graphql::GET_AUTOMATIONS, json!({}))?;
    let automations = data
        .get("automations")
        .and_then(|a| a.as_array())
        .ok_or_else(|| CdxctlError::Parse("Failed to fetch automations".into()))?;

    let current = automations
        .iter()
        .find(|a| {
            a.get("id").and_then(|v| v.as_str()) == Some(&node_id)
                || a.get("uuid").and_then(|v| v.as_str()) == Some(id)
        })
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Automation not found: {id}")]))?;

    let auto_type = current
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("AGENT");

    let mut input = json!({
        "type": auto_type,
        "title": title.map(String::from).or_else(|| current.get("title").and_then(|v| v.as_str()).map(String::from)).unwrap_or_default(),
        "cron": cron.map(String::from).or_else(|| current.get("cron").and_then(|v| v.as_str()).map(String::from)).unwrap_or_default(),
        "enabled": json!(enabled.or_else(|| current.get("enabled").and_then(|v| v.as_bool())).unwrap_or(true)),
        "description": description.map(String::from).or_else(|| current.get("description").and_then(|v| v.as_str()).map(String::from)),
    });

    if auto_type == "COMMAND" {
        input["command"] = current.get("command").cloned().unwrap_or(Value::Null);
    } else {
        input["prompt"] = prompt
            .map(String::from)
            .or_else(|| {
                current
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .map(|s| json!(s))
            .unwrap_or(Value::Null);

        input["agentFullName"] = agent
            .map(String::from)
            .or_else(|| {
                current
                    .get("agentFullName")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .map(|s| json!(s))
            .unwrap_or(Value::Null);

        input["skillFullNames"] = match skills {
            Some(s) => json!(s),
            None => current
                .get("skillFullNames")
                .cloned()
                .unwrap_or(Value::Null),
        };

        input["maxTurns"] = json!(max_turns.or_else(|| current.get("maxTurns").and_then(|v| v.as_u64()).map(|v| v as u32)).unwrap_or(20));
    }

    input["workDir"] = work_dir
        .map(String::from)
        .or_else(|| {
            current
                .get("workDirPathInfo")
                .and_then(|p| p.get("absolutePath"))
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .map(|s| json!(s))
        .unwrap_or(Value::Null);

    let data = client.execute(
        graphql::UPDATE_AUTOMATION,
        json!({ "id": node_id, "input": input }),
    )?;
    let result = data.get("updateAutomation").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn delete(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Automation");
    let data = client.execute(graphql::DELETE_AUTOMATION, json!({ "id": node_id }))?;
    let result = data.get("deleteNode").cloned().unwrap_or(Value::Null);
    print_output(&json!({ "deleted": result }), format);
    Ok(())
}

pub fn trigger(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Automation");
    let data = client.execute(graphql::TRIGGER_AUTOMATION, json!({ "id": node_id }))?;
    let result = data
        .get("triggerAutomation")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}
