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
    At,
    CitedByDecisions,
    Related,
    RelatedCounts,
    Citations,
    PublicationResolve,
    BwbId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaSource {
    Nlbwb,
    Nluit,
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
        .map(|source| {
            format!(
                "`cdx-nl schema {} {}`",
                resource_endpoint_name(kind),
                source.source_code()
            )
        })
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

    let patterns = source_specific_cdx_patterns(kind, source);

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
    use ResourceSchemaKind as K;
    Ok(match kind {
        K::Text => include_str!("../../schemas/resource/text.bundle.json"),
        K::Versions => include_str!("../../schemas/resource/versions-nlbwb.bundle.json"),
        K::At => include_str!("../../schemas/resource/at-nlbwb.bundle.json"),
        K::CitedByDecisions => {
            include_str!("../../schemas/resource/cited-by-decisions-nlbwb.bundle.json")
        }
        K::Citations => include_str!("../../schemas/resource/citations-nlbwb.bundle.json"),
        K::PublicationResolve => {
            include_str!("../../schemas/resource/publication-resolve.bundle.json")
        }
        K::BwbId => include_str!("../../schemas/resource/bwbid-nlbwb.bundle.json"),
        _ => {
            return Err(CliError::InvalidStoredSchema(format!(
                "/{} is source-aware, not shared",
                resource_endpoint_name(kind)
            )))
        }
    })
}

fn load_source_aware_schema_bundle(
    kind: ResourceSchemaKind,
    source: SchemaSource,
) -> Result<&'static str, CliError> {
    use ResourceSchemaKind as K;
    use SchemaSource as S;
    Ok(match (kind, source) {
        (K::Meta, S::Nlbwb) => include_str!("../../schemas/resource/meta-nlbwb.bundle.json"),
        (K::Meta, S::Nluit) => include_str!("../../schemas/resource/meta-nluit.bundle.json"),
        (K::Toc, S::Nlbwb) => include_str!("../../schemas/resource/toc-nlbwb.bundle.json"),
        (K::Toc, S::Nluit) => include_str!("../../schemas/resource/toc-nluit.bundle.json"),
        (K::Parts, S::Nlbwb) => include_str!("../../schemas/resource/parts-nlbwb.bundle.json"),
        (K::Parts, S::Nluit) => include_str!("../../schemas/resource/parts-nluit.bundle.json"),
        (K::Related, S::Nlbwb) => include_str!("../../schemas/resource/related-nlbwb.bundle.json"),
        (K::Related, S::Nluit) => include_str!("../../schemas/resource/related-nluit.bundle.json"),
        (K::RelatedCounts, S::Nlbwb) => {
            include_str!("../../schemas/resource/related-counts-nlbwb.bundle.json")
        }
        (K::RelatedCounts, S::Nluit) => {
            include_str!("../../schemas/resource/related-counts-nluit.bundle.json")
        }
        (kind, source) => {
            return Err(CliError::InvalidStoredSchema(format!(
                "no bundle for /{} on {}",
                resource_endpoint_name(kind),
                source.source_code()
            )))
        }
    })
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

        // Preferred content types first (deterministic ordering).
        for content_type in ["application/json", "text/plain", "text/plain;charset=UTF-8"] {
            if let Some(schema) = content
                .get(content_type)
                .and_then(Value::as_object)
                .and_then(|entry| entry.get("schema"))
            {
                return Ok(schema.clone());
            }
        }

        // Fallback: take the first content entry that exposes a schema (covers
        // text/plain variants with extra parameters, etc.).
        for (_, entry) in content.iter() {
            if let Some(schema) = entry.as_object().and_then(|entry| entry.get("schema")) {
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
        ResourceSchemaKind::At => "at",
        ResourceSchemaKind::CitedByDecisions => "cited-by-decisions",
        ResourceSchemaKind::Related => "related",
        ResourceSchemaKind::RelatedCounts => "related/counts",
        ResourceSchemaKind::Citations => "citations",
        ResourceSchemaKind::PublicationResolve => "publication-resolve",
        ResourceSchemaKind::BwbId => "bwbid",
    }
}

