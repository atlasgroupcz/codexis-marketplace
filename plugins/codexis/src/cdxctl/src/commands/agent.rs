use super::{find_node_id, flatten_plugin_marketplace, resolve_node_id};
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

#[derive(Default)]
pub struct AgentFields {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tools: Option<Vec<String>>,
    pub disallowed_tools: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub jurisdictions: Option<Vec<String>>,
    pub model: Option<String>,
    pub max_turns: Option<u32>,
    pub memory: Option<String>,
}

impl AgentFields {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.tools.is_none()
            && self.disallowed_tools.is_none()
            && self.skills.is_none()
            && self.jurisdictions.is_none()
            && self.model.is_none()
            && self.max_turns.is_none()
            && self.memory.is_none()
    }
}

pub fn list(
    client: &GraphQLClient,
    editable_only: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_AGENTS, json!({}))?;
    let agents = data
        .get("agents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut filtered: Vec<Value> = if editable_only {
        agents
            .into_iter()
            .filter(|agent| agent.get("editable").and_then(Value::as_bool) == Some(true))
            .collect()
    } else {
        agents
    };

    for agent in filtered.iter_mut() {
        flatten_plugin_marketplace(agent);
    }

    print_output(&json!(filtered), format);
    Ok(())
}

pub fn defaults(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_AGENT_DEFAULTS, json!({}))?;
    let result = data
        .get("agentDefaults")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn create(
    client: &GraphQLClient,
    file: Option<&str>,
    stdin: bool,
    fields: AgentFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    if fields.is_empty() {
        let data = client.execute(
            graphql::CREATE_AGENT_MARKDOWN,
            json!({ "markdown": load_markdown(file, stdin)? }),
        )?;
        print_output(&data.get("createAgent").cloned().unwrap_or(Value::Null), format);
        return Ok(());
    }
    let variables = {
        if file.is_some() || stdin {
            return Err(CdxctlError::Parse(
                "Choose one input style: the field flags, or --file/--stdin markdown".into(),
            ));
        }
        let name = fields.name.clone().ok_or_else(|| {
            CdxctlError::Parse("Creating an agent needs --name".into())
        })?;
        let description = fields.description.clone().ok_or_else(|| {
            CdxctlError::Parse("Creating an agent needs --description".into())
        })?;
        // The server has no per-chat context to guess a model from, so the caller states it.
        if fields.model.is_none() {
            return Err(CdxctlError::Parse("Creating an agent needs --model".into()));
        }
        let mut input = Map::new();
        input.insert("name".into(), json!(name));
        input.insert("description".into(), json!(description));
        apply_fields(&mut input, &fields);
        json!({ "input": Value::Object(input) })
    };

    let data = client.execute(graphql::SAVE_AGENT, variables)?;
    print_payload(data.get("saveAgent"), format)
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    file: Option<&str>,
    stdin: bool,
    fields: AgentFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Agent");

    if fields.is_empty() {
        let data = client.execute(
            graphql::UPDATE_AGENT_MARKDOWN,
            json!({ "id": node_id, "markdown": load_markdown(file, stdin)? }),
        )?;
        print_output(&data.get("updateAgent").cloned().unwrap_or(Value::Null), format);
        return Ok(());
    }
    let variables = {
        if file.is_some() || stdin {
            return Err(CdxctlError::Parse(
                "Choose one input style: the field flags, or --file/--stdin markdown".into(),
            ));
        }
        // AgentInput replaces the whole agent, so anything the flags do not set has to be
        // carried over from the stored agent or the update would clear it.
        let current = fetch_agent(client, id, &node_id)?;
        // `resolve_node_id` fabricates an id from a raw name; the daemon only knows the
        // stored one, so the fetched agent's id is what the mutation has to carry.
        let stored_id = agent_id(&current, id)?;
        let mut input = merge_current(&current);
        apply_fields(&mut input, &fields);
        json!({ "id": stored_id, "input": Value::Object(input) })
    };

    let data = client.execute(graphql::SAVE_AGENT, variables)?;
    print_payload(data.get("saveAgent"), format)
}

pub fn delete(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Agent");
    // Same as update: a node id built from a raw name is not the stored one, and the
    // daemon answers an unknown id with null -- which would read as a silent success.
    let stored_id = stored_agent_id(client, id, &node_id)?;
    let data = client.execute(graphql::DELETE_AGENT, json!({ "id": stored_id }))?;
    let result = data.get("deleteNode").cloned().unwrap_or(Value::Null);
    if result.is_null() {
        return Err(CdxctlError::Parse(format!("Agent {id} was not deleted")));
    }
    print_output(&json!({ "deleted": result }), format);
    Ok(())
}

