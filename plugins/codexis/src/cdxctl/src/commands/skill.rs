use super::resolve_node_id;
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

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

    let filtered: Vec<Value> = if editable_only {
        skills
            .into_iter()
            .filter(|skill| skill.get("editable").and_then(Value::as_bool) == Some(true))
            .collect()
    } else {
        skills
    };

    print_output(&json!(filtered), format);
    Ok(())
}

pub fn create(
    client: &GraphQLClient,
    file: Option<&str>,
    stdin: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let markdown = load_markdown(file, stdin)?;
    let data = client.execute(graphql::CREATE_SKILL, json!({ "markdown": markdown }))?;
    let result = data.get("createSkill").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    file: Option<&str>,
    stdin: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let markdown = load_markdown(file, stdin)?;
    let node_id = resolve_node_id(id, "Skill");
    let data = client.execute(
        graphql::UPDATE_SKILL,
        json!({ "id": node_id, "markdown": markdown }),
    )?;
    let result = data.get("updateSkill").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn delete(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let node_id = resolve_node_id(id, "Skill");
    let data = client.execute(graphql::DELETE_SKILL, json!({ "id": node_id }))?;
    let result = data.get("deleteNode").cloned().unwrap_or(Value::Null);
    print_output(&json!({ "deleted": result }), format);
    Ok(())
}

fn load_markdown(file: Option<&str>, stdin: bool) -> Result<String, CdxctlError> {
    match (file, stdin) {
        (Some(_), true) => Err(CdxctlError::Parse(
            "Choose exactly one input source: --file or --stdin".into(),
        )),
        (None, false) => Err(CdxctlError::Parse(
            "Provide skill markdown via --file or --stdin".into(),
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
    use super::{load_markdown, load_markdown_from_file, validate_markdown};
    use crate::error::CdxctlError;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_markdown_requires_one_input_source() {
        let error = load_markdown(None, false).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));

        let error = load_markdown(Some("/tmp/skill.md"), true).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));
    }

    #[test]
    fn load_markdown_from_file_reads_skill_contents() {
        let path = unique_temp_path("skill-md");
        fs::write(
            &path,
            "---\nname: test\ndescription: demo\n---\n\n# Instructions\n",
        )
        .unwrap();

        let content = load_markdown_from_file(path.to_str().unwrap()).unwrap();

        assert!(content.contains("name: test"));
        assert!(content.contains("# Instructions"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_markdown_rejects_blank_content() {
        let error = validate_markdown(" \n\t ".to_string()).unwrap_err();
        assert!(matches!(error, CdxctlError::Parse(_)));
    }

    fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.md"))
    }
}