fn resource_endpoint_label(kind: ResourceSchemaKind) -> &'static str {
    match kind {
        ResourceSchemaKind::Meta => "Metadata",
        ResourceSchemaKind::Text => "Text",
        ResourceSchemaKind::Toc => "Table of contents",
        ResourceSchemaKind::Parts => "Parts",
        ResourceSchemaKind::Versions => "Versions",
        ResourceSchemaKind::At => "Version in force on a date",
        ResourceSchemaKind::CitedByDecisions => "Rechtspraak decisions citing this law",
        ResourceSchemaKind::Related => "Related documents",
        ResourceSchemaKind::RelatedCounts => "Related counts",
        ResourceSchemaKind::Citations => "Citations",
        ResourceSchemaKind::PublicationResolve => "Publication resolver",
        ResourceSchemaKind::BwbId => "BWB-id resolver",
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
        ResourceSchemaKind::Meta => "Metadata for one document or Dutch law.",
        ResourceSchemaKind::Text => "Rendered document text.",
        ResourceSchemaKind::Toc => {
            "Table of contents with stable part ids and line ranges aligned with /text output."
        }
        ResourceSchemaKind::Parts => "Document parts with content.",
        ResourceSchemaKind::Versions => "Available document versions (toestanden).",
        ResourceSchemaKind::At => {
            "The toestand in force on a given date plus its PDFs and neighbour pointers."
        }
        ResourceSchemaKind::CitedByDecisions => {
            "Rechtspraak decisions whose lawReferences cite this BWB document."
        }
        ResourceSchemaKind::Related => "Related documents plus paging metadata.",
        ResourceSchemaKind::RelatedCounts => {
            "Per-relation-type counts for the current related-document scope."
        }
        ResourceSchemaKind::Citations => {
            "Structured citations extracted from the toestand XML, sliced by article scope."
        }
        ResourceSchemaKind::PublicationResolve => {
            "Resolved publication mapping to hosting document and attachment filename."
        }
        ResourceSchemaKind::BwbId => {
            "Resolution from a BWB identifier or afkorting to a display document id."
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
    use ResourceSchemaKind as K;
    match kind {
        K::Citations => vec![
            "cdx-nl://doc/<NLBWB_ID>/citations?toestandId=X[&page=N]".into(),
            "cdx-nl://law/NL/<BWB_ID>/citations?toestandId=X[&page=N]".into(),
            "cdx-nl://afkorting/<ABBR>/citations?toestandId=X[&page=N]".into(),
        ],
        K::PublicationResolve => vec!["cdx-nl://publication/<PUB_ID>/resolve".into()],
        K::BwbId => vec![
            "cdx-nl://law/NL/<BWB_ID>".into(),
            "cdx-nl://afkorting/<ABBR>".into(),
        ],
        K::Versions => vec![
            "cdx-nl://doc/<NLBWB_ID>/versions".into(),
            "cdx-nl://law/NL/<BWB_ID>/versions".into(),
            "cdx-nl://afkorting/<ABBR>/versions".into(),
        ],
        K::At => vec![
            "cdx-nl://doc/<NLBWB_ID>/at?date=YYYY-MM-DD".into(),
            "cdx-nl://law/NL/<BWB_ID>/at?date=YYYY-MM-DD".into(),
            "cdx-nl://afkorting/<ABBR>/at?date=YYYY-MM-DD".into(),
        ],
        K::CitedByDecisions => vec![
            "cdx-nl://doc/<NLBWB_ID>/cited-by-decisions".into(),
            "cdx-nl://law/NL/<BWB_ID>/cited-by-decisions".into(),
            "cdx-nl://afkorting/<ABBR>/cited-by-decisions".into(),
        ],
        K::Text => vec![
            "cdx-nl://doc/<DOC_ID>/text".into(),
            "cdx-nl://law/NL/<BWB_ID>/text".into(),
            "cdx-nl://afkorting/<ABBR>/text".into(),
            "cdx-nl://ecli/<ECLI>/text".into(),
        ],
        _ => {
            let endpoint = resource_endpoint_name(kind);
            vec![
                format!("cdx-nl://doc/<DOC_ID>/{endpoint}"),
                format!("cdx-nl://law/NL/<BWB_ID>/{endpoint} (NLBWB only)"),
                format!("cdx-nl://afkorting/<ABBR>/{endpoint} (NLBWB only)"),
                format!("cdx-nl://ecli/<ECLI>/{endpoint} (NLUIT only)"),
            ]
        }
    }
}

fn source_specific_cdx_patterns(
    kind: ResourceSchemaKind,
    source: SchemaSource,
) -> Vec<String> {
    let endpoint = resource_endpoint_name(kind);

    match source {
        SchemaSource::Nlbwb => vec![
            format!("cdx-nl://doc/<NLBWB_ID>/{endpoint}"),
            format!("cdx-nl://law/NL/<BWB_ID>/{endpoint}"),
            format!("cdx-nl://afkorting/<ABBR>/{endpoint}"),
        ],
        SchemaSource::Nluit => vec![
            format!("cdx-nl://doc/<NLUIT_ID>/{endpoint}"),
            format!("cdx-nl://ecli/<ECLI>/{endpoint}"),
        ],
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
    lines.push("Accepted cdx-nl:// patterns:".to_string());
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
        use ResourceSchemaKind as K;
        matches!(
            self,
            K::Meta | K::Toc | K::Parts | K::Related | K::RelatedCounts
        )
    }

    fn available_sources(self) -> &'static [SchemaSource] {
        use ResourceSchemaKind as K;
        use SchemaSource as S;
        match self {
            K::Meta | K::Toc | K::Parts | K::Related | K::RelatedCounts => {
                &[S::Nlbwb, S::Nluit]
            }
            K::Text
            | K::Versions
            | K::At
            | K::CitedByDecisions
            | K::Citations
            | K::PublicationResolve
            | K::BwbId => &[],
        }
    }
}

