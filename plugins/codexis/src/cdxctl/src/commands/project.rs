use super::{find_node_id, resolve_node_id, root_node_id};
use crate::client::GraphQLClient;
use crate::commands::agent::format_input_error;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

/// Structured `project create`/`update` fields.
#[derive(Debug, Default)]
pub struct ProjectFields {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    /// Shared folders as raw `--drive` values (drive name or Node ID); `Some` replaces the
    /// whole list.
    pub drives: Option<Vec<String>>,
}

impl ProjectFields {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.instructions.is_none()
            && self.drives.is_none()
    }
}

pub fn list(client: &GraphQLClient, format: OutputFormat) -> Result<(), CdxctlError> {
    let projects = fetch_projects(client)?;
    let rows: Vec<Value> = projects.iter().map(summarize).collect();
    print_output(&json!(rows), format);
    Ok(())
}

pub fn get(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    print_output(&find_project(client, id)?, format);
    Ok(())
}

pub fn create(
    client: &GraphQLClient,
    instructions_file: Option<&str>,
    fields: ProjectFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
    let name = fields
        .name
        .clone()
        .ok_or_else(|| CdxctlError::Parse("Creating a project needs --name".into()))?;
    let drive_folders = resolve_drives(client, &fields)?;
    let mut input = Map::new();
    input.insert("name".into(), json!(name));
    apply_fields(&mut input, &fields, drive_folders.as_deref());
    save_project(client, json!({ "input": Value::Object(input) }), format)
}

pub fn update(
    client: &GraphQLClient,
    id: &str,
    instructions_file: Option<&str>,
    fields: ProjectFields,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let fields = merge_instructions_file(fields, instructions_file)?;
    if fields.is_empty() {
        return Err(CdxctlError::Parse(
            "Nothing to update: pass at least one field flag (--name, --description, --instructions, --drive)".into(),
        ));
    }
    // ProjectInput replaces the whole project, so a partial update has to carry every stored
    // field forward. That includes the desktop folders, which only the desktop app can register
    // and which the CLI would otherwise silently drop.
    let current = find_project(client, id)?;
    let stored_id = project_id(&current, id)?;
    let drive_folders = resolve_drives(client, &fields)?;
    let mut input = merge_current(&current);
    apply_fields(&mut input, &fields, drive_folders.as_deref());
    save_project(
        client,
        json!({ "id": stored_id, "input": Value::Object(input) }),
        format,
    )
}

fn apply_fields(
    input: &mut Map<String, Value>,
    fields: &ProjectFields,
    drive_folders: Option<&[Value]>,
) {
    if let Some(name) = &fields.name {
        input.insert("name".into(), json!(name));
    }
    if let Some(description) = &fields.description {
        input.insert("description".into(), json!(description));
    }
    if let Some(instructions) = &fields.instructions {
        input.insert("instructions".into(), json!(instructions));
    }
    if let Some(folders) = drive_folders {
        input.insert("driveFolders".into(), json!(folders));
    }
}

fn merge_current(current: &Value) -> Map<String, Value> {
    let mut input = Map::new();
    for key in ["name", "description", "instructions"] {
        if let Some(value) = current.get(key).and_then(Value::as_str) {
            input.insert(key.into(), json!(value));
        }
    }
    for key in ["desktopFolders", "driveFolders"] {
        if let Some(folders) = current.get(key).and_then(Value::as_array) {
            input.insert(key.into(), json!(folders));
        }
    }
    input
}

/// A project as `project list` shows it: the nested folders collapse into one cell each, so the
/// list reads as a table. `project get` prints the whole record.
fn summarize(project: &Value) -> Value {
    let mut row = Map::new();
    for key in ["id", "name", "description"] {
        row.insert(key.into(), project.get(key).cloned().unwrap_or(Value::Null));
    }
    row.insert(
        "directoryPath".into(),
        project
            .get("directoryPath")
            .and_then(|path| path.get("absolutePath"))
            .cloned()
            .unwrap_or(Value::Null),
    );
    row.insert(
        "sharedFolders".into(),
        joined_names(project, "driveFolders"),
    );
    row.insert(
        "desktopFolders".into(),
        joined(project, "desktopFolders", "folderName"),
    );
    for key in ["createdAt", "modifiedAt"] {
        row.insert(key.into(), project.get(key).cloned().unwrap_or(Value::Null));
    }
    Value::Object(row)
}

