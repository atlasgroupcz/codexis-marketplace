use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Map, Value};

use crate::core::error::CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceSchemaKind {
    Meta,
    Text,
    Toc,
    Parts,
    Versions,
    Related,
    RelatedCounts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaSource {
    Czsb,
}

struct RenderedSchemaDocument {
    intro: String,
    schema: Option<Value>,
}

pub(crate) fn render_resource_schema(
    kind: ResourceSchemaKind,
    source: Option<SchemaSource>,
) -> Result<String, CliError> {
    if kind.is_source_aware() {
        render_source_aware_schema(kind, source)
    } else {
        render_shared_schema(kind)
    }
}

fn render_shared_schema(kind: ResourceSchemaKind) -> Result<String, CliError> {
    let raw_bundle = load_shared_schema_bundle(kind)?;
    let bundle: Value = serde_json::from_str(raw_bundle)
        .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;

    let operations = bundle
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema("resource bundle is missing operations".to_string())
        })?;

    let output_schema = resource_output_schema(kind, operations)?;
    let query_parameters = simplify_query_parameters(kind, operations);
    let patterns = resource_cdx_patterns(kind);
    let components = bundle
        .get("components")
        .and_then(Value::as_object)
        .and_then(|components| components.get("schemas"))
        .and_then(Value::as_object);

    let rendered = RenderedSchemaDocument {
        intro: render_resource_intro(
            kind,
            &patterns,
            &query_parameters,
            resource_output_description(kind),
            resource_extra_detail(kind),
            None,
        ),
        schema: Some(make_standalone_json_schema(output_schema, components)?),
    };

    format_rendered_document(&rendered)
}

fn render_source_aware_schema(
    kind: ResourceSchemaKind,
    source: Option<SchemaSource>,
) -> Result<String, CliError> {
    match source {
        Some(source) => render_typed_source_schema(kind, source),
        None => render_generic_source_schema(kind),
    }
}

fn render_generic_source_schema(kind: ResourceSchemaKind) -> Result<String, CliError> {
    let available_sources = kind.available_sources();
    let subcommands = available_sources
        .iter()
        .map(|source| format!("`cdx-cz-spp schema {} {}`", resource_endpoint_name(kind), source.source_code()))
        .collect::<Vec<_>>()
        .join(", ");

    let rendered = RenderedSchemaDocument {
        intro: render_resource_intro(
            kind,
            &resource_cdx_patterns(kind),
            &[],
            resource_output_description(kind),
            Some(format!(
                "The `/{endpoint}` response shape is source-specific, so call `schema {endpoint} <SOURCE>` to get the actual output schema.",
                endpoint = resource_endpoint_name(kind)
            )),
            Some(format!(
                "For a source-specific schema, run one of: {subcommands}."
            )),
        ),
        schema: None,
    };

    format_rendered_document(&rendered)
}

fn render_typed_source_schema(
    kind: ResourceSchemaKind,
    source: SchemaSource,
) -> Result<String, CliError> {
    // Validate that this source is available for this endpoint
    if !kind.available_sources().contains(&source) {
        return Err(CliError::InvalidStoredSchema(format!(
            "the /{} endpoint is not available for {} documents",
            resource_endpoint_name(kind),
            source.source_code()
        )));
    }

    let raw_bundle = load_source_aware_schema_bundle(kind, source)?;
    let bundle: Value = serde_json::from_str(raw_bundle)
        .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;

    let operations = bundle
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema("resource bundle is missing operations".to_string())
        })?;

    let output_schema = resource_output_schema(kind, operations)?;
    let query_parameters = simplify_query_parameters(kind, operations);
    let components = bundle
        .get("components")
        .and_then(Value::as_object)
        .and_then(|components| components.get("schemas"))
        .and_then(Value::as_object);

    let patterns = source_specific_cdx_patterns(kind);

    let rendered = RenderedSchemaDocument {
        intro: render_resource_intro(
            kind,
            &patterns,
            &query_parameters,
            &format!(
                "{} response for {} documents.",
                resource_endpoint_label(kind),
                source.source_code()
            ),
            None,
            None,
        ),
        schema: Some(make_standalone_json_schema(output_schema, components)?),
    };

    format_rendered_document(&rendered)
}

fn format_rendered_document(rendered: &RenderedSchemaDocument) -> Result<String, CliError> {
    match &rendered.schema {
        Some(schema) => {
            let schema_json = serde_json::to_string_pretty(schema)
                .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;
            Ok(format!("{}\n---\n{}", rendered.intro, schema_json))
        }
        None => Ok(rendered.intro.clone()),
    }
}

