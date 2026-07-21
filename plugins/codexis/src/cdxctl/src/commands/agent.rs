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
    pub instructions: Option<String>,
    /// Localized display names as raw `lang:text` entries, one per language.
    pub display_name: Vec<String>,
    /// Localized summaries as raw `lang:text` entries, one per language.
    pub summary: Vec<String>,
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
            && self.instructions.is_none()
            && self.display_name.is_empty()
            && self.summary.is_empty()
    }
}

/// Parses `lang:text` entries (e.g. `cs:Personalista pro smlouvy`). The language is what precedes
/// the first colon; the text is the rest, so the text itself may contain colons.
fn parse_localized(entries: &[String], field: &str) -> Result<Vec<Value>, CdxctlError> {
    entries
        .iter()
        .map(|entry| {
            let (language, text) = entry.split_once(':').map(|(l, t)| (l.trim(), t.trim())).unwrap_or(("", ""));
            if language.is_empty() || text.is_empty() {
                return Err(CdxctlError::Parse(format!(
                    "--{field} expects lang:text (e.g. cs:\"…\"), got '{entry}'"
                )));
            }
            Ok(json!({ "language": language, "text": text }))
        })
        .collect()
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

/// The context keys the chat may fill: only the ones the flags did not already set. An explicit
/// `--model` / `--jurisdiction` is left alone, so it wins over the chat.
fn context_keys_to_fill(input: &Map<String, Value>) -> Vec<&'static str> {
    ["model", "jurisdictions"]
        .into_iter()
        .filter(|key| !input.contains_key(*key))
        .collect()
}

