use std::collections::HashSet;
use std::fmt::Write;

use serde_json::{Map, Value};

use crate::core::error::CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchSchemaKind {
    Input,
    Output,
}

impl SearchSchemaKind {
    pub(crate) fn flag_name(self) -> &'static str {
        match self {
            Self::Input => "--schema-input",
            Self::Output => "--schema-output",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

pub(crate) fn render_search_schema(
    source_code: &str,
    kind: SearchSchemaKind,
) -> Result<String, CliError> {
    let raw_bundle = load_search_schema_bundle(source_code, kind)?;
    let bundle: Value = serde_json::from_str(raw_bundle)
        .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;

    let root_name = bundle.get("root").and_then(Value::as_str).ok_or_else(|| {
        CliError::InvalidStoredSchema("schema bundle is missing root".to_string())
    })?;
    let components = bundle
        .get("components")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema("schema bundle is missing components".to_string())
        })?;
    let root_schema = components.get(root_name).ok_or_else(|| {
        CliError::InvalidStoredSchema(format!(
            "schema bundle is missing root component {root_name}"
        ))
    })?;

    let mut output = String::new();
    writeln!(output, "{source_code} search {} schema", kind.label()).unwrap();
    writeln!(output, "Schema: {root_name}").unwrap();

    if let Some(description) = schema_description(root_schema, components) {
        writeln!(output, "Description: {description}").unwrap();
    }
    if let Some(api_docs_url) = bundle.get("apiDocsUrl").and_then(Value::as_str) {
        writeln!(output, "API docs: {api_docs_url}").unwrap();
    }
    if let Some(fetched_at) = bundle.get("fetchedAt").and_then(Value::as_str) {
        writeln!(output, "Fetched: {fetched_at}").unwrap();
    }

    let required = required_properties(root_schema, components);
    if !required.is_empty() {
        writeln!(output, "Required top-level fields: {}", required.join(", ")).unwrap();
    }

    output.push_str("\nFields:\n");
    render_object_fields("", root_schema, components, &mut output)?;

    Ok(output.trim_end().to_string())
}

fn load_search_schema_bundle(
    source_code: &str,
    kind: SearchSchemaKind,
) -> Result<&'static str, CliError> {
    match (source_code, kind) {
        ("ALL", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/ALL/input.bundle.json"))
        }
        ("ALL", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/ALL/output.bundle.json"))
        }
        ("COMMENT", SearchSchemaKind::Input) => Ok(include_str!(
            "../../schemas/search/COMMENT/input.bundle.json"
        )),
        ("COMMENT", SearchSchemaKind::Output) => Ok(include_str!(
            "../../schemas/search/COMMENT/output.bundle.json"
        )),
        ("CR", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/CR/input.bundle.json"))
        }
        ("CR", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/CR/output.bundle.json"))
        }
        ("ES", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/ES/input.bundle.json"))
        }
        ("ES", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/ES/output.bundle.json"))
        }
        ("EU", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/EU/input.bundle.json"))
        }
        ("EU", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/EU/output.bundle.json"))
        }
        ("JD", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/JD/input.bundle.json"))
        }
        ("JD", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/JD/output.bundle.json"))
        }
        ("LT", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/LT/input.bundle.json"))
        }
        ("LT", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/LT/output.bundle.json"))
        }
        ("SK", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/SK/input.bundle.json"))
        }
        ("SK", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/SK/output.bundle.json"))
        }
        ("VS", SearchSchemaKind::Input) => {
            Ok(include_str!("../../schemas/search/VS/input.bundle.json"))
        }
        ("VS", SearchSchemaKind::Output) => {
            Ok(include_str!("../../schemas/search/VS/output.bundle.json"))
        }
        _ => Err(CliError::InvalidStoredSchema(format!(
            "no stored schema bundle for {source_code} {}",
            kind.label()
        ))),
    }
}

fn render_object_fields(
    prefix: &str,
    schema: &Value,
    components: &Map<String, Value>,
    output: &mut String,
) -> Result<(), CliError> {
    let resolved = resolve_schema(schema, components)?;
    let properties = schema_properties(resolved)
        .ok_or_else(|| CliError::InvalidStoredSchema("expected object schema".to_string()))?;
    let required = required_properties(resolved, components);

    for (name, value) in properties {
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}.{name}")
        };
        render_field(&path, value, required.contains(name), components, output)?;
    }

    Ok(())
}

fn render_field(
    path: &str,
    schema: &Value,
    required: bool,
    components: &Map<String, Value>,
    output: &mut String,
) -> Result<(), CliError> {
    writeln!(output, "{path}").unwrap();
    writeln!(output, "  Type: {}", type_summary(schema, components)?).unwrap();
    if required {
        output.push_str("  Required: yes\n");
    }
    if let Some(default_value) = schema_default(schema, components)? {
        writeln!(output, "  Default: {default_value}").unwrap();
    }
    if let Some(description) = schema_description(schema, components) {
        writeln!(output, "  Description: {description}").unwrap();
    }
    if let Some(examples) = schema_examples(schema, components)? {
        writeln!(output, "  Examples: {examples}").unwrap();
    }
    output.push('\n');

    let resolved = resolve_schema(schema, components)?;
    if is_object_schema(resolved) {
        render_object_fields(path, resolved, components, output)?;
    } else if let Some(items) = resolved.get("items") {
        let item_path = format!("{path}[]");
        let item_schema = resolve_schema(items, components)?;
        if is_object_schema(item_schema) {
            render_object_fields(&item_path, item_schema, components, output)?;
        }
    }

    Ok(())
}