fn load_shared_schema_bundle(kind: ResourceSchemaKind) -> Result<&'static str, CliError> {
    match kind {
        ResourceSchemaKind::Text => Ok(include_str!("../../schemas/resource/text.bundle.json")),
        ResourceSchemaKind::RelatedCounts => Ok(include_str!(
            "../../schemas/resource/related-counts.bundle.json"
        )),
        _ => Err(CliError::InvalidStoredSchema(format!(
            "/{} is source-aware, not shared",
            resource_endpoint_name(kind)
        ))),
    }
}

fn load_source_aware_schema_bundle(
    kind: ResourceSchemaKind,
    source: SchemaSource,
) -> Result<&'static str, CliError> {
    match (kind, source) {
        (ResourceSchemaKind::Meta, SchemaSource::Czsb) => {
            Ok(include_str!("../../schemas/resource/meta-czsb.bundle.json"))
        }
        (ResourceSchemaKind::Parts, SchemaSource::Czsb) => {
            Ok(include_str!("../../schemas/resource/parts-czsb.bundle.json"))
        }
        (ResourceSchemaKind::Versions, SchemaSource::Czsb) => Ok(include_str!(
            "../../schemas/resource/versions-czsb.bundle.json"
        )),
        (ResourceSchemaKind::Toc, SchemaSource::Czsb) => {
            Ok(include_str!("../../schemas/resource/toc-czsb.bundle.json"))
        }
        (ResourceSchemaKind::Related, SchemaSource::Czsb) => Ok(include_str!(
            "../../schemas/resource/related-czsb.bundle.json"
        )),
        _ => Err(CliError::InvalidStoredSchema(format!(
            "/{} is not source-aware",
            resource_endpoint_name(kind)
        ))),
    }
}

fn resource_output_schema(
    kind: ResourceSchemaKind,
    operations: &[Value],
) -> Result<Value, CliError> {
    if matches!(kind, ResourceSchemaKind::Text) {
        return Ok(json!({
            "type": "string",
            "description": "Rendered markdown text. No JSON envelope."
        }));
    }

    for operation in operations {
        let Some(response) = operation
            .get("responses")
            .and_then(Value::as_object)
            .and_then(|responses| responses.get("200"))
        else {
            continue;
        };

        let Some(content) = response.get("content").and_then(Value::as_object) else {
            continue;
        };

        for content_type in ["application/json", "text/plain", "text/plain;charset=UTF-8"] {
            if let Some(schema) = content
                .get(content_type)
                .and_then(Value::as_object)
                .and_then(|entry| entry.get("schema"))
            {
                return Ok(schema.clone());
            }
        }
    }

    Err(CliError::InvalidStoredSchema(
        "resource bundle is missing a 200 response schema".to_string(),
    ))
}

fn make_standalone_json_schema(
    mut schema: Value,
    components: Option<&Map<String, Value>>,
) -> Result<Value, CliError> {
    rewrite_component_refs_to_defs(&mut schema);

    let root = schema.as_object_mut().ok_or_else(|| {
        CliError::InvalidStoredSchema("output schema must be a JSON object".to_string())
    })?;
    root.insert(
        "$schema".to_string(),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );

    let Some(components) = components else {
        return Ok(schema);
    };

    let referenced_names = referenced_component_closure(&schema, components)?;
    if referenced_names.is_empty() {
        return Ok(schema);
    }

    let mut defs = Map::new();
    for name in referenced_names {
        let mut component_schema = components.get(&name).cloned().ok_or_else(|| {
            CliError::InvalidStoredSchema(format!("missing referenced schema {name}"))
        })?;
        rewrite_component_refs_to_defs(&mut component_schema);
        defs.insert(name, component_schema);
    }

    if let Some(root) = schema.as_object_mut() {
        root.insert("$defs".to_string(), Value::Object(defs));
    }

    Ok(schema)
}

fn rewrite_component_refs_to_defs(schema: &mut Value) {
    match schema {
        Value::Object(object) => {
            if let Some(reference) = object.get_mut("$ref") {
                if let Some(reference) = reference.as_str() {
                    if let Some(name) = reference.strip_prefix("#/components/schemas/") {
                        *object.get_mut("$ref").expect("$ref just checked") =
                            Value::String(format!("#/$defs/{name}"));
                    }
                }
            }

            for value in object.values_mut() {
                rewrite_component_refs_to_defs(value);
            }
        }
        Value::Array(items) => {
            for item in items {
                rewrite_component_refs_to_defs(item);
            }
        }
        _ => {}
    }
}

