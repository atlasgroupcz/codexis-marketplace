use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Map, Value};

use crate::core::error::CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceSchemaKind {
    Meta,
    Text,
    Toc,
    Versions,
    Related,
    RelatedCounts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MetaSchemaSource {
    Comment,
    Cr,
    Es,
    Eu,
    Jd,
    Lt,
    Sk,
    Vs,
}

struct RenderedSchemaDocument {
    intro: String,
    schema: Option<Value>,
}

pub(crate) fn render_resource_schema_with_source(
    kind: ResourceSchemaKind,
    meta_source: Option<MetaSchemaSource>,
) -> Result<String, CliError> {
    let raw_bundle = load_resource_schema_bundle(kind)?;
    let bundle: Value = serde_json::from_str(raw_bundle)
        .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;
    let rendered = simplify_resource_schema_bundle(kind, meta_source, &bundle)?;

    match rendered.schema {
        Some(schema) => {
            let schema_json = serde_json::to_string_pretty(&schema)
                .map_err(|error| CliError::InvalidStoredSchema(error.to_string()))?;
            Ok(format!("{}\n---\n{}", rendered.intro, schema_json))
        }
        None => Ok(rendered.intro),
    }
}

fn load_resource_schema_bundle(kind: ResourceSchemaKind) -> Result<&'static str, CliError> {
    match kind {
        ResourceSchemaKind::Meta => Ok(include_str!("../../schemas/resource/meta.bundle.json")),
        ResourceSchemaKind::Text => Ok(include_str!("../../schemas/resource/text.bundle.json")),
        ResourceSchemaKind::Toc => Ok(include_str!("../../schemas/resource/toc.bundle.json")),
        ResourceSchemaKind::Versions => {
            Ok(include_str!("../../schemas/resource/versions.bundle.json"))
        }
        ResourceSchemaKind::Related => {
            Ok(include_str!("../../schemas/resource/related.bundle.json"))
        }
        ResourceSchemaKind::RelatedCounts => Ok(include_str!(
            "../../schemas/resource/related-counts.bundle.json"
        )),
    }
}

fn simplify_resource_schema_bundle(
    kind: ResourceSchemaKind,
    meta_source: Option<MetaSchemaSource>,
    bundle: &Value,
) -> Result<RenderedSchemaDocument, CliError> {
    if matches!(kind, ResourceSchemaKind::Meta) {
        return simplify_meta_resource_schema_bundle(bundle, meta_source);
    }

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

    Ok(RenderedSchemaDocument {
        intro: render_resource_intro(
            kind,
            &patterns,
            &query_parameters,
            resource_output_description(kind),
            resource_extra_detail(kind),
            None,
        ),
        schema: Some(make_standalone_json_schema(output_schema, components)?),
    })
}

fn simplify_meta_resource_schema_bundle(
    bundle: &Value,
    meta_source: Option<MetaSchemaSource>,
) -> Result<RenderedSchemaDocument, CliError> {
    match meta_source {
        Some(source) => simplify_typed_meta_resource_schema_bundle(bundle, source),
        None => simplify_generic_meta_resource_schema_bundle(bundle),
    }
}

fn simplify_generic_meta_resource_schema_bundle(
    _bundle: &Value,
) -> Result<RenderedSchemaDocument, CliError> {
    let subcommands = MetaSchemaSource::all()
        .into_iter()
        .map(|source| format!("`cdx-cli schema meta {}`", source.source_code()))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(RenderedSchemaDocument {
        intro: render_resource_intro(
            ResourceSchemaKind::Meta,
            &resource_cdx_patterns(ResourceSchemaKind::Meta),
            &[],
            "Metadata envelope for one document or Czech law.",
            Some(
                "The `/meta` response shape is source-specific, so call `schema meta <SOURCE>` to get the actual output schema."
                    .to_string(),
            ),
            Some(format!(
                "For a source-specific schema, run one of: {subcommands}."
            )),
        ),
        schema: None,
    })
}

fn simplify_typed_meta_resource_schema_bundle(
    bundle: &Value,
    source: MetaSchemaSource,
) -> Result<RenderedSchemaDocument, CliError> {
    let components = meta_components(bundle)?;
    let schema = make_standalone_json_schema(
        typed_meta_response_schema(source, components)?,
        Some(components),
    )?;

    Ok(RenderedSchemaDocument {
        intro: render_resource_intro(
            ResourceSchemaKind::Meta,
            &meta_source_patterns(source),
            &[],
            &format!("Metadata response for a {} document.", source.source_code()),
            Some(format!(
                "Use this schema when the metadata response has `source: \"{}\"` and includes the `{}` field.",
                source.source_code(),
                source.property_name()
            )),
            None,
        ),
        schema: Some(schema),
    })
}

fn meta_components(bundle: &Value) -> Result<&Map<String, Value>, CliError> {
    bundle
        .get("components")
        .and_then(Value::as_object)
        .and_then(|components| components.get("schemas"))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema(
                "meta schema bundle is missing components.schemas".to_string(),
            )
        })
}

