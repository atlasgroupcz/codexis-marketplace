use super::{find_node_id, resolve_node_id};
use crate::client::GraphQLClient;
use crate::commands::schedule::ScheduleArgs;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use reqwest::blocking::multipart::Form;
use serde_json::{json, Map, Value};

/// Fields for creating an AGENT automation.
pub struct CreateArgs<'a> {
    pub title: &'a str,
    pub schedule: ScheduleArgs<'a>,
    pub prompt: &'a str,
    pub description: Option<&'a str>,
    pub agent: Option<&'a str>,
    pub skills: &'a [String],
    pub max_turns: Option<u32>,
    pub model: Option<&'a str>,
    pub jurisdictions: Option<&'a [String]>,
    pub project: Option<&'a str>,
    pub connections: &'a [String],
    pub files: &'a [String],
    pub disabled: bool,
}

/// Fields for a partial automation update. `None` fields keep their current value;
/// `files` are always added on top of the files the automation already stores.
pub struct UpdateArgs<'a> {
    pub id: &'a str,
    pub title: Option<&'a str>,
    pub schedule: ScheduleArgs<'a>,
    pub prompt: Option<&'a str>,
    pub description: Option<&'a str>,
    pub agent: Option<&'a str>,
    pub skills: Option<&'a [String]>,
    pub max_turns: Option<u32>,
    pub model: Option<&'a str>,
    pub jurisdictions: Option<&'a [String]>,
    pub project: Option<&'a str>,
    pub connections: Option<&'a [String]>,
    pub files: &'a [String],
}

const ATTACHMENTS_PATH: &str = "/rest/attachments";

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
    args: CreateArgs,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let CreateArgs {
        title,
        schedule,
        prompt,
        description,
        agent,
        skills,
        max_turns,
        model,
        jurisdictions,
        project,
        connections,
        files,
        disabled,
    } = args;
    let mut input = json!({
        "title": title,
        "schedule": require_schedule(&schedule)?,
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
    if let Some(m) = model {
        input["model"] = json!(m);
    }
    if let Some(list) = jurisdictions {
        input["jurisdictions"] = json!(list);
    }
    if let Some(p) = project {
        input["projectId"] = resolve_project_id(client, p)?;
    }
    if !connections.is_empty() {
        input["connectionIds"] = Value::Array(resolve_connection_ids(client, connections)?);
    }
    if !files.is_empty() {
        input["attachmentIds"] = Value::Array(upload_files(client, files)?);
    }
    if let Some(object) = input.as_object_mut() {
        apply_chat_context(client, object);
    }

    save_automation(client, json!({ "input": input }), format)
}

pub fn create_command(
    client: &GraphQLClient,
    title: &str,
    schedule: ScheduleArgs<'_>,
    command: &str,
    description: Option<&str>,
    disabled: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let mut input = json!({
        "type": "COMMAND",
        "title": title,
        "schedule": require_schedule(&schedule)?,
        "command": command,
        "enabled": !disabled,
    });

    if let Some(desc) = description {
        input["description"] = json!(desc);
    }

    save_automation(client, json!({ "input": input }), format)
}

pub fn update(
    client: &GraphQLClient,
    args: UpdateArgs,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(args.id, "Automation");

    // Fetch current automation to merge with
    let data = client.execute(graphql::GET_AUTOMATIONS, json!({}))?;
    let automations = data
        .get("automations")
        .and_then(|a| a.as_array())
        .ok_or_else(|| CdxctlError::Parse("Failed to fetch automations".into()))?;

    // Only `id`: no query in this CLI selects `uuid`. A raw UUID still resolves,
    // because `resolve_node_id` encodes it into the Node ID compared here.
    let current = automations
        .iter()
        .find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&node_id))
        .cloned()
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Automation not found: {}", args.id)]))?;

    let auto_type = current
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("AGENT")
        .to_string();

    // Only AGENT automations carry an agent and skills, so a COMMAND update must
    // not spend a round trip resolving names it will never send.
    let (agent_id, skill_ids) = if auto_type == "COMMAND" {
        (None, None)
    } else {
        (
            args.agent
                .map(|a| resolve_agent_id(client, a))
                .transpose()?,
            args.skills
                .map(|s| resolve_skill_ids(client, s))
                .transpose()?,
        )
    };

    let inputs = ResolvedInputs {
        agent_id,
        skill_ids,
        project_id: args
            .project
            .map(|p| resolve_project_id(client, p))
            .transpose()?,
        connection_ids: args
            .connections
            .map(|c| resolve_connection_ids(client, c))
            .transpose()?,
        attachment_ids: upload_files(client, args.files)?,
    };

    let input = build_update_input(current, &args, &auto_type, inputs)?;
    save_automation(client, json!({ "id": node_id, "input": input }), format)
}