fn joined_names(project: &Value, key: &str) -> Value {
    joined(project, key, "name")
}

fn joined(project: &Value, key: &str, field: &str) -> Value {
    let names: Vec<&str> = project
        .get(key)
        .and_then(Value::as_array)
        .map(|folders| {
            folders
                .iter()
                .filter_map(|folder| folder.get(field).and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    if names.is_empty() {
        Value::Null
    } else {
        json!(names.join(", "))
    }
}

fn resolve_drives(
    client: &GraphQLClient,
    fields: &ProjectFields,
) -> Result<Option<Vec<Value>>, CdxctlError> {
    match &fields.drives {
        Some(values) => {
            let data = client.execute(graphql::GET_CONNECTIONS, json!({}))?;
            let drives: Vec<Value> = ["cloudDrives", "myDrives", "sharedWithMe"]
                .into_iter()
                .flat_map(|key| {
                    data.get(key)
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect();
            drive_folders(&drives, values).map(Some)
        }
        None => Ok(None),
    }
}

/// Turn each `--drive` value into the `{driveId, name}` pair `ProjectDriveFolderInput` takes.
/// `driveId` is the drive's Node ID - the same value the projects UI stores - so a value may be
/// that ID or the drive's name. Naming one drive twice still attaches it once.
fn drive_folders(drives: &[Value], values: &[String]) -> Result<Vec<Value>, CdxctlError> {
    let mut folders: Vec<Value> = Vec::new();
    for value in values {
        let folder = drive_folder(drives, value)?;
        if !folders.contains(&folder) {
            folders.push(folder);
        }
    }
    Ok(folders)
}

fn drive_folder(drives: &[Value], value: &str) -> Result<Value, CdxctlError> {
    drives
        .iter()
        .find(|drive| find_node_id(std::slice::from_ref(drive), value, &["name"]).is_some())
        .map(|drive| {
            json!({
                "driveId": drive.get("id").cloned().unwrap_or(Value::Null),
                "name": drive.get("name").cloned().unwrap_or(Value::Null),
            })
        })
        .ok_or_else(|| CdxctlError::GraphQL(vec![format!("Shared folder not found: {value}")]))
}

fn fetch_projects(client: &GraphQLClient) -> Result<Vec<Value>, CdxctlError> {
    let data = client.execute(graphql::GET_PROJECTS, json!({}))?;
    Ok(data
        .get("projects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn find_project(client: &GraphQLClient, id: &str) -> Result<Value, CdxctlError> {
    select_project(&fetch_projects(client)?, id)
}

pub fn resolve_project_id(client: &GraphQLClient, identifier: &str) -> Result<String, CdxctlError> {
    let projects = fetch_projects(client)?;
    if let Some(project) = projects.iter().find(|project| {
        let Some(id) = project.get("id").and_then(Value::as_str) else {
            return false;
        };
        id == identifier
            || root_node_id(id, "Project").is_some_and(|uuid| uuid.eq_ignore_ascii_case(identifier))
    }) {
        return project_id(project, identifier);
    }
    let matches: Vec<&Value> = projects
        .iter()
        .filter(|project| project.get("name").and_then(Value::as_str) == Some(identifier))
        .collect();
    match matches.as_slice() {
        [] => Err(CdxctlError::Parse(format!("No such project: {identifier}"))),
        [project] => project_id(project, identifier),
        _ => {
            let ids = matches
                .iter()
                .filter_map(|project| project.get("id").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(", ");
            Err(CdxctlError::Parse(format!(
                "Ambiguous project: {identifier}. Use a project ID: {ids}"
            )))
        }
    }
}

/// Pick the project a command acts on. The value is either the project's name or its Node ID;
/// a bare UUID is wrapped into one, which is what the daemon's ids decode to.
fn select_project(projects: &[Value], id: &str) -> Result<Value, CdxctlError> {
    let node_id = resolve_node_id(id, "Project");
    projects
        .iter()
        .find(|project| {
            project.get("id").and_then(Value::as_str) == Some(node_id.as_str())
                || find_node_id(std::slice::from_ref(project), id, &["name"]).is_some()
        })
        .cloned()
        .ok_or_else(|| CdxctlError::Parse(format!("No such project: {id}")))
}

fn project_id(project: &Value, id: &str) -> Result<String, CdxctlError> {
    project
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdxctlError::Parse(format!("Project {id} has no id")))
}

/// Runs the structured `saveProject` mutation (id null → create, id set → update).
fn save_project(
    client: &GraphQLClient,
    variables: Value,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::SAVE_PROJECT, variables)?;
    print_payload(data.get("saveProject"), format)
}

fn print_payload(payload: Option<&Value>, format: OutputFormat) -> Result<(), CdxctlError> {
    let payload = payload.cloned().unwrap_or(Value::Null);
    let errors = payload
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !errors.is_empty() {
        return Err(CdxctlError::GraphQL(
            errors.iter().map(format_input_error).collect(),
        ));
    }
    print_output(
        &payload.get("project").cloned().unwrap_or(Value::Null),
        format,
    );
    Ok(())
}

/// Folds `--instructions-file` into the structured fields, so long project instructions can live
/// in a temp file instead of an inline arg.
fn merge_instructions_file(
    mut fields: ProjectFields,
    instructions_file: Option<&str>,
) -> Result<ProjectFields, CdxctlError> {
    let Some(path) = instructions_file else {
        return Ok(fields);
    };
    if fields.instructions.is_some() {
        return Err(CdxctlError::Parse(
            "Choose one source for the project instructions: --instructions or --instructions-file, not both"
                .into(),
        ));
    }
    let content = fs::read_to_string(path).map_err(|error| {
        CdxctlError::Parse(format!(
            "Failed to read project instructions from {}: {}",
            Path::new(path).display(),
            error
        ))
    })?;
    fields.instructions = Some(content);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_fields, create, drive_folders, get, list, merge_current, merge_instructions_file,
        select_project, summarize, ProjectFields,
    };
    use crate::client::GraphQLClient;
    use crate::commands::encode_node_id;
    use crate::error::CdxctlError;
    use crate::output::OutputFormat;
    use crate::test_support::stub_daemon;
    use serde_json::{json, Map, Value};
    use std::fs;

    fn drives() -> Vec<Value> {
        vec![
            json!({ "id": encode_node_id("Drive", "d-1"), "name": "Smlouvy" }),
            json!({ "id": encode_node_id("Drive", "d-2"), "name": "Judikatura" }),
        ]
    }

    fn stored_project() -> Value {
        json!({
            "id": encode_node_id("Project", "5b1c1c4e-0000-4000-8000-000000000001"),
            "name": "Audit 2026",
            "description": "Roční audit smluv",
            "instructions": "Vycházej jen ze smluv v projektu.",
            "directoryPath": { "absolutePath": "/home/codexis/projects/audit", "displayPath": "~/projects/audit" },
            "desktopFolders": [{
                "deviceId": "7f0e2a1c-0000-4000-8000-00000000000b",
                "deviceName": "MacBook",
                "shareName": Value::Null,
                "folderName": "Smlouvy 2026",
                "relativePath": "Documents/Smlouvy 2026"
            }],
            "driveFolders": [{ "driveId": encode_node_id("Drive", "d-1"), "name": "Smlouvy" }],
            "createdAt": "2026-01-05T08:00:00Z",
            "modifiedAt": "2026-02-01T09:30:00Z"
        })
    }

    #[test]
    fn project_fields_are_empty_until_one_is_set() {
        assert!(ProjectFields::default().is_empty());
        assert!(!ProjectFields {
            drives: Some(vec!["Smlouvy".into()]),
            ..ProjectFields::default()
        }
        .is_empty());
    }

    #[test]
    fn drive_folders_resolve_a_name_to_the_drives_node_id() {
        let resolved = drive_folders(&drives(), &["Smlouvy".to_string()]).unwrap();

        assert_eq!(
            resolved,
            vec![json!({ "driveId": encode_node_id("Drive", "d-1"), "name": "Smlouvy" })]
        );
    }

    #[test]
    fn drive_folders_accept_a_node_id_the_caller_already_has() {
        let node_id = encode_node_id("Drive", "d-2");

        let resolved = drive_folders(&drives(), &[node_id.clone()]).unwrap();

        assert_eq!(
            resolved,
            vec![json!({ "driveId": node_id, "name": "Judikatura" })]
        );
    }

    #[test]
    fn drive_folders_attach_a_repeated_drive_once() {
        let resolved =
            drive_folders(&drives(), &["Smlouvy".to_string(), "Smlouvy".to_string()]).unwrap();

        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn drive_folders_name_the_shared_folder_that_does_not_exist() {
        let error = drive_folders(&drives(), &["Neexistuje".to_string()]).unwrap_err();

        assert!(
            matches!(&error, CdxctlError::GraphQL(messages) if messages == &["Shared folder not found: Neexistuje"]),
            "{error}"
        );
    }

    #[test]
    fn merge_current_carries_every_stored_field_so_a_partial_update_keeps_it() {
        let merged = merge_current(&stored_project());

        assert_eq!(merged["name"], json!("Audit 2026"));
        assert_eq!(merged["description"], json!("Roční audit smluv"));
        assert_eq!(
            merged["instructions"],
            json!("Vycházej jen ze smluv v projektu.")
        );
        assert_eq!(merged["desktopFolders"], stored_project()["desktopFolders"]);
        assert_eq!(merged["driveFolders"], stored_project()["driveFolders"]);
    }

    #[test]
    fn apply_fields_overwrites_only_what_the_flags_set() {
        let mut input = merge_current(&stored_project());
        let fields = ProjectFields {
            instructions: Some("Nové pokyny.".into()),
            ..ProjectFields::default()
        };

        apply_fields(&mut input, &fields, None);

        assert_eq!(input["instructions"], json!("Nové pokyny."));
        assert_eq!(input["name"], json!("Audit 2026"));
        assert_eq!(input["driveFolders"], stored_project()["driveFolders"]);
    }

    #[test]
    fn apply_fields_replaces_the_whole_shared_folder_list() {
        let mut input = merge_current(&stored_project());
        let fields = ProjectFields {
            drives: Some(vec!["Judikatura".into()]),
            ..ProjectFields::default()
        };
        let resolved = drive_folders(&drives(), fields.drives.as_ref().unwrap()).unwrap();

        apply_fields(&mut input, &fields, Some(&resolved));

        assert_eq!(
            input["driveFolders"],
            json!([{ "driveId": encode_node_id("Drive", "d-2"), "name": "Judikatura" }])
        );
    }

    #[test]
    fn create_input_starts_from_the_name_alone() {
        let mut input = Map::new();
        input.insert("name".into(), json!("Audit 2026"));
        let fields = ProjectFields {
            name: Some("Audit 2026".into()),
            ..ProjectFields::default()
        };

        apply_fields(&mut input, &fields, None);

        assert_eq!(input.len(), 1);
        assert!(!input.contains_key("driveFolders"));
    }

    #[test]
    fn summarize_collapses_the_folders_into_one_cell_each() {
        let row = summarize(&stored_project());

        assert_eq!(row["sharedFolders"], json!("Smlouvy"));
        assert_eq!(row["desktopFolders"], json!("Smlouvy 2026"));
        assert_eq!(row["directoryPath"], json!("/home/codexis/projects/audit"));
        assert_eq!(row["name"], json!("Audit 2026"));
        assert!(row.get("instructions").is_none());
    }

    #[test]
    fn summarize_leaves_a_project_without_folders_empty_rather_than_blank() {
        let row = summarize(&json!({ "id": "Project:x", "name": "Prázdný" }));

        assert_eq!(row["sharedFolders"], Value::Null);
        assert_eq!(row["desktopFolders"], Value::Null);
        assert_eq!(row["directoryPath"], Value::Null);
    }

    fn stub_client(body: &'static str) -> (GraphQLClient, std::sync::mpsc::Receiver<String>) {
        let (address, requests) = stub_daemon("HTTP/1.1 200 OK", body);
        (
            GraphQLClient::with_endpoint(
                format!("http://{address}/graphql"),
                "Bearer test-token".to_string(),
            ),
            requests,
        )
    }

    #[test]
    fn select_project_takes_the_name_or_the_node_id() {
        let projects = vec![stored_project()];
        let node_id = stored_project()["id"].as_str().unwrap().to_string();

        assert_eq!(
            select_project(&projects, "Audit 2026").unwrap(),
            stored_project()
        );
        assert_eq!(
            select_project(&projects, &node_id).unwrap(),
            stored_project()
        );
        assert_eq!(
            select_project(&projects, "5b1c1c4e-0000-4000-8000-000000000001").unwrap(),
            stored_project()
        );
    }

    #[test]
    fn select_project_names_the_project_that_does_not_exist() {
        let error = select_project(&[stored_project()], "Chybí").unwrap_err();

        assert!(
            matches!(&error, CdxctlError::Parse(message) if message == "No such project: Chybí"),
            "{error}"
        );
    }

    #[test]
    fn list_asks_the_daemon_for_every_project() {
        let (client, requests) = stub_client(r#"{"data":{"projects":[]}}"#);

        list(&client, OutputFormat::Json).unwrap();

        let request = requests.recv().unwrap();
        assert!(request.contains("query GetProjects"), "{request}");
    }

    #[test]
    fn get_resolves_the_project_by_name() {
        let (client, _requests) = stub_client(
            r#"{"data":{"projects":[{"id":"P1","name":"Audit 2026","driveFolders":[],"desktopFolders":[]}]}}"#,
        );

        get(&client, "Audit 2026", OutputFormat::Json).unwrap();
    }

    #[test]
    fn create_sends_the_name_without_an_id_so_the_daemon_inserts() {
        let (client, requests) = stub_client(
            r#"{"data":{"saveProject":{"project":{"id":"P1","name":"Audit 2026"},"errors":[]}}}"#,
        );
        let fields = ProjectFields {
            name: Some("Audit 2026".into()),
            description: Some("Roční audit smluv".into()),
            ..ProjectFields::default()
        };

        create(&client, None, fields, OutputFormat::Json).unwrap();

        let request = requests.recv().unwrap();
        assert!(request.contains("mutation SaveProject"), "{request}");
        assert!(request.contains(r#""name":"Audit 2026""#), "{request}");
        assert!(!request.contains(r#""id":"#), "{request}");
    }

    #[test]
    fn create_reports_the_field_the_daemon_rejected() {
        let (client, _requests) = stub_client(
            r#"{"data":{"saveProject":{"project":null,"errors":[{"field":"name","message":"Name must not be blank."}]}}}"#,
        );
        let fields = ProjectFields {
            name: Some(" ".into()),
            ..ProjectFields::default()
        };

        let error = create(&client, None, fields, OutputFormat::Json).unwrap_err();

        assert!(
            matches!(&error, CdxctlError::GraphQL(messages) if messages == &["name: Name must not be blank."]),
            "{error}"
        );
    }

    #[test]
    fn create_without_a_name_never_reaches_the_daemon() {
        let (client, _requests) = stub_client(r#"{"data":{}}"#);

        let error =
            create(&client, None, ProjectFields::default(), OutputFormat::Json).unwrap_err();

        assert!(
            matches!(&error, CdxctlError::Parse(message) if message.contains("--name")),
            "{error}"
        );
    }

    #[test]
    fn instructions_file_becomes_the_projects_instructions() {
        let path = std::env::temp_dir().join("cdxctl-project-instructions.md");
        fs::write(&path, "Vycházej jen ze smluv v projektu.").unwrap();

        let fields = merge_instructions_file(ProjectFields::default(), path.to_str()).unwrap();

        assert_eq!(
            fields.instructions.as_deref(),
            Some("Vycházej jen ze smluv v projektu.")
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn instructions_cannot_come_from_a_flag_and_a_file_at_once() {
        let fields = ProjectFields {
            instructions: Some("inline".into()),
            ..ProjectFields::default()
        };

        let error = merge_instructions_file(fields, Some("./pokyny.md")).unwrap_err();

        assert!(
            matches!(&error, CdxctlError::Parse(message) if message.contains("not both")),
            "{error}"
        );
    }

    #[test]
    fn a_missing_instructions_file_names_the_path() {
        let error =
            merge_instructions_file(ProjectFields::default(), Some("/nope/pokyny.md")).unwrap_err();

        assert!(
            matches!(&error, CdxctlError::Parse(message) if message.contains("/nope/pokyny.md")),
            "{error}"
        );
    }
}
