use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

pub fn list(
    client: &GraphQLClient,
    marketplace: Option<&str>,
    available: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    if available {
        let data = client.execute(graphql::GET_AVAILABLE_PLUGINS, json!({}))?;
        let mut out: Vec<Value> = Vec::new();
        if let Some(marketplaces) = data.get("marketplaces").and_then(|v| v.as_array()) {
            for mp in marketplaces {
                let mp_name = mp.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let mp_id = mp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(filter) = marketplace {
                    if filter != mp_name && filter != mp_id {
                        continue;
                    }
                }
                if let Some(plugins) = mp.get("plugins").and_then(|v| v.as_array()) {
                    for p in plugins {
                        let mut p = p.clone();
                        if let Some(obj) = p.as_object_mut() {
                            obj.insert("marketplace".into(), Value::String(mp_name.into()));
                        }
                        out.push(p);
                    }
                }
            }
        }
        print_output(&Value::Array(out), format);
    } else {
        let m = marketplace.ok_or_else(|| {
            CdxctlError::Parse("--marketplace is required for listing installed plugins".into())
        })?;
        let data = client.execute(graphql::GET_INSTALLED_PLUGINS, json!({ "marketplace": m }))?;
        let result = data
            .get("installedPlugins")
            .cloned()
            .unwrap_or(Value::Array(vec![]));
        print_output(&result, format);
    }
    Ok(())
}

pub fn install(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let input = json!({
        "id": id,
    });
    let data = client.execute(graphql::INSTALL_PLUGIN, json!({ "input": input }))?;
    let result = data.get("installPlugin").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}

pub fn uninstall(
    client: &GraphQLClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let input = json!({
        "id": id,
    });
    let data = client.execute(graphql::UNINSTALL_PLUGIN, json!({ "input": input }))?;
    let result = data.get("uninstallPlugin").cloned().unwrap_or(Value::Null);
    print_output(&result, format);
    Ok(())
}