impl SchemaSource {
    pub(crate) fn source_code(self) -> &'static str {
        match self {
            Self::Nlbwb => "NLBWB",
            Self::Nluit => "NLUIT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_endpoint_renders_without_panic() {
        let cases: Vec<(ResourceSchemaKind, Option<SchemaSource>)> = vec![
            (ResourceSchemaKind::Meta, Some(SchemaSource::Nlbwb)),
            (ResourceSchemaKind::Meta, Some(SchemaSource::Nluit)),
            (ResourceSchemaKind::Meta, None), // generic intro
            (ResourceSchemaKind::Text, None),
            (ResourceSchemaKind::Versions, None),
            (ResourceSchemaKind::At, None),
            (ResourceSchemaKind::CitedByDecisions, None),
            (ResourceSchemaKind::Citations, None),
            (ResourceSchemaKind::PublicationResolve, None),
            (ResourceSchemaKind::BwbId, None),
            (ResourceSchemaKind::Related, Some(SchemaSource::Nlbwb)),
            (ResourceSchemaKind::RelatedCounts, Some(SchemaSource::Nluit)),
        ];
        for (kind, source) in cases {
            let body = render_resource_schema(kind, source)
                .unwrap_or_else(|e| panic!("{kind:?}/{source:?}: {e}"));
            assert!(
                !body.is_empty(),
                "{kind:?}/{source:?} produced empty output"
            );
            assert!(
                body.contains("cdx-nl://"),
                "{kind:?}/{source:?} missing cdx-nl:// pattern"
            );
        }
    }

    #[test]
    fn typed_schema_has_separator_and_json() {
        let body =
            render_resource_schema(ResourceSchemaKind::Meta, Some(SchemaSource::Nlbwb)).unwrap();
        assert!(
            body.contains("\n---\n"),
            "typed schema should have intro/--- /JSON separator"
        );
        let after = body.split("\n---\n").nth(1).expect("schema section");
        let _parsed: serde_json::Value =
            serde_json::from_str(after).expect("section after --- must be valid JSON");
    }

    #[test]
    fn generic_source_intro_lists_typed_invocations() {
        let body = render_resource_schema(ResourceSchemaKind::Meta, None).unwrap();
        assert!(body.contains("schema meta NLBWB"));
        assert!(body.contains("schema meta NLUIT"));
        // No JSON section in generic intro mode.
        assert!(!body.contains("\n---\n"));
    }
}
