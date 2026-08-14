use super::{find_node_id, resolve_node_id};
use crate::client::GraphQLClient;
use crate::commands::schedule::ScheduleArgs;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
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
    pub work_dir: Option<&'a str>,
    pub disabled: bool,
}

/// Fields for a partial automation update. `None` fields keep their current value.
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
    pub work_dir: Option<&'a str>,
}

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
        work_dir,
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
    if let Some(dir) = work_dir {
        input["workDir"] = json!(dir);
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

    let current = automations
        .iter()
        .find(|a| {
            a.get("id").and_then(|v| v.as_str()) == Some(&node_id)
                || a.get("uuid").and_then(|v| v.as_str()) == Some(args.id)
        })
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

    let input = build_update_input(current, &args, &auto_type, agent_id, skill_ids)?;
    save_automation(client, json!({ "id": node_id, "input": input }), format)
}

/// Merges the update flags onto the automation as it is stored, producing the
/// `AutomationInput` payload. `agent_id` / `skill_ids` arrive already resolved
/// because that needs the daemon; everything else is a pure merge.
fn build_update_input(
    mut current: Value,
    args: &UpdateArgs,
    auto_type: &str,
    agent_id: Option<Value>,
    skill_ids: Option<Vec<Value>>,
) -> Result<Value, CdxctlError> {
    lift_agent_and_skills(&mut current);

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

    input["workDir"] = args
        .work_dir
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
/// `lift_agent_and_skills` instead, because it needs the schedule as an object.
fn flatten_automation(value: &mut Value) {
    lift_agent_and_skills(value);
    if let Some(object) = value.as_object_mut() {
        if let Some(schedule) = object.remove("schedule") {
            object.insert("schedule".into(), json!(format_schedule(&schedule)));
        }
    }
}

/// Flatten the daemon's nested `agent { id fullName }` / `skills { id fullName }`
/// selections into the flat `agentFullName` / `agentId` / `skillFullNames` /
/// `skillIds` keys the CLI output and the update-merge logic rely on.
fn lift_agent_and_skills(value: &mut Value) {
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
    ["model", "modelConfiguration", "jurisdictions"]
        .into_iter()
        .filter(|key| !input.contains_key(*key))
        .collect()
}

fn apply_chat_context(client: &GraphQLClient, input: &mut Map<String, Value>) {
    let missing = context_keys_to_fill(input);
    if missing.is_empty() {
        return;
    }
    let Some(chat) = crate::commands::agent::current_chat(client) else {
        return;
    };
    for key in missing {
        match chat.get(key) {
            Some(value) if !value.is_null() => {
                input.insert(key.into(), value.clone());
            }
            _ => {}
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

#[cfg(test)]
mod tests {
    use super::{
        build_update_input, context_keys_to_fill, flatten_automation, lift_agent_and_skills,
        print_automation_payload, ScheduleArgs, UpdateArgs,
    };
    use crate::error::CdxctlError;
    use crate::output::OutputFormat;
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
            work_dir: None,
        }
    }

    #[test]
    fn lifting_nodes_leaves_the_schedule_object_alone() {
        // The write path lifts without rendering; only display collapses the
        // schedule, and merging an update on top of a rendered one fails.
        let mut value = stored_automation();

        lift_agent_and_skills(&mut value);

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

        let input = build_update_input(stored_automation(), &args, "AGENT", None, None).unwrap();

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

        let input = build_update_input(stored_automation(), &args, "AGENT", None, None).unwrap();

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
            vec!["model", "modelConfiguration", "jurisdictions"]
        );

        input.insert("model".into(), json!("CLAUDE_SONNET_5"));
        assert_eq!(
            context_keys_to_fill(&input),
            vec!["modelConfiguration", "jurisdictions"]
        );

        input.insert(
            "modelConfiguration".into(),
            json!([{"key": "effort", "value": "HIGH"}]),
        );
        input.insert("jurisdictions".into(), json!(["CZ"]));
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