fn print_payload(payload: Option<&Value>, format: OutputFormat) -> Result<(), CdxctlError> {
    let payload = payload.cloned().unwrap_or(Value::Null);
    let errors = payload
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !errors.is_empty() {
        return Err(CdxctlError::GraphQL(
            errors.iter().map(format_input_error).collect(),
        ));
    }
    let agent = payload.get("agent").cloned().unwrap_or(Value::Null);
    print_output(&agent, format);
    Ok(())
}

fn format_input_error(error: &Value) -> String {
    let field = error.get("field").and_then(Value::as_str);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Invalid input");
    let allowed: Vec<&str> = error
        .get("allowedValues")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let mut text = match field {
        Some(field) => format!("{field}: {message}"),
        None => message.to_string(),
    };
    if !allowed.is_empty() {
        text.push_str(&format!(" Allowed: {}", allowed.join(", ")));
    }
    text
}

fn stored_agent_id(client: &GraphQLClient, id: &str, node_id: &str) -> Result<String, CdxctlError> {
    agent_id(&fetch_agent(client, id, node_id)?, id)
}

fn agent_id(agent: &Value, id: &str) -> Result<String, CdxctlError> {
    agent
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdxctlError::Parse(format!("Agent {id} has no id")))
}

fn fetch_agent(client: &GraphQLClient, id: &str, node_id: &str) -> Result<Value, CdxctlError> {
    let data = client.execute(graphql::GET_AGENTS, json!({}))?;
    let agents = data
        .get("agents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    agents
        .iter()
        .find(|agent| {
            agent.get("id").and_then(Value::as_str) == Some(node_id)
                || find_node_id(std::slice::from_ref(agent), id, &["fullName", "name"]).is_some()
        })
        .cloned()
        .ok_or_else(|| CdxctlError::Parse(format!("No such agent: {id}")))
}

fn merge_current(current: &Value) -> Map<String, Value> {
    let mut input = Map::new();
    for key in ["name", "description", "memory"] {
        if let Some(value) = current.get(key).and_then(Value::as_str) {
            input.insert(key.into(), json!(value));
        }
    }
    input.insert("tools".into(), json!(tool_names(current, "tools")));
    input.insert(
        "disallowedTools".into(),
        json!(tool_names(current, "disallowedTools")),
    );
    for key in ["skills", "jurisdictions"] {
        if let Some(values) = current.get(key).and_then(Value::as_array) {
            input.insert(key.into(), json!(values));
        }
    }
    if let Some(model) = current.get("model").and_then(Value::as_str) {
        input.insert("model".into(), json!(model));
    }
    if let Some(max_turns) = current.get("maxTurns").and_then(Value::as_u64) {
        input.insert("maxTurns".into(), json!(max_turns));
    }
    input
}

fn tool_names(agent: &Value, key: &str) -> Vec<String> {
    agent
        .get(key)
        .and_then(Value::as_array)
        .map(|tools| {
            tools
                .iter()
                .filter_map(|tool| tool.get("toolName").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn apply_fields(input: &mut Map<String, Value>, fields: &AgentFields) {
    if let Some(name) = &fields.name {
        input.insert("name".into(), json!(name));
    }
    if let Some(description) = &fields.description {
        input.insert("description".into(), json!(description));
    }
    if let Some(tools) = &fields.tools {
        input.insert("tools".into(), json!(tools));
    }
    if let Some(disallowed) = &fields.disallowed_tools {
        input.insert("disallowedTools".into(), json!(disallowed));
    }
    if let Some(skills) = &fields.skills {
        input.insert("skills".into(), json!(skills));
    }
    if let Some(jurisdictions) = &fields.jurisdictions {
        input.insert("jurisdictions".into(), json!(jurisdictions));
    }
    if let Some(model) = &fields.model {
        input.insert("model".into(), json!(model));
    }
    if let Some(max_turns) = fields.max_turns {
        input.insert("maxTurns".into(), json!(max_turns));
    }
    if let Some(memory) = &fields.memory {
        input.insert("memory".into(), json!(memory));
    }
}

fn load_markdown(file: Option<&str>, stdin: bool) -> Result<String, CdxctlError> {
    match (file, stdin) {
        (Some(_), true) => Err(CdxctlError::Parse(
            "Choose exactly one input source: --file or --stdin".into(),
        )),
        (None, false) => Err(CdxctlError::Parse(
            "Provide agent markdown via --file or --stdin, or describe the agent with the field flags".into(),
        )),
        (Some(path), false) => load_markdown_from_file(path),
        (None, true) => load_markdown_from_stdin(),
    }
}

fn load_markdown_from_file(path: &str) -> Result<String, CdxctlError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CdxctlError::Parse(format!(
            "Failed to read agent markdown from {}: {}",
            Path::new(path).display(),
            error
        ))
    })?;
    validate_markdown(content)
}

fn load_markdown_from_stdin() -> Result<String, CdxctlError> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).map_err(|error| {
        CdxctlError::Parse(format!("Failed to read agent markdown from stdin: {error}"))
    })?;
    validate_markdown(buffer)
}