/// Update flags that name daemon entities, already resolved to the Node `ID`s
/// (or, for files, uploaded to attachment ids) the mutation input expects.
#[derive(Default)]
struct ResolvedInputs {
    agent_id: Option<Value>,
    skill_ids: Option<Vec<Value>>,
    project_id: Option<Value>,
    connection_ids: Option<Vec<Value>>,
    attachment_ids: Vec<Value>,
}

/// Merges the update flags onto the automation as it is stored, producing the
/// `AutomationInput` payload. The resolved inputs arrive already turned into ids
/// because that needs the daemon; everything else is a pure merge.
fn build_update_input(
    mut current: Value,
    args: &UpdateArgs,
    auto_type: &str,
    inputs: ResolvedInputs,
) -> Result<Value, CdxctlError> {
    let ResolvedInputs {
        agent_id,
        skill_ids,
        project_id,
        connection_ids,
        attachment_ids,
    } = inputs;
    lift_nested_nodes(&mut current);

    let mut input = json!({
        "type": auto_type,
        "title": args.title.map(String::from).or_else(|| current.get("title").and_then(|v| v.as_str()).map(String::from)).unwrap_or_default(),
        "schedule": args.schedule.to_input()?.or_else(|| current.get("schedule").cloned()).unwrap_or(Value::Null),
        // `update` never changes the enabled flag - preserve the current value.
        // Toggling enabled goes through `enable`/`disable` (setAutomationEnabled),
        // which also works for read-only carried automations.
        "enabled": json!(current.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
        "description": args.description.map(String::from).or_else(|| current.get("description").and_then(|v| v.as_str()).map(String::from)),
    });

    if auto_type == "COMMAND" {
        input["command"] = current.get("command").cloned().unwrap_or(Value::Null);
    } else {
        input["prompt"] = args
            .prompt
            .map(String::from)
            .or_else(|| {
                current
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .map(|s| json!(s))
            .unwrap_or(Value::Null);

        input["agentId"] = match agent_id {
            Some(a) => a,
            None => current.get("agentId").cloned().unwrap_or(Value::Null),
        };

        input["skillIds"] = match skill_ids {
            Some(s) => Value::Array(s),
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

        input["maxTurns"] = json!(args
            .max_turns
            .or_else(|| current
                .get("maxTurns")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32))
            .unwrap_or(20));
    }

    input["model"] = match args.model {
        Some(m) => json!(m),
        None => current.get("model").cloned().unwrap_or(Value::Null),
    };
    input["jurisdictions"] = match args.jurisdictions {
        Some(list) => json!(list),
        None => current
            .get("jurisdictions")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    };

    input["modelConfiguration"] = current
        .get("modelConfiguration")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));

    input["projectId"] = match project_id {
        Some(p) => p,
        None => current.get("projectId").cloned().unwrap_or(Value::Null),
    };
    input["connectionIds"] = match connection_ids {
        Some(c) => Value::Array(c),
        None => current
            .get("connectionIds")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    };
    if !attachment_ids.is_empty() {
        input["attachmentIds"] = Value::Array(attachment_ids);
    }

    Ok(input)
}

/// Runs the structured `saveAutomation` mutation (id null → create, id set → update).
fn save_automation(
    client: &GraphQLClient,
    variables: Value,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::SAVE_AUTOMATION, variables)?;
    print_automation_payload(data.get("saveAutomation"), format)
}

