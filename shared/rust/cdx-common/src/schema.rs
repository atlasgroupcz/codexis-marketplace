//! Generic JSON-schema machinery shared by the CODEXIS search plugins.
//!
//! These functions manipulate OpenAPI-style schemas as plain `serde_json`
//! values - rewriting `#/components/schemas/…` refs to `#/$defs/…`, computing
//! the referenced-component closure and assembling a standalone JSON Schema.
//! They carry no plugin-specific knowledge; the schema *data* (bundles) and the
//! per-plugin `ResourceSchemaKind`-typed rendering stay in each plugin.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::error::CliError;

/// Turns an output schema plus its OpenAPI `components` into a standalone JSON
/// Schema: refs are rewritten to `#/$defs/…`, the transitive closure of
/// referenced components is inlined under `$defs`, and the `$schema` dialect is
/// declared.
pub fn make_standalone_json_schema(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rewrites_component_refs_to_defs() {
        let mut schema = json!({"$ref": "#/components/schemas/Doc"});
        rewrite_component_refs_to_defs(&mut schema);
        assert_eq!(schema, json!({"$ref": "#/$defs/Doc"}));
    }

    #[test]
    fn standalone_schema_inlines_referenced_components() {
        let schema = json!({"$ref": "#/components/schemas/Doc"});
        let mut components = Map::new();
        components.insert("Doc".to_string(), json!({"type": "object"}));
        let out = make_standalone_json_schema(schema, Some(&components)).unwrap();
        assert_eq!(out["$ref"], "#/$defs/Doc");
        assert_eq!(out["$defs"]["Doc"], json!({"type": "object"}));
        assert!(out["$schema"].is_string());
    }

    #[test]
    fn closure_follows_transitive_refs() {
        let schema = json!({"$ref": "#/components/schemas/A"});
        let mut components = Map::new();
        components.insert("A".to_string(), json!({"$ref": "#/components/schemas/B"}));
        components.insert("B".to_string(), json!({"type": "string"}));
        let names = referenced_component_closure(&schema, &components).unwrap();
        assert_eq!(names, vec!["A".to_string(), "B".to_string()]);
    }
}