fn typed_meta_response_schema(
    source: MetaSchemaSource,
    components: &Map<String, Value>,
) -> Result<Value, CliError> {
    let document_metadata_response = components
        .get("DocumentMetadataResponse")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema(
                "meta schema bundle is missing DocumentMetadataResponse".to_string(),
            )
        })?;
    let response_properties = document_metadata_response
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::InvalidStoredSchema(
                "DocumentMetadataResponse is missing properties".to_string(),
            )
        })?;

    let doc_id_schema = response_properties.get("docId").cloned().ok_or_else(|| {
        CliError::InvalidStoredSchema("DocumentMetadataResponse.docId is missing".to_string())
    })?;
    let source_field_schema = response_properties
        .get(source.property_name())
        .cloned()
        .ok_or_else(|| {
            CliError::InvalidStoredSchema(format!(
                "DocumentMetadataResponse.{} is missing",
                source.property_name()
            ))
        })?;

    let mut properties = Map::new();
    properties.insert("docId".to_string(), doc_id_schema);
    properties.insert(
        "source".to_string(),
        json!({
            "type": "string",
            "enum": [source.source_code()],
            "description": format!("Document source. Always {} for this schema view.", source.source_code())
        }),
    );
    properties.insert(source.property_name().to_string(), source_field_schema);

    Ok(json!({
        "type": "object",
        "properties": properties,
        "required": ["docId", "source", source.property_name()]
    }))
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

fn resource_output_format(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Text => "markdown",
        _ => "json",
    }
}

fn resource_output_description(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "Metadata for one document or Czech law.",
        ResourceSchemaKind::Text => "Rendered document text.",
        ResourceSchemaKind::Toc => {
            "Table of contents with stable part ids and line ranges aligned with /text output."
        }
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
        ResourceSchemaKind::Versions => Some(
            "This endpoint is relevant for CR documents and Czech laws.".to_string(),
        ),
        _ => None,
    }
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
    lines.push("Accepted cdx:// patterns:".to_string());
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

fn resource_endpoint_name(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "meta",
        ResourceSchemaKind::Text => "text",
        ResourceSchemaKind::Toc => "toc",
        ResourceSchemaKind::Versions => "versions",
        ResourceSchemaKind::Related => "related",
        ResourceSchemaKind::RelatedCounts => "related/counts",
    }
}

fn resource_cdx_patterns(kind: ResourceSchemaKind) -> Vec<String> {
    let endpoint = resource_endpoint_name(kind);

    vec![
        format!("cdx://doc/<DOC_ID>/{endpoint}"),
        format!("cdx://cz_law/<NUM>/<YEAR>/{endpoint}"),
    ]
}

fn meta_source_patterns(source: MetaSchemaSource) -> Vec<String> {
    let mut patterns = vec!["cdx://doc/<DOC_ID>/meta".to_string()];
    if source.supports_cz_law() {
        patterns.push("cdx://cz_law/<NUM>/<YEAR>/meta".to_string());
    }
    patterns
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

impl MetaSchemaSource {
    pub(crate) fn all() -> [Self; 8] {
        [
            Self::Cr,
            Self::Sk,
            Self::Jd,
            Self::Es,
            Self::Eu,
            Self::Lt,
            Self::Vs,
            Self::Comment,
        ]
    }

    pub(crate) fn source_code(self) -> &'static str {
        match self {
            Self::Comment => "COMMENT",
            Self::Cr => "CR",
            Self::Es => "ES",
            Self::Eu => "EU",
            Self::Jd => "JD",
            Self::Lt => "LT",
            Self::Sk => "SK",
            Self::Vs => "VS",
        }
    }

    fn property_name(self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Cr => "cr",
            Self::Es => "es",
            Self::Eu => "eu",
            Self::Jd => "jd",
            Self::Lt => "lt",
            Self::Sk => "sk",
            Self::Vs => "vs",
        }
    }

    fn supports_cz_law(self) -> bool {
        matches!(self, Self::Cr)
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
    fn generic_meta_resource_schema_points_to_typed_subcommands() {
        let output = render_resource_schema_with_source(ResourceSchemaKind::Meta, None).unwrap();
        let intro = output.as_str();

        assert!(intro.contains("Endpoint: /meta"));
        assert!(intro.contains("Metadata envelope for one document or Czech law."));
        assert!(intro.contains("cdx://doc/<DOC_ID>/meta"));
        assert!(intro.contains("cdx://cz_law/<NUM>/<YEAR>/meta"));
        assert!(intro.contains("cdx-cli schema meta JD"));
        assert!(intro.contains("schema meta <SOURCE>"));
        assert!(split_rendered_schema_document(&output).is_none());
    }

    #[test]
    fn typed_meta_resource_schema_is_narrowed_to_requested_source() {
        let output = render_resource_schema_with_source(
            ResourceSchemaKind::Meta,
            Some(MetaSchemaSource::Jd),
        )
        .unwrap();
        let (intro, schema) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /meta"));
        assert!(intro.contains("source: \"JD\""));
        assert!(intro.contains("cdx://doc/<DOC_ID>/meta"));
        assert!(!intro.contains("cdx://cz_law/<NUM>/<YEAR>/meta"));
        assert_eq!(schema["properties"]["source"]["enum"], json!(["JD"]));
        assert_eq!(
            schema["properties"]["jd"]["$ref"],
            "#/$defs/JdMetadataApiDto"
        );
        assert!(schema["$defs"]["JdMetadataApiDto"].is_object());
    }

    #[test]
    fn text_resource_schema_is_user_facing_and_marks_markdown_output() {
        let output = render_resource_schema_with_source(ResourceSchemaKind::Text, None).unwrap();
        let (intro, schema) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("Endpoint: /text"));
        assert!(intro.contains("Response format: markdown."));
        assert!(intro.contains("part (repeatable)"));
        assert_eq!(schema["type"], "string");
        assert!(!output.contains("/rest/cdx-api/"));
    }

    #[test]
    fn related_resource_schema_keeps_cdx_law_pattern_even_when_docs_are_incomplete() {
        let output = render_resource_schema_with_source(ResourceSchemaKind::Related, None).unwrap();
        let (intro, _) = split_rendered_schema_document(&output).unwrap();

        assert!(intro.contains("cdx://cz_law/<NUM>/<YEAR>/related"));
    }
}
