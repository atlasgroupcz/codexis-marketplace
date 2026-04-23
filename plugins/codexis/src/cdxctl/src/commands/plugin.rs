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
        let vars = match marketplace {
            Some(m) => json!({ "marketplace": m }),
            None => json!({ "marketplace": null }),
        };
        let data = client.execute(graphql::GET_AVAILABLE_PLUGINS, vars)?;
        let result = data
            .get("availablePlugins")
            .cloned()
            .unwrap_or(Value::Array(vec![]));
        print_output(&result, format);
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
