use super::{find_node_id, flatten_plugin_marketplace, resolve_node_id};
use crate::client::GraphQLClient;
use crate::commands::agent::{format_input_error, parse_localized};
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Structured `skill create`/`update` fields. When all are unset the command falls back to the
/// whole-markdown `--file`/`--stdin` path (deprecated `createSkill`/`updateSkill`).
#[derive(Default)]
pub struct SkillFields {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    /// Localized display names as raw `lang:text` entries, one per language.
    pub display_name: Vec<String>,
    /// Localized summaries as raw `lang:text` entries, one per language.
    pub summary: Vec<String>,
    pub jurisdictions: Option<Vec<String>>,
    pub jurisdiction_required: Option<bool>,
    pub allowed_tools: Option<Vec<String>>,
}

impl SkillFields {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.instructions.is_none()
            && self.display_name.is_empty()
            && self.summary.is_empty()
            && self.jurisdictions.is_none()
            && self.jurisdiction_required.is_none()
            && self.allowed_tools.is_none()
    }
}

pub fn list(
    client: &GraphQLClient,
    editable_only: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_SKILLS, json!({}))?;
    let skills = data
        .get("skills")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut filtered: Vec<Value> = if editable_only {
        skills
            .into_iter()
            .filter(|skill| skill.get("editable").and_then(Value::as_bool) == Some(true))
            .collect()
    } else {
        skills
    };

    for skill in filtered.iter_mut() {
        flatten_plugin_marketplace(skill);
    }

    print_output(&json!(filtered), format);
    Ok(())
}

pub fn create(
    client: &GraphQLClient,
    file: Option<&str>,
    stdin: bool,
    instructions_file: Option<&str>,
    fields: SkillFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
    if fields.is_empty() {
        let data = client.execute(
            graphql::CREATE_SKILL,
            json!({ "markdown": load_markdown(file, stdin)? }),
        )?;
        print_output(&data.get("createSkill").cloned().unwrap_or(Value::Null), format);
        return Ok(());
    }
    if file.is_some() || stdin {
        return Err(CdxctlError::Parse(
            "Choose one input style: the field flags, or --file/--stdin markdown".into(),
        ));
    }
    let name = fields
        .name
        .clone()
        .ok_or_else(|| CdxctlError::Parse("Creating a skill needs --name".into()))?;
    let description = fields
        .description
        .clone()
        .ok_or_else(|| CdxctlError::Parse("Creating a skill needs --description".into()))?;
    let mut input = Map::new();
    input.insert("name".into(), json!(name));
    input.insert("description".into(), json!(description));
    apply_fields(&mut input, &fields)?;
    save_skill(client, json!({ "input": Value::Object(input) }), format)
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    file: Option<&str>,
    stdin: bool,
    instructions_file: Option<&str>,
    fields: SkillFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
    let node_id = resolve_node_id(id, "Skill");

    if fields.is_empty() {
        let data = client.execute(
            graphql::UPDATE_SKILL,
            json!({ "id": node_id, "markdown": load_markdown(file, stdin)? }),
        )?;
        print_output(&data.get("updateSkill").cloned().unwrap_or(Value::Null), format);
        return Ok(());
    }
    if file.is_some() || stdin {
        return Err(CdxctlError::Parse(
            "Choose one input style: the field flags, or --file/--stdin markdown".into(),
        ));
    }
    // SkillInput replaces the whole skill; carry name/description/allowedTools from the stored
    // skill so a partial update does not clear them. The daemon keeps the body, i18n display
    // strings and jurisdictions from the stored skill for the fields the flags do not set.
    let current = fetch_skill(client, id, &node_id)?;
    let stored_id = skill_id(&current, id)?;
    let mut input = merge_current(&current);
    apply_fields(&mut input, &fields)?;
    save_skill(client, json!({ "id": stored_id, "input": Value::Object(input) }), format)
}

pub fn delete(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Skill");
    let data = client.execute(graphql::DELETE_SKILL, json!({ "id": node_id }))?;
    let result = data.get("deleteNode").cloned().unwrap_or(Value::Null);
    print_output(&json!({ "deleted": result }), format);
    Ok(())
}