/// Reads the `saveAutomation` payload: a non-empty `errors` list becomes a structured
/// `CdxctlError::GraphQL` (each `field: message` (+ allowed values), exactly like agents);
/// otherwise the `automation` is flattened and printed on the normal output path.
fn print_automation_payload(
    payload: Option<&Value>,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let payload = payload.cloned().unwrap_or(Value::Null);
    let errors = payload
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !errors.is_empty() {
        return Err(CdxctlError::GraphQL(
            errors
                .iter()
                .map(crate::commands::agent::format_input_error)
                .collect(),
        ));
    }
    let mut automation = payload.get("automation").cloned().unwrap_or(Value::Null);
    flatten_automation(&mut automation);
    print_output(&automation, format);
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

/// Enable or disable an automation via the daemon's dedicated `setAutomationEnabled`
/// mutation. Works for USER automations and for read-only carried (plugin/marketplace)
/// automations alike - the latter persist a per-user enabled override. This is the
/// only way to disable a shipped plugin automation; `update` cannot (USER-only guard).
pub fn set_enabled(
    client: &GraphQLClient,
    id: &str,
    enabled: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Automation");
    let data = client.execute(
        graphql::SET_AUTOMATION_ENABLED,
        json!({ "id": node_id, "enabled": enabled }),
    )?;
    let result = data
        .get("setAutomationEnabled")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

/// Prepares a fetched automation for display: lifts the nested nodes, then
/// renders the schedule as one cell. Output only - the update merge takes
/// `lift_nested_nodes` instead, because it needs the schedule as an object.
fn flatten_automation(value: &mut Value) {
    lift_nested_nodes(value);
    if let Some(object) = value.as_object_mut() {
        if let Some(schedule) = object.remove("schedule") {
            object.insert("schedule".into(), json!(format_schedule(&schedule)));
        }
    }
}

/// Flatten the daemon's nested `agent { id fullName }` / `skills { id fullName }` /
/// `project { id name }` / `connections { id name }` selections into the flat
/// `agentFullName` / `agentId` / `skillFullNames` / `skillIds` / `project` /
/// `projectId` / `connections` / `connectionIds` keys the CLI output and the
/// update-merge logic rely on.
fn lift_nested_nodes(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if let Some(project) = object.remove("project") {
        object.insert(
            "project".into(),
            project.get("name").cloned().unwrap_or(Value::Null),
        );
        object.insert(
            "projectId".into(),
            project.get("id").cloned().unwrap_or(Value::Null),
        );
    }
    if let Some(connections) = object.remove("connections") {
        let connections = connections.as_array().cloned().unwrap_or_default();
        let names: Vec<Value> = connections
            .iter()
            .map(|connection| connection.get("name").cloned().unwrap_or(Value::Null))
            .collect();
        let ids: Vec<Value> = connections
            .iter()
            .map(|connection| connection.get("id").cloned().unwrap_or(Value::Null))
            .collect();
        object.insert("connections".into(), Value::Array(names));
        object.insert("connectionIds".into(), Value::Array(ids));
    }
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

/// Renders the nested `AutomationSchedule` as one cell. Table output drops object
/// and array columns, so the schedule has to reach it as a scalar.
fn format_schedule(schedule: &Value) -> String {
    let Some(repeat) = schedule.get("repeat").and_then(Value::as_str) else {
        return "-".to_string();
    };
    let day_part = schedule
        .get("dayPart")
        .and_then(Value::as_str)
        .unwrap_or("-");
    match repeat {
        "WEEKLY" => {
            let days: Vec<String> = schedule
                .get("daysOfWeek")
                .and_then(Value::as_array)
                .map(|days| days.iter().map(|day| day.to_string()).collect())
                .unwrap_or_default();
            format!("WEEKLY {day_part} {}", days.join(","))
        }
        "MONTHLY" => {
            let day = schedule
                .get("dayOfMonth")
                .and_then(Value::as_u64)
                .map(|day| day.to_string())
                .unwrap_or_else(|| "-".to_string());
            format!("MONTHLY {day_part} day {day}")
        }
        other => format!("{other} {day_part}"),
    }
}

fn context_keys_to_fill(input: &Map<String, Value>) -> Vec<&'static str> {
    [
        "model",
        "modelConfiguration",
        "jurisdictions",
        "projectId",
        "connectionIds",
    ]
    .into_iter()
    .filter(|key| !input.contains_key(*key))
    .collect()
}

/// The chat's setup as `AutomationInput` keys: the run configuration is copied as is,
/// the project and the connections are lifted to the ids the input takes. Keys the
/// chat does not carry (no project, no connections) are left out.
fn chat_context_values(chat: &Map<String, Value>) -> Map<String, Value> {
    let mut values = Map::new();
    for key in ["model", "modelConfiguration", "jurisdictions"] {
        if let Some(value) = chat.get(key).filter(|value| !value.is_null()) {
            values.insert(key.into(), value.clone());
        }
    }
    if let Some(project_id) = chat
        .get("project")
        .and_then(|project| project.get("id"))
        .filter(|id| !id.is_null())
    {
        values.insert("projectId".into(), project_id.clone());
    }
    let connection_ids: Vec<Value> = chat
        .get("connections")
        .and_then(Value::as_array)
        .map(|connections| {
            connections
                .iter()
                .filter_map(|connection| connection.get("id").cloned())
                .collect()
        })
        .unwrap_or_default();
    if !connection_ids.is_empty() {
        values.insert("connectionIds".into(), Value::Array(connection_ids));
    }
    values
}

fn apply_chat_context(client: &GraphQLClient, input: &mut Map<String, Value>) {
    let missing = context_keys_to_fill(input);
    if missing.is_empty() {
        return;
    }
    let Some(chat) = crate::commands::agent::current_chat(client) else {
        return;
    };
    let context = chat_context_values(&chat);
    for key in missing {
        if let Some(value) = context.get(key) {
            input.insert(key.into(), value.clone());
        }
    }
}

fn require_schedule(schedule: &ScheduleArgs<'_>) -> Result<Value, CdxctlError> {
    schedule.to_input()?.ok_or_else(|| {
        CdxctlError::Parse(
            "a schedule is required: pass --repeat (with --day-part) or --cron".to_string(),
        )
    })
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

/// Resolve a `--project` value (Node ID or project name) to the Node `ID` the
/// daemon's `AutomationInput.projectId` expects.
fn resolve_project_id(client: &GraphQLClient, value: &str) -> Result<Value, CdxctlError> {
    let data = client.execute(graphql::GET_PROJECTS, json!({}))?;
    let projects = data
        .get("projects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    find_node_id(&projects, value, &["name"])
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Project not found: {value}")]))
}

/// Resolve each `--connection` value (Node ID or connection name) to the Node `ID`
/// the daemon's `AutomationInput.connectionIds` expects. Cloud drives, own drives
/// and drives shared with the user are all candidates.
fn resolve_connection_ids(
    client: &GraphQLClient,
    values: &[String],
) -> Result<Vec<Value>, CdxctlError> {
    let data = client.execute(graphql::GET_CONNECTIONS, json!({}))?;
    let connections: Vec<Value> = ["cloudDrives", "myDrives", "sharedWithMe"]
        .into_iter()
        .flat_map(|key| {
            data.get(key)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        })
        .collect();
    values
        .iter()
        .map(|value| {
            find_node_id(&connections, value, &["name"])
                .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Connection not found: {value}")]))
        })
        .collect()
}

/// Upload each `--file` from the VM to the daemon's attachment store and return the
/// attachment Node `ID`s `AutomationInput.attachmentIds` takes. The daemon moves them
/// into the automation's files when it saves.
fn upload_files(client: &GraphQLClient, paths: &[String]) -> Result<Vec<Value>, CdxctlError> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let mut form = Form::new();
    for path in paths {
        form = form.part(
            "files",
            crate::commands::notification_email::attachment_part(path)?,
        );
    }
    let uploaded = client.rest_post_multipart(ATTACHMENTS_PATH, form, &[])?;
    let ids: Vec<Value> = uploaded
        .as_array()
        .map(|attachments| {
            attachments
                .iter()
                .filter_map(|attachment| attachment.get("id").cloned())
                .collect()
        })
        .unwrap_or_default();
    if ids.len() != paths.len() {
        return Err(CdxctlError::Parse(format!(
            "uploaded {} of {} files; the daemon returned {uploaded}",
            ids.len(),
            paths.len()
        )));
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::{
        build_update_input, chat_context_values, context_keys_to_fill, flatten_automation,
        lift_nested_nodes, print_automation_payload, upload_files, ResolvedInputs, ScheduleArgs,
        UpdateArgs,
    };
    use crate::client::GraphQLClient;
    use crate::error::CdxctlError;
    use crate::output::OutputFormat;
    use crate::test_support::stub_daemon;
    use serde_json::{json, Map, Value};

    /// An automation as `GET_AUTOMATIONS` returns it, with the schedule nested.
    fn stored_automation() -> Value {
        json!({
            "id": "Automation:a",
            "type": "AGENT",
            "title": "Kontrola katastru",
            "prompt": "zkontroluj",
            "enabled": true,
            "schedule": {
                "repeat": "DAILY",
                "dayPart": "MORNING",
                "daysOfWeek": [],
                "dayOfMonth": null
            },
            "agent": null,
            "skills": []
        })
    }

    fn update_args(id: &str) -> UpdateArgs<'_> {
        UpdateArgs {
            id,
            title: None,
            schedule: ScheduleArgs::default(),
            prompt: None,
            description: None,
            agent: None,
            skills: None,
            max_turns: None,
            model: None,
            jurisdictions: None,
            project: None,
            connections: None,
            files: &[],
        }
    }

    /// An automation as `GET_AUTOMATIONS` returns it, with a project and a connection.
    fn stored_automation_with_inputs() -> Value {
        let mut value = stored_automation();
        value["project"] = json!({ "id": "Project:p", "name": "Audit" });
        value["connections"] = json!([{ "id": "Drive:d", "name": "Smlouvy" }]);
        value["files"] = json!(["brief.pdf"]);
        value
    }

    #[test]
    fn update_keeps_the_stored_project_and_connections() {
        let args = update_args("a");

        let input = build_update_input(
            stored_automation_with_inputs(),
            &args,
            "AGENT",
            ResolvedInputs::default(),
        )
        .unwrap();

        assert_eq!(input["projectId"], json!("Project:p"));
        assert_eq!(input["connectionIds"], json!(["Drive:d"]));
        assert!(input.get("attachmentIds").is_none());
    }

    #[test]
    fn update_replaces_project_and_connections_and_adds_files() {
        let args = update_args("a");
        let inputs = ResolvedInputs {
            project_id: Some(json!("Project:q")),
            connection_ids: Some(vec![]),
            attachment_ids: vec![json!("Attachment:f")],
            ..ResolvedInputs::default()
        };

        let input =
            build_update_input(stored_automation_with_inputs(), &args, "AGENT", inputs).unwrap();

        assert_eq!(input["projectId"], json!("Project:q"));
        assert_eq!(input["connectionIds"], json!([]));
        assert_eq!(input["attachmentIds"], json!(["Attachment:f"]));
    }

    #[test]
    fn chat_context_values_lift_project_and_connection_ids() {
        let chat: Map<String, Value> = json!({
            "model": "CLAUDE_SONNET_5",
            "modelConfiguration": [],
            "jurisdictions": ["CZ"],
            "project": { "id": "Project:p" },
            "connections": [{ "id": "Drive:d" }, { "id": "CloudDriveMount:g" }]
        })
        .as_object()
        .cloned()
        .unwrap();

        let values = chat_context_values(&chat);

        assert_eq!(values["projectId"], json!("Project:p"));
        assert_eq!(
            values["connectionIds"],
            json!(["Drive:d", "CloudDriveMount:g"])
        );
        assert_eq!(values["model"], json!("CLAUDE_SONNET_5"));
    }

    #[test]
    fn chat_context_values_skip_a_chat_without_project_or_connections() {
        let chat: Map<String, Value> = json!({
            "model": "CLAUDE_SONNET_5",
            "project": Value::Null,
            "connections": []
        })
        .as_object()
        .cloned()
        .unwrap();

        let values = chat_context_values(&chat);

        assert!(values.get("projectId").is_none());
        assert!(values.get("connectionIds").is_none());
    }

    #[test]
    fn flatten_automation_lifts_project_and_connections() {
        let mut value = stored_automation_with_inputs();

        flatten_automation(&mut value);

        assert_eq!(value["project"], json!("Audit"));
        assert_eq!(value["projectId"], json!("Project:p"));
        assert_eq!(value["connections"], json!(["Smlouvy"]));
        assert_eq!(value["connectionIds"], json!(["Drive:d"]));
        assert_eq!(value["files"], json!(["brief.pdf"]));
    }

    #[test]
    fn upload_files_posts_each_file_and_returns_the_attachment_ids() {
        let directory = std::env::temp_dir().join(format!("cdxctl-upload-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("brief.txt");
        std::fs::write(&path, "brief").unwrap();
        let (address, requests) = stub_daemon(
            "HTTP/1.1 201 Created",
            "[{\"id\":\"Attachment:f\",\"name\":\"brief.txt\",\"size\":5}]",
        );
        let client = GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".to_string(),
        );

        let ids = upload_files(&client, &[path.to_string_lossy().to_string()]).unwrap();

        assert_eq!(ids, vec![json!("Attachment:f")]);
        let request = requests.recv().unwrap();
        assert!(request.starts_with("POST /rest/attachments HTTP/1.1"));
        assert!(request
            .to_lowercase()
            .contains("authorization: bearer test-token"));
        assert!(request.contains("name=\"files\""));
        assert!(request.contains("filename=\"brief.txt\""));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn upload_files_rejects_a_short_daemon_answer() {
        let directory =
            std::env::temp_dir().join(format!("cdxctl-upload-short-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("brief.txt");
        std::fs::write(&path, "brief").unwrap();
        let (address, _requests) = stub_daemon("HTTP/1.1 201 Created", "[]");
        let client = GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".to_string(),
        );

        let error = upload_files(&client, &[path.to_string_lossy().to_string()]).unwrap_err();

        assert!(matches!(error, CdxctlError::Parse(_)));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn upload_files_skips_the_daemon_without_files() {
        let client = GraphQLClient::with_endpoint(
            "http://127.0.0.1:9/graphql".to_string(),
            "Bearer test-token".to_string(),
        );

        assert!(upload_files(&client, &[]).unwrap().is_empty());
    }

    #[test]
    fn lifting_nodes_leaves_the_schedule_object_alone() {
        // The write path lifts without rendering; only display collapses the
        // schedule, and merging an update on top of a rendered one fails.
        let mut value = stored_automation();

        lift_nested_nodes(&mut value);

        assert!(value["schedule"].is_object());
        assert_eq!(value["skillIds"], json!([]));
    }

    #[test]
    fn display_flattening_renders_the_schedule_as_one_cell() {
        let mut value = stored_automation();

        flatten_automation(&mut value);

        assert_eq!(value["schedule"], json!("DAILY MORNING"));
    }

    #[test]
    fn update_replaces_the_stored_schedule_when_schedule_flags_are_given() {
        let mut args = update_args("a");
        args.schedule.repeat = Some("WEEKLY");
        args.schedule.day_part = Some("EVENING");
        args.schedule.days_of_week = &[1, 4];

        let input = build_update_input(
            stored_automation(),
            &args,
            "AGENT",
            ResolvedInputs::default(),
        )
        .unwrap();

        assert_eq!(input["schedule"]["repeat"], json!("WEEKLY"));
        assert_eq!(input["schedule"]["dayPart"], json!("EVENING"));
        assert_eq!(input["schedule"]["daysOfWeek"], json!([1, 4]));
    }

    #[test]
    fn update_sends_the_stored_schedule_as_an_input_object() {
        // `AutomationScheduleInput` is a Map. Sending the table rendering of the
        // schedule instead fails the whole mutation, which is what changing any
        // other field used to do.
        let mut args = update_args("a");
        args.prompt = Some("novy prompt");

        let input = build_update_input(
            stored_automation(),
            &args,
            "AGENT",
            ResolvedInputs::default(),
        )
        .unwrap();

        assert_eq!(
            input["schedule"],
            json!({
                "repeat": "DAILY",
                "dayPart": "MORNING",
                "daysOfWeek": [],
                "dayOfMonth": null
            })
        );
    }

    #[test]
    fn context_keys_to_fill_lists_only_unset_keys() {
        let mut input = Map::new();
        assert_eq!(
            context_keys_to_fill(&input),
            vec![
                "model",
                "modelConfiguration",
                "jurisdictions",
                "projectId",
                "connectionIds"
            ]
        );

        input.insert("model".into(), json!("CLAUDE_SONNET_5"));
        assert_eq!(
            context_keys_to_fill(&input),
            vec![
                "modelConfiguration",
                "jurisdictions",
                "projectId",
                "connectionIds"
            ]
        );

        input.insert(
            "modelConfiguration".into(),
            json!([{"key": "effort", "value": "HIGH"}]),
        );
        input.insert("jurisdictions".into(), json!(["CZ"]));
        input.insert("projectId".into(), json!("Project:p"));
        input.insert("connectionIds".into(), json!([]));
        assert!(context_keys_to_fill(&input).is_empty());
    }

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
        assert_eq!(
            value["skillFullNames"],
            json!(["codexis:research", "writer"])
        );
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

    #[test]
    fn print_automation_payload_renders_input_errors_as_structured_graphql() {
        // A saveAutomation payload whose `errors` are non-empty must surface as a
        // structured GraphQL error, each formatted `field: message` (+ allowed values).
        let payload = json!({
            "automation": Value::Null,
            "errors": [
                { "field": "model", "message": "Unknown model.", "allowedValues": ["gpt-5.6-terra", "claude-opus-4-8"] },
                { "field": "skillIds", "message": "No such skill: typo." }
            ]
        });

        let error = print_automation_payload(Some(&payload), OutputFormat::Json).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert_eq!(
                    messages,
                    vec![
                        "model: Unknown model. Allowed: gpt-5.6-terra, claude-opus-4-8".to_string(),
                        "skillIds: No such skill: typo.".to_string(),
                    ]
                );
            }
            other => panic!("expected GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn print_automation_payload_prints_automation_when_no_errors() {
        // No errors -> the automation flows through the normal flatten/output path
        // (Ok result), with `agent`/`skills` lifted into the flat CLI keys.
        let payload = json!({
            "automation": {
                "id": "Automation:x",
                "title": "Daily digest",
                "agent": { "id": "Agent:a", "fullName": "codexis:explorer" },
                "skills": []
            },
            "errors": []
        });

        assert!(print_automation_payload(Some(&payload), OutputFormat::Json).is_ok());
    }
}
