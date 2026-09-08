use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::output::{print_output, OutputFormat};
use clap::Subcommand;
use serde_json::{json, Value};
use std::time::Duration;

const PATH: &str = "/rest/v1/vm/app-env";
const TIMEOUT: Duration = Duration::from_secs(60);
const VALUE_LIMIT: usize = 4096;

#[derive(Subcommand)]
pub enum AppEnvCommands {
    #[command(about = "Set one NAME=VALUE in the app's .env (replaces an existing NAME)")]
    Set { name: String, value: String },
    #[command(about = "Remove NAME from the app's .env")]
    Delete { name: String },
}

pub fn execute(
    client: &GraphQLClient,
    app: &str,
    command: AppEnvCommands,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    validate_app(app)?;
    let body = match command {
        AppEnvCommands::Set { name, value } => {
            validate_name(&name)?;
            validate_value(&value)?;
            json!({"app": app, "action": "SET", "name": name, "value": value})
        }
        AppEnvCommands::Delete { name } => {
            validate_name(&name)?;
            json!({"app": app, "action": "DELETE", "name": name})
        }
    };
    let result = client.rest_post_json_waiting(PATH, &body, TIMEOUT)?;
    print_output(&result, format);
    if result.get("success").and_then(Value::as_bool) == Some(true) {
        return Ok(());
    }
    Err(CdxctlError::OperationFailed(
        result
            .get("output")
            .and_then(Value::as_str)
            .unwrap_or("the daemon did not report success")
            .to_string(),
    ))
}

/// `[a-z0-9][a-z0-9-]{0,63}` - a plugin name.
fn validate_app(app: &str) -> Result<(), CdxctlError> {
    let bytes = app.as_bytes();
    let ok = (1..=64).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-');
    if ok {
        return Ok(());
    }
    Err(CdxctlError::Parse(format!(
        "app '{app}' must be a plugin name: lowercase letters, digits and hyphens, up to 64 characters"
    )))
}

/// `[A-Z_][A-Z0-9_]{0,127}` - an environment variable name.
fn validate_name(name: &str) -> Result<(), CdxctlError> {
    let bytes = name.as_bytes();
    let ok = (1..=128).contains(&bytes.len())
        && (bytes[0].is_ascii_uppercase() || bytes[0] == b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || *b == b'_');
    if ok {
        return Ok(());
    }
    Err(CdxctlError::Parse(format!(
        "name '{name}' must be an env var name: uppercase letters, digits and underscores, not starting with a digit"
    )))
}

fn validate_value(value: &str) -> Result<(), CdxctlError> {
    if value.chars().count() > VALUE_LIMIT {
        return Err(CdxctlError::Parse(format!(
            "value is longer than {VALUE_LIMIT} characters"
        )));
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(CdxctlError::Parse("value must be a single line".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::stub_daemon;

    fn client(address: &str) -> GraphQLClient {
        GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer vm-token".into(),
        )
    }

    fn body_of(request: &str) -> Value {
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap()
    }

    #[test]
    fn sets_a_value_through_the_daemon_with_the_vm_token() {
        let (address, requests) = stub_daemon("HTTP/1.1 200 OK", r#"{"success":true,"output":""}"#);

        execute(
            &client(&address),
            "katastr",
            AppEnvCommands::Set {
                name: "API_KEY".into(),
                value: "s3cr3t".into(),
            },
            OutputFormat::Json,
        )
        .unwrap();

        let request = requests.recv().unwrap();
        assert!(request.starts_with("POST /rest/v1/vm/app-env HTTP/1.1"));
        assert!(request
            .to_lowercase()
            .contains("authorization: bearer vm-token"));
        assert_eq!(
            body_of(&request),
            json!({"app":"katastr","action":"SET","name":"API_KEY","value":"s3cr3t"})
        );
    }

    #[test]
    fn deletes_without_sending_a_value() {
        let (address, requests) = stub_daemon("HTTP/1.1 200 OK", r#"{"success":true,"output":""}"#);

        execute(
            &client(&address),
            "katastr",
            AppEnvCommands::Delete {
                name: "API_KEY".into(),
            },
            OutputFormat::Json,
        )
        .unwrap();

        assert_eq!(
            body_of(&requests.recv().unwrap()),
            json!({"app":"katastr","action":"DELETE","name":"API_KEY"})
        );
    }

    #[test]
    fn fails_with_exit_7_when_the_script_fails() {
        let (address, _requests) = stub_daemon(
            "HTTP/1.1 200 OK",
            r#"{"success":false,"output":"mv: cannot move"}"#,
        );

        let error = execute(
            &client(&address),
            "katastr",
            AppEnvCommands::Delete {
                name: "API_KEY".into(),
            },
            OutputFormat::Json,
        )
        .unwrap_err();

        assert_eq!(error.exit_code(), 7);
        assert_eq!(error.to_string(), "Operation failed: mv: cannot move");
    }

    #[test]
    fn reports_an_unsupported_daemon() {
        let (address, _requests) = stub_daemon("HTTP/1.1 404 Not Found", "{}");

        let result = execute(
            &client(&address),
            "katastr",
            AppEnvCommands::Delete {
                name: "API_KEY".into(),
            },
            OutputFormat::Json,
        );

        assert!(matches!(result, Err(CdxctlError::Unsupported(_))));
    }

    #[test]
    fn rejects_bad_app_names_before_any_request() {
        for app in ["Katastr", "../etc", "", "a b", "-x"] {
            assert!(validate_app(app).is_err(), "{app}");
        }
        assert!(validate_app("katastr").is_ok());
        assert!(validate_app("cdx-sk").is_ok());
    }

    #[test]
    fn rejects_bad_env_names() {
        for name in ["api-key", "1KEY", "", "lower", "A=B"] {
            assert!(validate_name(name).is_err(), "{name}");
        }
        assert!(validate_name("API_KEY").is_ok());
        assert!(validate_name("_X1").is_ok());
    }

    #[test]
    fn rejects_multiline_and_oversized_values() {
        assert!(validate_value("a\nb").is_err());
        assert!(validate_value("a\rb").is_err());
        assert!(validate_value(&"x".repeat(4097)).is_err());
        assert!(validate_value(&"x".repeat(4096)).is_ok());
        assert!(validate_value("").is_ok());
    }
}