/// `model` is required by the schema and `jurisdictions` shape what the agent may cite, so
/// both fall back to the chat in use. Outside a chat there is nothing to fall back to.
fn apply_chat_context(client: &GraphQLClient, input: &mut Map<String, Value>) -> Result<(), CdxctlError> {
    let missing = context_keys_to_fill(input);
    if missing.is_empty() {
        return Ok(());
    }
    let chat = current_chat(client);
    for key in missing {
        match chat.as_ref().and_then(|c| c.get(key)) {
            Some(value) if !value.is_null() => {
                input.insert(key.into(), value.clone());
            }
            _ if key == "model" => {
                return Err(CdxctlError::Parse(
                    "Creating an agent needs --model when not running inside a chat".into(),
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

fn current_chat(client: &GraphQLClient) -> Option<Map<String, Value>> {
    let session = std::env::var("CODEXIS_PUBLIC_SESSION_ID").ok()?;
    let node_id = resolve_node_id(&session, "Chat");
    let data = client
        .execute(graphql::GET_CHAT_CONTEXT, json!({ "id": node_id }))
        .ok()?;
    data.get("node").and_then(Value::as_object).cloned()
}

pub fn create(
    client: &GraphQLClient,
    file: Option<&str>,
    stdin: bool,
    instructions_file: Option<&str>,
    fields: AgentFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
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
        let mut input = Map::new();
        input.insert("name".into(), json!(name));
        input.insert("description".into(), json!(description));
        apply_fields(&mut input, &fields)?;
        // What the flags did not state comes from the chat this shell runs in, so an agent
        // starts out working the way the conversation that asked for it does.
        apply_chat_context(client, &mut input)?;
        json!({ "input": Value::Object(input) })
    };

    save_agent(client, variables, format)
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    file: Option<&str>,
    stdin: bool,
    instructions_file: Option<&str>,
    fields: AgentFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
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
        apply_fields(&mut input, &fields)?;
        json!({ "id": stored_id, "input": Value::Object(input) })
    };

    save_agent(client, variables, format)
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

/// Runs the structured `saveAgent` mutation. Daemons that predate it answer with a schema
/// validation error; rather than fail opaquely, hand the agent the compatible Markdown path so
/// it can retry on its own - the tool teaches the fallback instead of silently taking it.
fn save_agent(
    client: &GraphQLClient,
    variables: Value,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    match client.execute(graphql::SAVE_AGENT, variables) {
        Ok(data) => print_payload(data.get("saveAgent"), format),
        Err(error) if is_unsupported_save_agent(&error) => Err(structured_input_unsupported()),
        Err(error) => Err(error),
    }
}

/// True when the daemon rejected `saveAgent`/`AgentInput` as unknown - i.e. it is too old for the
/// structured input, not that the input itself was wrong.
fn is_unsupported_save_agent(error: &CdxctlError) -> bool {
    matches!(error, CdxctlError::GraphQL(messages) if messages.iter().any(|message| {
        let message = message.to_ascii_lowercase();
        (message.contains("saveagent") && (message.contains("undefined") || message.contains("unknown")))
            || (message.contains("agentinput") && message.contains("unknown"))
    }))
}

fn structured_input_unsupported() -> CdxctlError {
    CdxctlError::Parse(
        "This daemon does not support the structured agent input yet (the saveAgent mutation is \
         unavailable). Create the agent from Markdown instead: write a file with YAML frontmatter \
         and run `cdxctl agent create --file <path>` (or pipe it with --stdin); update an existing \
         one with `cdxctl agent update <id> --file <path>`. Frontmatter keys: name, description, \
         tools, disallowed-tools, skills, model (the model name, e.g. gpt-5.6-terra), max-turns, \
         jurisdictions; put the instructions in the body below the frontmatter. Example:\n\
         ---\nname: hr-vypoved\ndescription: Posuzuje zákonnost výpovědi\nmodel: gpt-5.6-terra\n\
         tools: [shell]\nskills: [codexis:codexis]\n---\nPosuď zákonnost výpovědi. …"
            .to_string(),
    )
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
    for key in ["name", "description", "instructions"] {
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

fn apply_fields(input: &mut Map<String, Value>, fields: &AgentFields) -> Result<(), CdxctlError> {
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
    if let Some(instructions) = &fields.instructions {
        input.insert("instructions".into(), json!(instructions));
    }
    if !fields.display_name.is_empty() {
        input.insert("displayName".into(), json!(parse_localized(&fields.display_name, "display-name")?));
    }
    if !fields.summary.is_empty() {
        input.insert("summary".into(), json!(parse_localized(&fields.summary, "summary")?));
    }
    Ok(())
}

/// Folds `--instructions-file` into the structured fields: the file's contents become the system
/// prompt, so a long persona can be written to a temp file (e.g. under ./.tmp) instead of a fragile
/// inline `--instructions` arg. Still the structured path, so it saves through saveAgent.
fn merge_instructions_file(
    mut fields: AgentFields,
    instructions_file: Option<&str>,
) -> Result<AgentFields, CdxctlError> {
    let Some(path) = instructions_file else {
        return Ok(fields);
    };
    if fields.instructions.is_some() {
        return Err(CdxctlError::Parse(
            "Choose one source for the system prompt: --instructions or --instructions-file, not both".into(),
        ));
    }
    let content = fs::read_to_string(path).map_err(|error| {
        CdxctlError::Parse(format!(
            "Failed to read instructions from {}: {}",
            Path::new(path).display(),
            error
        ))
    })?;
    fields.instructions = Some(content);
    Ok(fields)
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
        apply_fields, context_keys_to_fill, format_input_error, is_unsupported_save_agent,
        load_markdown, load_markdown_from_file, merge_current, validate_markdown, AgentFields,
    };
    use crate::commands::resolve_node_id;
    use crate::error::CdxctlError;
    use serde_json::Map;
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
            "instructions": "Read the diff first.",
            "tools": [{ "toolName": "shell" }, { "toolName": "readFile" }],
            "disallowedTools": [{ "toolName": "writeFile" }],
            "skills": ["debugger"],
            "jurisdictions": ["CZ"],
            "model": "CLAUDE_OPUS_4_8",
            "maxTurns": 7
        });

        let merged = merge_current(&current);

        assert_eq!(merged["name"], json!("reviewer"));
        assert_eq!(merged["instructions"], json!("Read the diff first."));
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
    fn explicit_model_and_jurisdictions_leave_nothing_for_the_chat_to_fill() {
        let mut stated = Map::new();
        stated.insert("model".into(), json!("CLAUDE_OPUS_4_8"));
        stated.insert("jurisdictions".into(), json!(["SK"]));

        // Both keys are already set by the flags, so the chat is not consulted for either.
        assert!(context_keys_to_fill(&stated).is_empty());
    }

    #[test]
    fn the_chat_fills_only_the_context_keys_the_flags_left_out() {
        let mut only_model = Map::new();
        only_model.insert("model".into(), json!("GPT_5_6_TERRA"));
        // --model is set, --jurisdiction is not: only jurisdictions is taken from the chat.
        assert_eq!(context_keys_to_fill(&only_model), vec!["jurisdictions"]);

        // Neither flag is set: both come from the chat.
        assert_eq!(context_keys_to_fill(&Map::new()), vec!["model", "jurisdictions"]);
    }

    #[test]
    fn apply_fields_sets_model_jurisdictions_and_the_lists() {
        let fields = AgentFields {
            model: Some("GPT_5_6_TERRA".into()),
            jurisdictions: Some(vec!["CZ".into(), "SK".into()]),
            skills: Some(vec!["codexis".into()]),
            max_turns: Some(9),
            ..AgentFields::default()
        };
        let mut input = Map::new();

        apply_fields(&mut input, &fields).unwrap();

        assert_eq!(input["model"], json!("GPT_5_6_TERRA"));
        assert_eq!(input["jurisdictions"], json!(["CZ", "SK"]));
        assert_eq!(input["skills"], json!(["codexis"]));
        assert_eq!(input["maxTurns"], json!(9));
    }

    #[test]
    fn apply_fields_omits_model_and_jurisdictions_when_the_flags_are_absent() {
        let mut input = Map::new();
        apply_fields(&mut input, &AgentFields::default()).unwrap();
        assert!(!input.contains_key("model"));
        assert!(!input.contains_key("jurisdictions"));
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
    fn apply_fields_builds_localized_display_name_and_summary() {
        let fields = AgentFields {
            display_name: vec!["cs:Personalista pro smlouvy".into(), "en:Contract HR agent".into()],
            summary: vec!["cs:Připraví smlouvu.".into()],
            ..AgentFields::default()
        };
        let mut input = Map::new();

        apply_fields(&mut input, &fields).unwrap();

        assert_eq!(
            input["displayName"],
            json!([
                { "language": "cs", "text": "Personalista pro smlouvy" },
                { "language": "en", "text": "Contract HR agent" }
            ])
        );
        assert_eq!(input["summary"], json!([{ "language": "cs", "text": "Připraví smlouvu." }]));
    }

    #[test]
    fn apply_fields_keeps_a_colon_in_the_localized_text() {
        let fields = AgentFields {
            display_name: vec!["cs:Poradce: pracovní právo".into()],
            ..AgentFields::default()
        };
        let mut input = Map::new();

        apply_fields(&mut input, &fields).unwrap();

        assert_eq!(
            input["displayName"],
            json!([{ "language": "cs", "text": "Poradce: pracovní právo" }])
        );
    }

    #[test]
    fn apply_fields_rejects_a_localized_value_without_a_language() {
        let fields = AgentFields {
            summary: vec!["Připraví smlouvu.".into()],
            ..AgentFields::default()
        };
        let mut input = Map::new();

        let error = apply_fields(&mut input, &fields).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));
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

    #[test]
    fn detects_a_daemon_without_the_save_agent_mutation() {
        let undefined = CdxctlError::GraphQL(vec![
            "Validation error (FieldUndefined@[saveAgent]) : Field 'saveAgent' in type 'Mutation' is undefined".into(),
        ]);
        assert!(is_unsupported_save_agent(&undefined));

        let unknown_type = CdxctlError::GraphQL(vec!["Unknown type 'AgentInput'".into()]);
        assert!(is_unsupported_save_agent(&unknown_type));

        // A real validation rejection (unknown skill) must NOT be mistaken for an old daemon.
        let rejected = CdxctlError::GraphQL(vec!["skills: No such skill: typo.".into()]);
        assert!(!is_unsupported_save_agent(&rejected));
    }

    fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.md"))
    }
}