fn apply_fields(input: &mut Map<String, Value>, fields: &SkillFields) -> Result<(), CdxctlError> {
    if let Some(name) = &fields.name {
        input.insert("name".into(), json!(name));
    }
    if let Some(description) = &fields.description {
        input.insert("description".into(), json!(description));
    }
    if let Some(instructions) = &fields.instructions {
        input.insert("instructions".into(), json!(instructions));
    }
    if let Some(jurisdictions) = &fields.jurisdictions {
        input.insert("jurisdictions".into(), json!(jurisdictions));
    }
    if let Some(required) = fields.jurisdiction_required {
        input.insert("jurisdictionRequired".into(), json!(required));
    }
    if let Some(allowed) = &fields.allowed_tools {
        input.insert("allowedTools".into(), json!(allowed));
    }
    if !fields.display_name.is_empty() {
        input.insert("displayName".into(), json!(parse_localized(&fields.display_name, "display-name")?));
    }
    if !fields.summary.is_empty() {
        input.insert("summary".into(), json!(parse_localized(&fields.summary, "summary")?));
    }
    Ok(())
}

fn merge_current(current: &Value) -> Map<String, Value> {
    let mut input = Map::new();
    for key in ["name", "description"] {
        if let Some(value) = current.get(key).and_then(Value::as_str) {
            input.insert(key.into(), json!(value));
        }
    }
    if let Some(tools) = current.get("allowedTools").and_then(Value::as_array) {
        input.insert("allowedTools".into(), json!(tools));
    }
    input
}

fn fetch_skill(client: &GraphQLClient, id: &str, node_id: &str) -> Result<Value, CdxctlError> {
    let data = client.execute(graphql::GET_SKILLS, json!({}))?;
    let skills = data
        .get("skills")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    skills
        .iter()
        .find(|skill| {
            skill.get("id").and_then(Value::as_str) == Some(node_id)
                || find_node_id(std::slice::from_ref(skill), id, &["fullName", "name"]).is_some()
        })
        .cloned()
        .ok_or_else(|| CdxctlError::Parse(format!("No such skill: {id}")))
}

fn skill_id(skill: &Value, id: &str) -> Result<String, CdxctlError> {
    skill
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdxctlError::Parse(format!("Skill {id} has no id")))
}

/// Runs the structured `saveSkill` mutation, teaching the Markdown fallback when the daemon is too
/// old for the structured input rather than failing opaquely.
fn save_skill(
    client: &GraphQLClient,
    variables: Value,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    match client.execute(graphql::SAVE_SKILL, variables) {
        Ok(data) => print_payload(data.get("saveSkill"), format),
        Err(error) if is_unsupported_save_skill(&error) => Err(structured_input_unsupported()),
        Err(error) => Err(error),
    }
}

fn is_unsupported_save_skill(error: &CdxctlError) -> bool {
    matches!(error, CdxctlError::GraphQL(messages) if messages.iter().any(|message| {
        let message = message.to_ascii_lowercase();
        (message.contains("saveskill") && (message.contains("undefined") || message.contains("unknown")))
            || (message.contains("skillinput") && message.contains("unknown"))
    }))
}

fn structured_input_unsupported() -> CdxctlError {
    CdxctlError::Parse(
        "This daemon does not support the structured skill input yet (the saveSkill mutation is \
         unavailable). Create the skill from Markdown instead: write a SKILL.md with YAML \
         frontmatter and run `cdxctl skill create --file <path>` (or pipe it with --stdin); update \
         an existing one with `cdxctl skill update <id> --file <path>`. Frontmatter keys: name, \
         description, i18n (displayName/summary per language), jurisdictions, jurisdiction-required, \
         allowed-tools; put the skill instructions in the body below the frontmatter."
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
    print_output(&payload.get("skill").cloned().unwrap_or(Value::Null), format);
    Ok(())
}

/// Folds `--instructions-file` into the structured fields: the file's contents become the skill
/// body, so a long skill can live in a temp file (e.g. under ./.tmp) instead of an inline arg.
fn merge_instructions_file(
    mut fields: SkillFields,
    instructions_file: Option<&str>,
) -> Result<SkillFields, CdxctlError> {
    let Some(path) = instructions_file else {
        return Ok(fields);
    };
    if fields.instructions.is_some() {
        return Err(CdxctlError::Parse(
            "Choose one source for the skill body: --instructions or --instructions-file, not both".into(),
        ));
    }
    let content = fs::read_to_string(path).map_err(|error| {
        CdxctlError::Parse(format!(
            "Failed to read skill body from {}: {}",
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
            "Provide skill markdown via --file or --stdin, or describe the skill with the field flags".into(),
        )),
        (Some(path), false) => load_markdown_from_file(path),
        (None, true) => load_markdown_from_stdin(),
    }
}

fn load_markdown_from_file(path: &str) -> Result<String, CdxctlError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CdxctlError::Parse(format!(
            "Failed to read skill markdown from {}: {}",
            Path::new(path).display(),
            error
        ))
    })?;
    validate_markdown(content)
}