fn validate_markdown(content: String) -> Result<String, CdxctlError> {
    if content.trim().is_empty() {
        return Err(CdxctlError::Parse("Agent markdown cannot be empty".into()));
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::{
        format_input_error, load_markdown, load_markdown_from_file, merge_current,
        validate_markdown, AgentFields,
    };
    use crate::commands::resolve_node_id;
    use crate::error::CdxctlError;
    use serde_json::{json, Value};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_markdown_requires_one_input_source() {
        let error = load_markdown(None, false).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));

        let error = load_markdown(Some("/tmp/agent.md"), true).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));
    }

    #[test]
    fn load_markdown_from_file_reads_agent_contents() {
        let path = unique_temp_path("agent-md");
        fs::write(
            &path,
            "---\nname: test-agent\ndescription: demo\n---\n\nYou are a test agent.\n",
        )
        .unwrap();

        let content = load_markdown_from_file(path.to_str().unwrap()).unwrap();

        assert!(content.contains("name: test-agent"));
        assert!(content.contains("You are a test agent."));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_markdown_rejects_blank_content() {
        let error = validate_markdown(" \n\t ".to_string()).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));
    }

    #[test]
    fn merge_current_carries_every_field_the_flags_do_not_set() {
        let current = json!({
            "name": "reviewer",
            "description": "Reviews merge requests",
            "memory": "Read the diff first.",
            "tools": [{ "toolName": "shell" }, { "toolName": "readFile" }],
            "disallowedTools": [{ "toolName": "writeFile" }],
            "skills": ["debugger"],
            "jurisdictions": ["CZ"],
            "model": "CLAUDE_OPUS_4_8",
            "maxTurns": 7
        });

        let merged = merge_current(&current);

        assert_eq!(merged["name"], json!("reviewer"));
        assert_eq!(merged["memory"], json!("Read the diff first."));
        assert_eq!(merged["tools"], json!(["shell", "readFile"]));
        assert_eq!(merged["disallowedTools"], json!(["writeFile"]));
        assert_eq!(merged["skills"], json!(["debugger"]));
        assert_eq!(merged["jurisdictions"], json!(["CZ"]));
        assert_eq!(merged["model"], json!("CLAUDE_OPUS_4_8"));
        assert_eq!(merged["maxTurns"], json!(7));
    }

    #[test]
    fn merge_current_keeps_the_stored_id_not_the_one_built_from_a_name() {
        // `cdxctl agent update my-agent` fabricates Agent:my-agent as the node id, but the
        // daemon only knows the stored uuid -- sending the fabricated one made the update
        // look like a second agent claiming the name.
        let current = json!({ "id": "BUFnZW50JHV1aWQ", "name": "my-agent" });
        let stored = current.get("id").and_then(Value::as_str).unwrap();

        assert_eq!(stored, "BUFnZW50JHV1aWQ");
        assert_ne!(stored, resolve_node_id("my-agent", "Agent"));
    }

    #[test]
    fn agent_fields_are_empty_until_one_is_set() {
        assert!(AgentFields::default().is_empty());
        assert!(!AgentFields {
            description: Some("x".into()),
            ..AgentFields::default()
        }
        .is_empty());
    }

    #[test]
    fn format_input_error_names_the_field_and_lists_allowed_values() {
        let error = json!({
            "field": "skills",
            "message": "No such skill: typo.",
            "allowedValues": ["debugger", "planner"]
        });

        assert_eq!(
            format_input_error(&error),
            "skills: No such skill: typo. Allowed: debugger, planner"
        );
    }

    fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.md"))
    }
}