fn referenced_component_closure(
    schema: &Value,
    components: &Map<String, Value>,
) -> Result<Vec<String>, CliError> {
    let mut names = BTreeSet::new();
    let mut queue = direct_referenced_schema_names(schema);

    while let Some(name) = queue.pop_first() {
        if !names.insert(name.clone()) {
            continue;
        }

        let component = components.get(&name).ok_or_else(|| {
            CliError::InvalidStoredSchema(format!("missing referenced schema {name}"))
        })?;
        for dep in direct_referenced_schema_names(component) {
            if !names.contains(&dep) {
                queue.insert(dep);
            }
        }
    }

    Ok(names.into_iter().collect())
}

fn direct_referenced_schema_names(schema: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_direct_referenced_schema_names(schema, &mut names);
    names
}

fn collect_direct_referenced_schema_names(schema: &Value, names: &mut BTreeSet<String>) {
    match schema {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                if let Some(name) = reference.strip_prefix("#/$defs/") {
                    names.insert(name.to_string());
                } else if let Some(name) = reference.strip_prefix("#/components/schemas/") {
                    names.insert(name.to_string());
                }
            }
            for value in object.values() {
                collect_direct_referenced_schema_names(value, names);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_direct_referenced_schema_names(item, names);
            }
        }
        _ => {}
    }
}

fn simplify_query_parameters(kind: ResourceSchemaKind, operations: &[Value]) -> Vec<String> {
    let mut unique = BTreeMap::<String, String>::new();

    for operation in operations {
        let Some(parameters) = operation.get("parameters").and_then(Value::as_array) else {
            continue;
        };

        for parameter in parameters {
            if parameter.get("in").and_then(Value::as_str) != Some("query") {
                continue;
            }

            let Some(name) = parameter.get("name").and_then(Value::as_str) else {
                continue;
            };

            let repeatable = parameter
                .get("schema")
                .and_then(Value::as_object)
                .and_then(|schema| schema.get("type"))
                .and_then(Value::as_str)
                == Some("array");
            let label = if repeatable {
                format!("{name} (repeatable)")
            } else {
                name.to_string()
            };
            let description = parameter
                .get("description")
                .and_then(Value::as_str)
                .map(compact_text)
                .unwrap_or_else(|| "No description.".to_string());
            let rendered = simplify_query_parameter_text(kind, name, &label, &description);
            unique.entry(label.clone()).or_insert(rendered);
        }
    }

    unique.into_values().collect()
}

fn simplify_query_parameter_text(
    kind: ResourceSchemaKind,
    name: &str,
    label: &str,
    description: &str,
) -> String {
    match (kind, name) {
        (ResourceSchemaKind::Text, "part") => {
            format!("{label}: optional TOC node ids; return only selected parts.")
        }
        _ => format!("{label}: {description}"),
    }
}

fn compact_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn resource_endpoint_name(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "meta",
        ResourceSchemaKind::Text => "text",
        ResourceSchemaKind::Toc => "toc",
        ResourceSchemaKind::Parts => "parts",
        ResourceSchemaKind::Versions => "versions",
        ResourceSchemaKind::Related => "related",
        ResourceSchemaKind::RelatedCounts => "related/counts",
    }
}

fn resource_endpoint_label(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "Metadata",
        ResourceSchemaKind::Text => "Text",
        ResourceSchemaKind::Toc => "Table of contents",
        ResourceSchemaKind::Parts => "Parts",
        ResourceSchemaKind::Versions => "Versions",
        ResourceSchemaKind::Related => "Related documents",
        ResourceSchemaKind::RelatedCounts => "Related counts",
    }
}

fn resource_output_format(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Text => "markdown",
        _ => "json",
    }
}

fn resource_output_description(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "Metadata for one document.",
        ResourceSchemaKind::Text => "Rendered document text.",
        ResourceSchemaKind::Toc => {
            "Table of contents with stable part ids and line ranges aligned with /text output."
        }
        ResourceSchemaKind::Parts => "Document parts with content.",
        ResourceSchemaKind::Versions => "Available document versions.",
        ResourceSchemaKind::Related => "Related documents plus paging metadata.",
        ResourceSchemaKind::RelatedCounts => {
            "Per-relation-type counts for the current related-document scope."
        }
    }
}

fn resource_extra_detail(kind: ResourceSchemaKind) -> Option<String> {
    match kind {
        ResourceSchemaKind::Text => Some(
            "This endpoint returns markdown text, so the schema below describes a string response body."
                .to_string(),
        ),
        _ => None,
    }
}

fn resource_cdx_patterns(kind: ResourceSchemaKind) -> Vec<String> {
    let endpoint = resource_endpoint_name(kind);
    vec![format!("cdx-cz-spp://doc/<DOC_ID>/{endpoint}")]
}

fn source_specific_cdx_patterns(kind: ResourceSchemaKind) -> Vec<String> {
    let endpoint = resource_endpoint_name(kind);
    vec![format!("cdx-cz-spp://doc/<DOC_ID>/{endpoint}")]
}

