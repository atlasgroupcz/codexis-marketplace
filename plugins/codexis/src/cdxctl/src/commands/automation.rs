use super::{find_node_id, resolve_node_id};
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn list(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_AUTOMATIONS, json!({}))?;
    let mut automations = data
        .get("automations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for automation in automations.iter_mut() {
        flatten_automation(automation);
    }
    print_output(&Value::Array(automations), format);
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
        input["agentId"] = resolve_agent_id(client, a)?;
    }
    if !skills.is_empty() {
        input["skillIds"] = Value::Array(resolve_skill_ids(client, skills)?);
    }
    if let Some(turns) = max_turns {
        input["maxTurns"] = json!(turns);
    }
    if let Some(dir) = work_dir {
        input["workDir"] = json!(dir);
    }

    let data = client.execute(graphql::CREATE_AUTOMATION, json!({ "input": input }))?;
    let mut result = data.get("createAutomation").cloned().unwrap_or(Value::Null);
    flatten_automation(&mut result);
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
    let mut result = data.get("createAutomation").cloned().unwrap_or(Value::Null);
    flatten_automation(&mut result);
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

    let mut current = automations
        .iter()
        .find(|a| {
            a.get("id").and_then(|v| v.as_str()) == Some(&node_id)
                || a.get("uuid").and_then(|v| v.as_str()) == Some(id)
        })
        .cloned()
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Automation not found: {id}")]))?;
    flatten_automation(&mut current);

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

        input["agentId"] = match agent {
            Some(a) => resolve_agent_id(client, a)?,
            None => current.get("agentId").cloned().unwrap_or(Value::Null),
        };

        input["skillIds"] = match skills {
            Some(s) => Value::Array(resolve_skill_ids(client, s)?),
            None => Value::Array(
                current
                    .get("skillIds")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|v| !v.is_null())
                    .collect(),
            ),
        };

        input["maxTurns"] = json!(max_turns.or_else(|| current.get("maxTurns").and_then(|v| v.as_u64()).map(|v| v as u32)).unwrap_or(20));
    }

    input["workDir"] = work_dir
        .map(String::from)
        .or_else(|| {
            current
                .get("workDirPath")
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
    let mut result = data.get("updateAutomation").cloned().unwrap_or(Value::Null);
    flatten_automation(&mut result);
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

/// Flatten the daemon's nested `agent { id fullName }` / `skills { id fullName }`
/// selections into the flat `agentFullName` / `agentId` / `skillFullNames` /
/// `skillIds` keys the CLI output and the update-merge logic rely on.
fn flatten_automation(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if let Some(agent) = object.remove("agent") {
        object.insert(
            "agentFullName".into(),
            agent.get("fullName").cloned().unwrap_or(Value::Null),
        );
        object.insert(
            "agentId".into(),
            agent.get("id").cloned().unwrap_or(Value::Null),
        );
    }
    if let Some(skills) = object.remove("skills") {
        let skills = skills.as_array().cloned().unwrap_or_default();
        let full_names: Vec<Value> = skills
            .iter()
            .map(|skill| skill.get("fullName").cloned().unwrap_or(Value::Null))
            .collect();
        let ids: Vec<Value> = skills
            .iter()
            .map(|skill| skill.get("id").cloned().unwrap_or(Value::Null))
            .collect();
        object.insert("skillFullNames".into(), Value::Array(full_names));
        object.insert("skillIds".into(), Value::Array(ids));
    }
}

/// Resolve a `--agent` value (Node ID, full name, or bare name) to the Node `ID`
/// the daemon's `AutomationInput.agentId` expects.
fn resolve_agent_id(client: &GraphQLClient, value: &str) -> Result<Value, CdxctlError> {
    let data = client.execute(graphql::GET_AGENTS, json!({}))?;
    let agents = data
        .get("agents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    find_node_id(&agents, value, &["fullName", "name"])
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Agent not found: {value}")]))
}

/// Resolve each `--skill` value (Node ID, full name, or bare name) to the Node
/// `ID` the daemon's `AutomationInput.skillIds` expects.
fn resolve_skill_ids(client: &GraphQLClient, values: &[String]) -> Result<Vec<Value>, CdxctlError> {
    let data = client.execute(graphql::GET_SKILLS, json!({}))?;
    let skills = data
        .get("skills")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    values
        .iter()
        .map(|value| {
            find_node_id(&skills, value, &["fullName", "name"])
                .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Skill not found: {value}")]))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::flatten_automation;
    use serde_json::{json, Value};

    #[test]
    fn flatten_automation_lifts_agent_and_skill_names_and_ids() {
        let mut value = json!({
            "id": "Automation:x",
            "agent": { "id": "Agent:a", "fullName": "codexis:explorer" },
            "skills": [
                { "id": "Skill:s1", "fullName": "codexis:research" },
                { "id": "Skill:s2", "fullName": "writer" }
            ]
        });
        flatten_automation(&mut value);
        assert_eq!(value["agentFullName"], json!("codexis:explorer"));
        assert_eq!(value["agentId"], json!("Agent:a"));
        assert_eq!(value["skillFullNames"], json!(["codexis:research", "writer"]));
        assert_eq!(value["skillIds"], json!(["Skill:s1", "Skill:s2"]));
        assert!(value.get("agent").is_none());
        assert!(value.get("skills").is_none());
    }

    #[test]
    fn flatten_automation_handles_command_automations() {
        // COMMAND automations carry a null agent and an empty skills list.
        let mut value = json!({ "id": "Automation:c", "agent": Value::Null, "skills": [] });
        flatten_automation(&mut value);
        assert_eq!(value["agentFullName"], Value::Null);
        assert_eq!(value["agentId"], Value::Null);
        assert_eq!(value["skillFullNames"], json!([]));
        assert_eq!(value["skillIds"], json!([]));
    }
}
