use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn list(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::GET_MARKETPLACES, json!({}))?;
    let marketplaces = data
        .get("marketplaces")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    print_output(&marketplaces, format);
    Ok(())
}

pub fn add(
    client: &GraphQLClient,
    source: &str,
    source_type: &str,
    git_ref: Option<&str>,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let mut input = json!({});

    match source_type.to_uppercase().as_str() {
        "GIT" => {
            input["sourceType"] = json!("GIT");
            input["gitUrl"] = json!(source);
            if let Some(r) = git_ref {
                input["gitRef"] = json!(r);
            }
        }
        "LOCAL_PATH" | "LOCAL" => {
            input["sourceType"] = json!("LOCAL_PATH");
            input["path"] = json!(source);
        }
        _ => {
            return Err(CdxctlError::Parse(format!(
                "Unknown source type: {source_type}. Use 'git' or 'local'"
            )));
        }
    }

    let data = client.execute(graphql::ADD_MARKETPLACE, json!({ "input": input }))?;
    let result = data.get("addMarketplace").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn remove(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::REMOVE_MARKETPLACE, json!({ "id": id }))?;
    let result = data.get("deleteNode").cloned().unwrap_or(Value::Null);

    if result.is_null() {
        print_output(
            &json!({ "deleted": false, "error": "Marketplace not found or could not be removed" }),
            format,
        );
    } else {
        print_output(&json!({ "deleted": true, "marketplace": result }), format);
    }
    Ok(())
}

pub fn update(
    client: &GraphQLClient,
    id: Option<&str>,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    match id {
        Some(i) => {
            let data = client.execute(graphql::UPDATE_MARKETPLACE, json!({ "id": i }))?;
            let result = data
                .get("updateMarketplace")
                .cloned()
                .unwrap_or(Value::Null);
            print_output(&result, format);
        }
        None => {
            let listing = client.execute(graphql::GET_MARKETPLACES, json!({}))?;
            let mut results: Vec<Value> = Vec::new();
            if let Some(marketplaces) = listing.get("marketplaces").and_then(|v| v.as_array()) {
                for mp in marketplaces {
                    let mp_id = mp.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if mp_id.is_empty() {
                        continue;
                    }
                    let mp_name = mp.get("name").cloned().unwrap_or(Value::Null);
                    match client.execute(graphql::UPDATE_MARKETPLACE, json!({ "id": mp_id })) {
                        Ok(d) => {
                            if let Some(r) = d.get("updateMarketplace").cloned() {
                                results.push(r);
                            }
                        }
                        Err(e) => {
                            results.push(json!({
                                "marketplace": mp_name,
                                "error": format!("{}", e),
                            }));
                        }
                    }
                }
            }
            print_output(&Value::Array(results), format);
        }
    }
    Ok(())
}