fn type_summary(schema: &Value, components: &Map<String, Value>) -> Result<String, CliError> {
    if let Some(reference_name) = reference_name(schema) {
        let target = components.get(reference_name).ok_or_else(|| {
            CliError::InvalidStoredSchema(format!("missing referenced schema {reference_name}"))
        })?;
        if is_object_schema(target) {
            return Ok(reference_name.to_string());
        }
    }

    let resolved = resolve_schema_without_tracking(schema, components)?;
    if let Some(enum_values) = resolved.get("enum").and_then(Value::as_array) {
        let base_type = primitive_type_name(resolved).unwrap_or("string");
        let enum_values = enum_values
            .iter()
            .map(render_inline_value)
            .collect::<Vec<_>>()
            .join(", ");
        return Ok(format!("{base_type} enum[{enum_values}]"));
    }

    if let Some(type_name) = primitive_type_name(resolved) {
        if type_name == "array" {
            let item_summary = resolved
                .get("items")
                .map(|items| type_summary(items, components))
                .transpose()?
                .unwrap_or_else(|| "unknown".to_string());
            return Ok(format!("array<{item_summary}>"));
        }

        if let Some(format_name) = resolved.get("format").and_then(Value::as_str) {
            return Ok(format!("{type_name} ({format_name})"));
        }

        return Ok(type_name.to_string());
    }

    if is_object_schema(resolved) {
        return Ok("object".to_string());
    }

    Ok("unknown".to_string())
}

fn schema_description<'a>(schema: &'a Value, components: &'a Map<String, Value>) -> Option<String> {
    resolve_schema_without_tracking(schema, components)
        .ok()
        .and_then(|resolved| resolved.get("description"))
        .and_then(Value::as_str)
        .map(compact_text)
}

fn schema_default(
    schema: &Value,
    components: &Map<String, Value>,
) -> Result<Option<String>, CliError> {
    Ok(resolve_schema_without_tracking(schema, components)?
        .get("default")
        .map(render_inline_value))
}

fn schema_examples(
    schema: &Value,
    components: &Map<String, Value>,
) -> Result<Option<String>, CliError> {
    let resolved = resolve_schema_without_tracking(schema, components)?;
    if let Some(examples) = resolved.get("examples").and_then(Value::as_array) {
        let rendered = examples
            .iter()
            .map(render_inline_value)
            .collect::<Vec<_>>()
            .join("; ");
        if !rendered.is_empty() {
            return Ok(Some(rendered));
        }
    }

    if let Some(example) = resolved.get("example") {
        return Ok(Some(render_inline_value(example)));
    }

    Ok(None)
}

fn required_properties(schema: &Value, components: &Map<String, Value>) -> Vec<String> {
    resolve_schema_without_tracking(schema, components)
        .ok()
        .and_then(|resolved| resolved.get("required"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn schema_properties<'a>(schema: &'a Value) -> Option<&'a Map<String, Value>> {
    schema.get("properties").and_then(Value::as_object)
}

fn is_object_schema(schema: &Value) -> bool {
    matches!(primitive_type_name(schema), Some("object")) || schema.get("properties").is_some()
}

fn primitive_type_name(schema: &Value) -> Option<&str> {
    match schema.get("type") {
        Some(Value::String(type_name)) => Some(type_name.as_str()),
        _ => None,
    }
}

fn resolve_schema<'a>(
    schema: &'a Value,
    components: &'a Map<String, Value>,
) -> Result<&'a Value, CliError> {
    let mut visited_refs = HashSet::new();
    let mut current = schema;
    while let Some(reference_name) = reference_name(current) {
        if !visited_refs.insert(reference_name.to_string()) {
            return Err(CliError::InvalidStoredSchema(format!(
                "cyclic schema reference detected at {reference_name}"
            )));
        }
        current = components.get(reference_name).ok_or_else(|| {
            CliError::InvalidStoredSchema(format!("missing referenced schema {reference_name}"))
        })?;
    }
    Ok(current)
}

fn resolve_schema_without_tracking<'a>(
    schema: &'a Value,
    components: &'a Map<String, Value>,
) -> Result<&'a Value, CliError> {
    resolve_schema(schema, components)
}

fn reference_name(schema: &Value) -> Option<&str> {
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#/components/schemas/"))
}

fn render_inline_value(value: &Value) -> String {
    match value {
        Value::String(text) => compact_text(text),
        _ => compact_text(&value.to_string()),
    }
}

fn compact_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jd_input_schema_is_rendered_human_readably() {
        let output = render_search_schema("JD", SearchSchemaKind::Input).unwrap();

        assert!(output.contains("JD search input schema"));
        assert!(output.contains("Schema: JdSearchRequest"));
        assert!(output.contains("query"));
        assert!(output.contains("Type: string"));
        assert!(output.contains("sort"));
        assert!(output.contains("enum[CITEX, DATE, NAME, RELEVANCE]"));
    }

    #[test]
    fn jd_output_schema_expands_result_item_fields() {
        let output = render_search_schema("JD", SearchSchemaKind::Output).unwrap();

        assert!(output.contains("results"));
        assert!(output.contains("results[].docId"));
    }
}
