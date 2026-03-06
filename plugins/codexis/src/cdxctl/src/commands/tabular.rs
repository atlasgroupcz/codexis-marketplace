use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn status(
    client: &GraphQLClient,
    folder: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_TABULAR_EXTRACTION, json!({ "folder": folder }))?;
    let result = data
        .get("tabularExtraction")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn add_column(
    client: &GraphQLClient,
    folder: &str,
    name: &str,
    col_type: &str,
    description: Option<&str>,
    tag_options: &[String],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let (query, vars) = match col_type.to_lowercase().as_str() {
        "text" => (
            graphql::ADD_TEXT_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "boolean" | "bool" => (
            graphql::ADD_BOOLEAN_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "date" => (
            graphql::ADD_DATE_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "number" => (
            graphql::ADD_NUMBER_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "currency" => (
            graphql::ADD_CURRENCY_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "list" => (
            graphql::ADD_LIST_COLUMN,
            json!({ "folder": folder, "name": name, "description": description }),
        ),
        "tag" => {
            let options = parse_tag_options(tag_options)?;
            (
                graphql::ADD_TAG_COLUMN,
                json!({ "folder": folder, "name": name, "description": description, "options": options }),
            )
        }
        "tags" => {
            let options = parse_tag_options(tag_options)?;
            (
                graphql::ADD_TAGS_COLUMN,
                json!({ "folder": folder, "name": name, "description": description, "options": options }),
            )
        }
        _ => {
            return Err(CdxctlError::Parse(format!(
                "Unknown column type: {col_type}. Use: text, boolean, date, number, currency, list, tag, tags"
            )));
        }
    };

    let data = client.execute(query, vars)?;
    // The mutation name varies, just get the first field from data
    let result = data
        .as_object()
        .and_then(|obj| obj.values().next())
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn remove_column(
    client: &GraphQLClient,
    folder: &str,
    column_id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(
        graphql::REMOVE_TABULAR_COLUMN,
        json!({ "folder": folder, "columnId": column_id }),
    )?;
    let result = data
        .get("removeTabularColumn")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn start(
    client: &GraphQLClient,
    folder: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(
        graphql::START_TABULAR_EXTRACTION,
        json!({ "folder": folder }),
    )?;
    let result = data
        .get("startTabularExtraction")
        .cloned()
        .unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn results(
    client: &GraphQLClient,
    folder: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_TABULAR_EXTRACTION, json!({ "folder": folder }))?;
    let extraction = data
        .get("tabularExtraction")
        .cloned()
        .unwrap_or(Value::Null);

    // For results, flatten rows into a more readable format
    if let Some(rows) = extraction.get("rows").and_then(|r| r.as_array()) {
        let flat_rows: Vec<Value> = rows
            .iter()
            .map(|row| {
                let mut flat = json!({
                    "fileName": row.get("fileName"),
                    "status": row.get("status"),
                });
                if let Some(err) = row.get("error") {
                    if !err.is_null() {
                        flat["error"] = err.clone();
                    }
                }
                if let Some(cells) = row.get("cells").and_then(|c| c.as_array()) {
                    for cell in cells {
                        let col_name = cell
                            .get("column")
                            .and_then(|c| c.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");
                        let value = extract_cell_value(cell);
                        flat[col_name] = value;
                    }
                }
                flat
            })
            .collect();
        print_output(&json!(flat_rows), format);
    } else {
        print_output(&extraction, format);
    }
    Ok(())
}

fn extract_cell_value(cell: &Value) -> Value {
    if let Some(v) = cell.get("text") {
        return v.clone();
    }
    if let Some(v) = cell.get("date") {
        return v.clone();
    }
    if let Some(v) = cell.get("checked") {
        return v.clone();
    }
    if let Some(v) = cell.get("number") {
        return v.clone();
    }
    if let Some(v) = cell.get("tag") {
        return v.clone();
    }
    if let Some(v) = cell.get("tags") {
        return v.clone();
    }
    if let Some(v) = cell.get("items") {
        return v.clone();
    }
    if let Some(v) = cell.get("amount") {
        let currency = cell
            .get("currencyCode")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        return json!(format!("{} {}", v, currency));
    }
    Value::Null
}

/// Parse tag options from "value:color" format
fn parse_tag_options(options: &[String]) -> Result<Vec<Value>, CdxctlError> {
    if options.is_empty() {
        return Err(CdxctlError::Parse(
            "Tag/tags columns require at least one --option in 'value:COLOR' format".into(),
        ));
    }
    options
        .iter()
        .map(|opt| {
            let parts: Vec<&str> = opt.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(CdxctlError::Parse(format!(
                    "Invalid tag option '{opt}'. Use 'value:COLOR' format (e.g., 'high:RED')"
                )));
            }
            Ok(json!({ "value": parts[0], "color": parts[1].to_uppercase() }))
        })
        .collect()
}
