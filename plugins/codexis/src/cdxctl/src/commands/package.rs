use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::output::{print_output, OutputFormat};
use clap::Subcommand;
use serde_json::{json, Value};
use std::time::Duration;

const BASE: &str = "/rest/v1/vm/packages";
/// The daemon runs apt for up to ten minutes; leave room for the response to travel back.
const APT_TIMEOUT: Duration = Duration::from_secs(11 * 60);

#[derive(Subcommand)]
pub enum PackageCommands {
    #[command(about = "Show whether an Ubuntu package is installed and its versions")]
    Show { name: String },
    #[command(about = "Install Ubuntu packages (apt-get update, then apt-get install)")]
    Install {
        #[arg(required = true, num_args = 1.., help = "Exact Ubuntu package names")]
        names: Vec<String>,
    },
    #[command(about = "Remove Ubuntu packages")]
    Remove {
        #[arg(required = true, num_args = 1.., help = "Exact Ubuntu package names")]
        names: Vec<String>,
    },
    #[command(about = "Refresh the Ubuntu package index (apt-get update)")]
    Update,
}

pub fn execute(
    client: &GraphQLClient,
    command: PackageCommands,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    match command {
        PackageCommands::Show { name } => {
            validate_names(std::slice::from_ref(&name))?;
            print_output(&client.rest_get_json(&format!("{BASE}/{name}"))?, format);
            Ok(())
        }
        PackageCommands::Install { names } => apply(client, "INSTALL", &names, format),
        PackageCommands::Remove { names } => apply(client, "REMOVE", &names, format),
        PackageCommands::Update => apply(client, "UPDATE", &[], format),
    }
}

fn apply(
    client: &GraphQLClient,
    action: &str,
    names: &[String],
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    if action != "UPDATE" {
        validate_names(names)?;
    }
    let result = client.rest_post_json_waiting(
        BASE,
        &json!({"action": action, "packages": names}),
        APT_TIMEOUT,
    )?;
    print_output(&result, format);
    if result.get("success").and_then(Value::as_bool) == Some(true) {
        return Ok(());
    }
    Err(CdxctlError::OperationFailed(
        result
            .get("output")
            .and_then(Value::as_str)
            .unwrap_or("apt did not report success")
            .to_string(),
    ))
}

fn validate_names(names: &[String]) -> Result<(), CdxctlError> {
    let valid = |name: &String| {
        let bytes = name.as_bytes();
        (2..=128).contains(&bytes.len())
            && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
            && bytes
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"+.-".contains(b))
    };
    if names.is_empty() || names.len() > 50 || !names.iter().all(valid) {
        return Err(CdxctlError::Parse(
            "Provide 1 to 50 exact Ubuntu package names without flags, versions, releases or paths"
                .into(),
        ));
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
    fn installs_through_the_daemon_with_the_vm_token() {
        let (address, requests) = stub_daemon(
            "HTTP/1.1 200 OK",
            r#"{"success":true,"output":"Setting up jq"}"#,
        );

        execute(
            &client(&address),
            PackageCommands::Install {
                names: vec!["jq".into()],
            },
            OutputFormat::Json,
        )
        .unwrap();

        let request = requests.recv().unwrap();
        assert!(request.starts_with("POST /rest/v1/vm/packages HTTP/1.1"));
        assert!(request
            .to_lowercase()
            .contains("authorization: bearer vm-token"));
        assert_eq!(
            body_of(&request),
            json!({"action":"INSTALL","packages":["jq"]})
        );
    }

    #[test]
    fn removes_several_packages_in_one_request() {
        let (address, requests) = stub_daemon("HTTP/1.1 200 OK", r#"{"success":true,"output":""}"#);

        execute(
            &client(&address),
            PackageCommands::Remove {
                names: vec!["jq".into(), "sl".into()],
            },
            OutputFormat::Json,
        )
        .unwrap();

        assert_eq!(
            body_of(&requests.recv().unwrap()),
            json!({"action":"REMOVE","packages":["jq","sl"]})
        );
    }

    #[test]
    fn updates_the_index_without_packages() {
        let (address, requests) = stub_daemon(
            "HTTP/1.1 200 OK",
            r#"{"success":true,"output":"Reading package lists..."}"#,
        );

        execute(
            &client(&address),
            PackageCommands::Update,
            OutputFormat::Json,
        )
        .unwrap();

        assert_eq!(
            body_of(&requests.recv().unwrap()),
            json!({"action":"UPDATE","packages":[]})
        );
    }

    #[test]
    fn fails_with_the_apt_output_when_apt_fails() {
        let (address, _requests) = stub_daemon(
            "HTTP/1.1 200 OK",
            r#"{"success":false,"output":"E: Unable to locate package nope"}"#,
        );

        let result = execute(
            &client(&address),
            PackageCommands::Install {
                names: vec!["nope".into()],
            },
            OutputFormat::Json,
        );

        let error = result.unwrap_err();
        assert_eq!(error.exit_code(), 7);
        assert_eq!(
            error.to_string(),
            "Operation failed: E: Unable to locate package nope"
        );
    }

    #[test]
    fn reports_an_unsupported_daemon() {
        let (address, _requests) = stub_daemon("HTTP/1.1 404 Not Found", "{}");

        let result = execute(
            &client(&address),
            PackageCommands::Update,
            OutputFormat::Json,
        );

        assert!(matches!(result, Err(CdxctlError::Unsupported(_))));
    }

    #[test]
    fn shows_a_package_by_name() {
        let (address, requests) = stub_daemon(
            "HTTP/1.1 200 OK",
            r#"{"name":"jq","installed":true,"installedVersion":"1.7","candidateVersion":"1.8"}"#,
        );

        execute(
            &client(&address),
            PackageCommands::Show { name: "jq".into() },
            OutputFormat::Json,
        )
        .unwrap();

        assert!(requests
            .recv()
            .unwrap()
            .starts_with("GET /rest/v1/vm/packages/jq HTTP/1.1"));
    }

    #[test]
    fn surfaces_an_unknown_package_as_an_error() {
        let (address, _requests) = stub_daemon(
            "HTTP/1.1 404 Not Found",
            r#"{"code":"NOT_FOUND","message":"Unknown package: nope"}"#,
        );

        let result = execute(
            &client(&address),
            PackageCommands::Show {
                name: "nope".into(),
            },
            OutputFormat::Json,
        );

        assert!(matches!(result, Err(CdxctlError::Network(message)) if message.contains("404")));
    }

    #[test]
    fn rejects_apt_expressions_and_paths() {
        for name in [
            "--help",
            "foo=1",
            "foo/noble",
            "./foo.deb",
            "foo*",
            "foo:amd64",
            "x",
            "",
        ] {
            assert!(validate_names(&[name.into()]).is_err(), "{name}");
        }
        assert!(validate_names(&["g++".into(), "libc6".into(), "python3.14".into()]).is_ok());
    }
}