fn render_resource_intro(
    kind: ResourceSchemaKind,
    patterns: &[String],
    query_parameters: &[String],
    summary: &str,
    detail: Option<String>,
    follow_up: Option<String>,
) -> String {
    let mut lines = vec![
        format!("Endpoint: /{}", resource_endpoint_name(kind)),
        summary.to_string(),
        format!("Response format: {}.", resource_output_format(kind)),
    ];

    if let Some(detail) = detail {
        lines.push(detail);
    }

    lines.push(String::new());
    lines.push("Accepted cdx-cz-spp:// patterns:".to_string());
    for pattern in patterns {
        lines.push(format!("- {pattern}"));
    }

    lines.push(String::new());
    if query_parameters.is_empty() {
        lines.push("Query parameters: none.".to_string());
    } else {
        lines.push("Query parameters:".to_string());
        for parameter in query_parameters {
            lines.push(format!("- {parameter}"));
        }
    }

    if let Some(follow_up) = follow_up {
        lines.push(String::new());
        lines.push(follow_up);
    }

    lines.join("\n")
}

impl ResourceSchemaKind {
    fn is_source_aware(self) -> bool {
        matches!(
            self,
            Self::Meta | Self::Parts | Self::Versions | Self::Toc | Self::Related
        )
    }

    fn available_sources(self) -> Vec<SchemaSource> {
        match self {
            Self::Meta | Self::Parts | Self::Versions | Self::Toc | Self::Related => {
                vec![SchemaSource::Czsb]
            }
            _ => vec![],
        }
    }
}

impl SchemaSource {
    pub(crate) fn source_code(self) -> &'static str {
        match self {
            Self::Czsb => "CZSB",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_rendered_schema_document(output: &str) -> Option<(String, Value)> {
        output.split_once("\n---\n").map(|(intro, schema_json)| {
            let schema = serde_json::from_str(schema_json).expect("schema JSON must parse");
            (intro.to_string(), schema)
        })
    }

    #[test]
    fn generic_meta_schema_points_to_typed_subcommands() {
        let output = render_resource_schema(ResourceSchemaKind::Meta, None).unwrap();
        let intro = output.as_str();

        assert!(intro.contains("Endpoint: /meta"));
        assert!(intro.contains("cdx-cz-spp://doc/<DOC_ID>/meta"));
        assert!(intro.contains("cdx-cz-spp schema meta CZSB"));
        assert!(intro.contains("schema meta <SOURCE>"));
        assert!(split_rendered_schema_document(&output).is_none());
    }

    #[test]
    fn typed_meta_schema_renders_source_specific_bundle() {
        let output =
            render_resource_schema(ResourceSchemaKind::Meta, Some(SchemaSource::Czsb)).unwrap();
        let (intro, schema) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /meta"));
        assert!(intro.contains("CZSB"));
        assert!(intro.contains("cdx-cz-spp://doc/<DOC_ID>/meta"));
        assert!(schema.get("$schema").is_some());
    }

    #[test]
    fn text_schema_is_shared_and_marks_markdown_output() {
        let output = render_resource_schema(ResourceSchemaKind::Text, None).unwrap();
        let (intro, schema) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /text"));
        assert!(intro.contains("Response format: markdown."));
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn related_counts_schema_is_shared() {
        let output = render_resource_schema(ResourceSchemaKind::RelatedCounts, None).unwrap();
        let (intro, _schema) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /related/counts"));
    }

    #[test]
    fn toc_czsb_renders_successfully() {
        let output =
            render_resource_schema(ResourceSchemaKind::Toc, Some(SchemaSource::Czsb)).unwrap();
        let (intro, _) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /toc"));
        assert!(intro.contains("CZSB"));
    }

    #[test]
    fn versions_czsb_renders_successfully() {
        let output =
            render_resource_schema(ResourceSchemaKind::Versions, Some(SchemaSource::Czsb)).unwrap();
        let (intro, _) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /versions"));
        assert!(intro.contains("CZSB"));
    }

    #[test]
    fn generic_parts_schema_lists_subcommands() {
        let output = render_resource_schema(ResourceSchemaKind::Parts, None).unwrap();
        assert!(output.contains("cdx-cz-spp schema parts CZSB"));
    }

    #[test]
    fn czsb_meta_does_not_include_law_pattern() {
        let output =
            render_resource_schema(ResourceSchemaKind::Meta, Some(SchemaSource::Czsb)).unwrap();
        let (intro, _) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("cdx-cz-spp://doc/<DOC_ID>/meta"));
        // CZSB has no law route
        assert!(!intro.contains("law/"));
    }
}