fn load_markdown_from_stdin() -> Result<String, CdxctlError> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).map_err(|error| {
        CdxctlError::Parse(format!("Failed to read skill markdown from stdin: {error}"))
    })?;
    validate_markdown(buffer)
}

fn validate_markdown(content: String) -> Result<String, CdxctlError> {
    if content.trim().is_empty() {
        return Err(CdxctlError::Parse("Skill markdown cannot be empty".into()));
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::{apply_fields, is_unsupported_save_skill, load_markdown, merge_current, SkillFields};
    use crate::error::CdxctlError;
    use serde_json::{json, Map};

    #[test]
    fn load_markdown_requires_one_input_source() {
        assert!(matches!(load_markdown(None, false).unwrap_err(), CdxctlError::Parse(_)));
        assert!(matches!(load_markdown(Some("/tmp/skill.md"), true).unwrap_err(), CdxctlError::Parse(_)));
    }

    #[test]
    fn skill_fields_are_empty_until_one_is_set() {
        assert!(SkillFields::default().is_empty());
        assert!(!SkillFields { description: Some("x".into()), ..SkillFields::default() }.is_empty());
    }

    #[test]
    fn apply_fields_builds_the_structured_skill_input() {
        let fields = SkillFields {
            name: Some("vypoved".into()),
            description: Some("Use when reviewing a dismissal.".into()),
            instructions: Some("Posuď výpověď.".into()),
            display_name: vec!["cs:Kontrola výpovědi".into(), "en:Dismissal review".into()],
            summary: vec!["cs:Posoudí výpověď.".into()],
            jurisdictions: Some(vec!["CZ".into()]),
            jurisdiction_required: Some(true),
            allowed_tools: Some(vec!["shell".into(), "readFile".into()]),
        };
        let mut input = Map::new();

        apply_fields(&mut input, &fields).unwrap();

        assert_eq!(input["instructions"], json!("Posuď výpověď."));
        assert_eq!(input["jurisdictions"], json!(["CZ"]));
        assert_eq!(input["jurisdictionRequired"], json!(true));
        assert_eq!(input["allowedTools"], json!(["shell", "readFile"]));
        assert_eq!(
            input["displayName"],
            json!([{ "language": "cs", "text": "Kontrola výpovědi" }, { "language": "en", "text": "Dismissal review" }])
        );
        assert_eq!(input["summary"], json!([{ "language": "cs", "text": "Posoudí výpověď." }]));
    }

    #[test]
    fn merge_current_carries_name_description_and_allowed_tools() {
        let current = json!({
            "name": "reviewer",
            "description": "Reviews things",
            "allowedTools": ["shell", "readFile"]
        });
        let merged = merge_current(&current);
        assert_eq!(merged["name"], json!("reviewer"));
        assert_eq!(merged["description"], json!("Reviews things"));
        assert_eq!(merged["allowedTools"], json!(["shell", "readFile"]));
    }

    #[test]
    fn detects_a_daemon_without_the_save_skill_mutation() {
        let undefined = CdxctlError::GraphQL(vec![
            "Validation error (FieldUndefined@[saveSkill]) : Field 'saveSkill' in type 'Mutation' is undefined".into(),
        ]);
        assert!(is_unsupported_save_skill(&undefined));
        let rejected = CdxctlError::GraphQL(vec!["name: Skill name must use lowercase.".into()]);
        assert!(!is_unsupported_save_skill(&rejected));
    }
}
