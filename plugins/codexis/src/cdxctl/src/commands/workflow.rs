use super::resolve_node_id;
use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const SESSION_ENV: &str = "CODEXIS_PUBLIC_SESSION_ID";
const FAST_POLL: Duration = Duration::from_secs(3);
const SLOW_POLL: Duration = Duration::from_secs(10);
const FAST_POLL_WINDOW: Duration = Duration::from_secs(120);
const MAX_POLL_FAILURES: u32 = 10;

pub struct StartArgs<'a> {
    pub script_path: &'a str,
    pub args_json: Option<&'a str>,
    pub wait: bool,
    pub timeout_secs: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunState {
    status: Option<String>,
    phases: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Terminal {
    Pending,
    ChangesRequested(String),
    Failed(String),
    Interrupted,
    Complete(String),
}

#[derive(Debug)]
pub struct Step {
    pub lines: Vec<String>,
    pub terminal: Terminal,
    pub state: RunState,
}

#[derive(Debug)]
pub enum Ending {
    Keep,
    Done(String),
    Stop(CdxctlError),
}

pub fn create(client: &GraphQLClient, file: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let input = json!({ "chatId": chat_node_id()?, "scriptPath": file });
    let data = client.execute(graphql::VALIDATE_WORKFLOW, json!({ "input": input }))?;
    let meta = union_result(&data, "validateWorkflow")?;
    print_output(&strip_typename(meta), format);
    Ok(())
}

pub fn start(
    client: &GraphQLClient,
    args: StartArgs,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    let input = start_input(&chat_node_id()?, args.script_path, args.args_json)?;
    let data = client.execute(graphql::START_WORKFLOW, json!({ "input": input }))?;
    let started = union_result(&data, "startWorkflow")?;
    let run_id = workflow_run_id(&started)?;

    if !args.wait {
        print_output(&json!({ "workflowRunId": run_id }), format);
        return Ok(());
    }
    println!("workflowRunId: {run_id}");
    wait_for_run(client, &run_id, Duration::from_secs(args.timeout_secs))
}

pub fn status(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = fetch_run(client, id)?;
    let run = union_result(&data, "workflowRun")?;
    print_output(&strip_typename(run), format);
    Ok(())
}

fn fetch_run(client: &GraphQLClient, id: &str) -> Result<Value, CdxctlError> {
    client.execute(
        graphql::GET_WORKFLOW_RUN,
        json!({ "id": resolve_node_id(id, "Workflow") }),
    )
}

/// Poll until the run reaches a terminal state, printing every state and phase
/// change as it happens so the caller sees progress while the shell call blocks.
/// A failed poll does not end the wait: the run keeps going on the server, so the
/// last known state is held and the read is retried until the failures run out.
fn wait_for_run(
    client: &GraphQLClient,
    run_id: &str,
    timeout: Duration,
) -> Result<(), CdxctlError> {
    let started = Instant::now();
    let mut state = RunState::default();
    let mut failures: u32 = 0;
    loop {
        match fetch_run(client, run_id).and_then(|data| poll_step(&data, &state)) {
            Ok(step) => {
                failures = 0;
                for line in &step.lines {
                    println!("{line}");
                }
                state = step.state;
                match ending(run_id, step.terminal) {
                    Ending::Keep => {}
                    Ending::Done(result) => {
                        println!("{result}");
                        return Ok(());
                    }
                    Ending::Stop(error) => return Err(error),
                }
            }
            Err(error) => {
                failures += 1;
                println!("{}", poll_failure(run_id, &state, &error, failures)?);
            }
        }
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(CdxctlError::GraphQL(vec![format!(
                "workflow {run_id} is still running after {}s; check it with `cdxctl workflow status {run_id}`",
                timeout.as_secs()
            )]));
        }
        std::thread::sleep(poll_interval(elapsed).min(timeout - elapsed));
    }
}

/// What one polled state means for the wait. Terminal messages name the run so an
/// aborted wait can still be resumed with `cdxctl workflow status <id>`.
fn ending(run_id: &str, terminal: Terminal) -> Ending {
    match terminal {
        Terminal::Pending => Ending::Keep,
        Terminal::Complete(result) => Ending::Done(result),
        Terminal::ChangesRequested(feedback) => {
            Ending::Stop(CdxctlError::ChangesRequested(feedback))
        }
        Terminal::Failed(error) => Ending::Stop(CdxctlError::GraphQL(vec![format!(
            "workflow {run_id} failed: {error}"
        )])),
        Terminal::Interrupted => Ending::Stop(CdxctlError::GraphQL(vec![format!(
            "workflow {run_id} was interrupted"
        )])),
    }
}

/// One failed read of the run: a line to print while there is budget left, or the
/// error that ends the wait once the reads have failed too many times in a row.
fn poll_failure(
    run_id: &str,
    state: &RunState,
    error: &CdxctlError,
    failures: u32,
) -> Result<String, CdxctlError> {
    if failures < MAX_POLL_FAILURES {
        return Ok(format!(
            "poll {failures}/{MAX_POLL_FAILURES} failed, retrying: {error}"
        ));
    }
    let last = state.status.as_deref().unwrap_or("unknown");
    Err(CdxctlError::GraphQL(vec![format!(
        "gave up reading workflow {run_id} after {MAX_POLL_FAILURES} failed polls ({error}); \
         it was last seen {last} and may still be running, so check it with \
         `cdxctl workflow status {run_id}`"
    )]))
}

fn poll_step(data: &Value, previous: &RunState) -> Result<Step, CdxctlError> {
    let run = union_result(data, "workflowRun")?;
    let state = read_state(&run);
    Ok(Step {
        lines: transitions(previous, &state),
        terminal: classify(&run, &state),
        state,
    })
}

fn read_state(run: &Value) -> RunState {
    RunState {
        status: run
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
        phases: run
            .get("phases")
            .and_then(Value::as_array)
            .map(|phases| {
                phases
                    .iter()
                    .filter_map(|phase| {
                        let title = phase.get("title").and_then(Value::as_str)?;
                        let status = phase.get("status").and_then(Value::as_str).unwrap_or("");
                        Some((title.to_string(), status.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn transitions(previous: &RunState, current: &RunState) -> Vec<String> {
    let mut lines = Vec::new();
    if previous.status != current.status {
        if let Some(status) = current.status.as_deref() {
            lines.push(status_line(status));
        }
    }
    for (index, (title, status)) in current.phases.iter().enumerate() {
        let before = previous
            .phases
            .get(index)
            .map(|(_, status)| status.as_str());
        if before == Some(status.as_str()) {
            continue;
        }
        if let Some(line) = phase_line(title, status) {
            lines.push(line);
        }
    }
    lines
}

fn status_line(status: &str) -> String {
    match status {
        "PROPOSED" => "waiting for approval...".to_string(),
        "RUNNING" => "running".to_string(),
        "COMPLETE" => "complete".to_string(),
        "FAILED" => "failed".to_string(),
        "INTERRUPTED" => "interrupted".to_string(),
        other => format!("status {other}"),
    }
}

fn phase_line(title: &str, status: &str) -> Option<String> {
    match status {
        "RUNNING" => Some(format!("phase \"{title}\" running")),
        "COMPLETE" => Some(format!("phase \"{title}\" complete")),
        _ => None,
    }
}

/// A declined proposal keeps the status `PROPOSED` and only carries `feedback`, so
/// the feedback decides before the status is consulted. A recorded `error` ends the
/// wait too, which covers reading a run between its error and its status write.
fn classify(run: &Value, state: &RunState) -> Terminal {
    if let Some(feedback) = text(run, "feedback") {
        return Terminal::ChangesRequested(feedback);
    }
    if let Some(error) = text(run, "error") {
        return Terminal::Failed(error);
    }
    match state.status.as_deref() {
        Some("FAILED") => Terminal::Failed("no error message was recorded".to_string()),
        Some("INTERRUPTED") => Terminal::Interrupted,
        Some("COMPLETE") => {
            Terminal::Complete(text(run, "resultJson").unwrap_or_else(|| "{}".to_string()))
        }
        _ => Terminal::Pending,
    }
}

fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// A 90-minute run at a 3s interval is 1800 requests, so the interval widens once
/// the approval and the first phase are past.
fn poll_interval(elapsed: Duration) -> Duration {
    if elapsed < FAST_POLL_WINDOW {
        FAST_POLL
    } else {
        SLOW_POLL
    }
}

fn start_input(
    chat_id: &str,
    script_path: &str,
    args_json: Option<&str>,
) -> Result<Value, CdxctlError> {
    Ok(json!({
        "chatId": chat_id,
        "scriptPath": script_path,
        "argsJson": normalize_args_json(args_json)?,
    }))
}

/// `--args` reaches the workflow script as its `args` object, so a malformed value
/// fails here instead of half-way through a run the user already approved.
fn normalize_args_json(args_json: Option<&str>) -> Result<Option<String>, CdxctlError> {
    let Some(raw) = args_json.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(None);
    };
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|error| CdxctlError::Parse(format!("--args is not valid JSON: {error}")))?;
    if !parsed.is_object() {
        return Err(CdxctlError::Parse(
            "--args takes a JSON object, e.g. --args '{\"question\":\"...\"}'".into(),
        ));
    }
    Ok(Some(raw.to_string()))
}

fn workflow_run_id(started: &Value) -> Result<String, CdxctlError> {
    text(started, "workflowRunId").ok_or_else(|| {
        CdxctlError::GraphQL(vec!["startWorkflow returned no workflowRunId".to_string()])
    })
}

fn chat_node_id() -> Result<String, CdxctlError> {
    chat_node_id_from(std::env::var(SESSION_ENV).ok())
}

fn chat_node_id_from(session: Option<String>) -> Result<String, CdxctlError> {
    let session = session
        .map(|session| session.trim().to_string())
        .filter(|session| !session.is_empty())
        .ok_or_else(|| {
            CdxctlError::Parse(format!(
                "workflow commands run inside a chat, and {SESSION_ENV} is not set"
            ))
        })?;
    Ok(resolve_node_id(&session, "Chat"))
}

/// Read one member of a workflow result union. Every error member of these unions
/// ends in `Error` and carries `message`, so drift in the member name still lands
/// as the daemon's own message instead of a missing-field parse failure.
fn union_result(data: &Value, field: &str) -> Result<Value, CdxctlError> {
    let payload = data.get(field).cloned().unwrap_or(Value::Null);
    if payload.is_null() {
        return Err(CdxctlError::GraphQL(vec![format!(
            "{field} returned nothing"
        )]));
    }
    let typename = payload
        .get("__typename")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if typename.ends_with("Error") {
        let message = text(&payload, "message")
            .unwrap_or_else(|| format!("{field} was rejected without a message"));
        return Err(CdxctlError::GraphQL(vec![message]));
    }
    Ok(payload)
}

fn strip_typename(mut value: Value) -> Value {
    match &mut value {
        Value::Object(map) => {
            map.remove("__typename");
            for (_, nested) in map.iter_mut() {
                *nested = strip_typename(nested.take());
            }
        }
        Value::Array(items) => {
            for item in items {
                *item = strip_typename(item.take());
            }
        }
        _ => {}
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn run(body: Value) -> Value {
        json!({ "workflowRun": body })
    }

    fn running(phases: Value) -> Value {
        run(json!({
            "__typename": "WorkflowRun",
            "id": "Workflow:1",
            "status": "RUNNING",
            "phases": phases,
            "feedback": Value::Null,
            "error": Value::Null,
            "resultJson": Value::Null,
        }))
    }

    #[test]
    fn a_proposed_run_waits_for_the_user_to_approve_it() {
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "PROPOSED",
                "phases": [{ "title": "Research", "status": "PLANNED" }],
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["waiting for approval..."]);
        assert_eq!(step.terminal, Terminal::Pending);
    }

    #[test]
    fn each_state_and_phase_change_is_printed_once() {
        let mut state = RunState::default();

        let first = poll_step(
            &running(json!([
                { "title": "Research", "status": "RUNNING" },
                { "title": "Draft", "status": "PLANNED" },
            ])),
            &state,
        )
        .unwrap();
        assert_eq!(first.lines, vec!["running", "phase \"Research\" running"]);
        state = first.state;

        let second = poll_step(
            &running(json!([
                { "title": "Research", "status": "RUNNING" },
                { "title": "Draft", "status": "PLANNED" },
            ])),
            &state,
        )
        .unwrap();
        assert!(second.lines.is_empty());
        state = second.state;

        let third = poll_step(
            &running(json!([
                { "title": "Research", "status": "COMPLETE" },
                { "title": "Draft", "status": "RUNNING" },
            ])),
            &state,
        )
        .unwrap();
        assert_eq!(
            third.lines,
            vec!["phase \"Research\" complete", "phase \"Draft\" running"]
        );
    }

    #[test]
    fn phases_that_share_a_title_are_tracked_apart() {
        let step = poll_step(
            &running(json!([
                { "title": "Rešerše", "status": "COMPLETE" },
                { "title": "Rešerše", "status": "RUNNING" },
            ])),
            &RunState {
                status: Some("RUNNING".to_string()),
                phases: vec![
                    ("Rešerše".to_string(), "RUNNING".to_string()),
                    ("Rešerše".to_string(), "PLANNED".to_string()),
                ],
            },
        )
        .unwrap();

        assert_eq!(
            step.lines,
            vec!["phase \"Rešerše\" complete", "phase \"Rešerše\" running"]
        );
    }

    #[test]
    fn a_phase_that_arrives_with_the_plan_is_not_announced_before_it_runs() {
        let step = poll_step(
            &running(json!([{ "title": "Draft", "status": "PLANNED" }])),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["running"]);
    }

    #[test]
    fn a_completed_run_returns_the_result_json_verbatim() {
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "COMPLETE",
                "phases": [{ "title": "Draft", "status": "COMPLETE" }],
                "resultJson": "{\"answer\":\"42\"}",
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["complete", "phase \"Draft\" complete"]);
        assert_eq!(
            step.terminal,
            Terminal::Complete("{\"answer\":\"42\"}".to_string())
        );
    }

    #[test]
    fn a_truncated_result_reaches_the_caller_as_the_daemon_wrote_it() {
        let truncated =
            "{\"answer\":\"42\"\n\n… [workflow output truncated at 60000 of 91234 characters]";
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "COMPLETE",
                "resultJson": truncated,
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.terminal, Terminal::Complete(truncated.to_string()));
    }

    #[test]
    fn a_completed_run_without_a_result_still_answers_with_json() {
        let step = poll_step(
            &run(json!({ "__typename": "WorkflowRun", "status": "COMPLETE" })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.terminal, Terminal::Complete("{}".to_string()));
    }

    #[test]
    fn feedback_means_the_user_declined_and_wants_the_script_revised() {
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "PROPOSED",
                "feedback": "Split the research phase in two",
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(
            step.terminal,
            Terminal::ChangesRequested("Split the research phase in two".to_string())
        );
    }

    #[test]
    fn changes_requested_exits_with_its_own_code_and_repeats_the_feedback() {
        let error = CdxctlError::ChangesRequested("Split the research phase".to_string());

        assert_eq!(error.exit_code(), crate::error::EXIT_CHANGES_REQUESTED);
        assert_ne!(error.exit_code(), crate::error::EXIT_NETWORK);
        assert_eq!(
            error.to_string(),
            "user requested changes: Split the research phase"
        );
    }

    #[test]
    fn a_failed_run_carries_the_daemon_error_message() {
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "FAILED",
                "error": "script threw TypeError",
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["failed"]);
        assert_eq!(
            step.terminal,
            Terminal::Failed("script threw TypeError".to_string())
        );
    }

    #[test]
    fn a_failed_run_without_a_message_still_classifies_as_failed() {
        let step = poll_step(
            &run(json!({ "__typename": "WorkflowRun", "status": "FAILED" })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(
            step.terminal,
            Terminal::Failed("no error message was recorded".to_string())
        );
    }

    #[test]
    fn a_proposal_nobody_answered_ends_the_wait_although_it_never_left_proposed() {
        let step = poll_step(
            &run(json!({
                "__typename": "WorkflowRun",
                "status": "PROPOSED",
                "error": "The user never answered the proposal.",
            })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(
            step.terminal,
            Terminal::Failed("The user never answered the proposal.".to_string())
        );
    }

    #[test]
    fn stopping_the_chat_interrupts_the_run() {
        let step = poll_step(
            &run(json!({ "__typename": "WorkflowRun", "status": "INTERRUPTED" })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["interrupted"]);
        assert_eq!(step.terminal, Terminal::Interrupted);
    }

    #[test]
    fn an_unknown_status_is_reported_rather_than_swallowed() {
        let step = poll_step(
            &run(json!({ "__typename": "WorkflowRun", "status": "QUEUED" })),
            &RunState::default(),
        )
        .unwrap();

        assert_eq!(step.lines, vec!["status QUEUED"]);
        assert_eq!(step.terminal, Terminal::Pending);
    }

    #[test]
    fn an_error_member_of_the_union_becomes_a_graphql_error() {
        let error = poll_step(
            &run(json!({ "__typename": "WorkflowCommandError", "message": "no live chat turn" })),
            &RunState::default(),
        )
        .unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => assert_eq!(messages, vec!["no live chat turn"]),
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn any_error_member_name_is_recognised_by_its_message() {
        let error = union_result(
            &json!({ "startWorkflow": { "__typename": "StartWorkflowError", "message": "script not found" } }),
            "startWorkflow",
        )
        .unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => assert_eq!(messages, vec!["script not found"]),
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn a_missing_run_is_an_error_not_an_empty_state() {
        let error =
            poll_step(&json!({ "workflowRun": Value::Null }), &RunState::default()).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert_eq!(messages, vec!["workflowRun returned nothing"])
            }
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn the_start_input_carries_the_chat_the_script_and_the_args() {
        let input = start_input("Chat:1", "/home/codexis/wf.js", Some(" {\"q\":\"a\"} ")).unwrap();

        assert_eq!(input["chatId"], json!("Chat:1"));
        assert_eq!(input["scriptPath"], json!("/home/codexis/wf.js"));
        assert_eq!(input["argsJson"], json!("{\"q\":\"a\"}"));
    }

    #[test]
    fn a_start_without_args_sends_no_args_json() {
        let input = start_input("Chat:1", "/home/codexis/wf.js", None).unwrap();
        assert_eq!(input["argsJson"], Value::Null);

        let blank = start_input("Chat:1", "/home/codexis/wf.js", Some("  ")).unwrap();
        assert_eq!(blank["argsJson"], Value::Null);
    }

    #[test]
    fn malformed_args_fail_before_the_run_is_proposed() {
        assert!(matches!(
            normalize_args_json(Some("{not json}")),
            Err(CdxctlError::Parse(_))
        ));
        assert!(matches!(
            normalize_args_json(Some("[1, 2]")),
            Err(CdxctlError::Parse(_))
        ));
    }

    #[test]
    fn the_started_run_id_is_required_to_wait_on_it() {
        assert_eq!(
            workflow_run_id(&json!({ "workflowRunId": "Workflow:7" })).unwrap(),
            "Workflow:7"
        );
        assert!(workflow_run_id(&json!({ "__typename": "StartWorkflowSuccess" })).is_err());
    }

    #[test]
    fn polling_slows_down_once_the_approval_and_first_phase_are_past() {
        assert_eq!(poll_interval(Duration::from_secs(0)), FAST_POLL);
        assert_eq!(poll_interval(Duration::from_secs(119)), FAST_POLL);
        assert_eq!(poll_interval(Duration::from_secs(120)), SLOW_POLL);
    }

    #[test]
    fn workflow_commands_need_the_chat_they_run_in() {
        let node_id = chat_node_id_from(Some(" 126-36a91d ".to_string())).unwrap();
        assert_eq!(
            node_id,
            crate::commands::encode_node_id("Chat", "126-36a91d")
        );

        assert!(matches!(
            chat_node_id_from(None),
            Err(CdxctlError::Parse(_))
        ));
        assert!(matches!(
            chat_node_id_from(Some("  ".to_string())),
            Err(CdxctlError::Parse(_))
        ));
    }

    #[test]
    fn create_validates_the_script_against_the_chat_and_prints_the_meta() {
        let (address, requests) = stub_daemon(vec![
            r#"{"data":{"validateWorkflow":{"__typename":"WorkflowMetaResult","name":"Legal research","description":"Answers from the law","phases":[{"title":"Research","detail":"Find the sections"}]}}}"#,
        ]);

        in_chat("126-36a91d");
        create(&client(&address), "/home/codexis/wf.js", OutputFormat::Json).unwrap();

        let body = request_body(&requests.recv().unwrap());
        assert_eq!(
            body["variables"]["input"]["scriptPath"],
            json!("/home/codexis/wf.js")
        );
        assert_eq!(
            body["variables"]["input"]["chatId"],
            json!(crate::commands::encode_node_id("Chat", "126-36a91d"))
        );
        assert!(body["query"].as_str().unwrap().contains("validateWorkflow"));
    }

    #[test]
    fn create_reports_the_daemon_rejection_of_an_invalid_script() {
        let (address, _requests) = stub_daemon(vec![
            r#"{"data":{"validateWorkflow":{"__typename":"WorkflowCommandError","message":"script has no exported meta"}}}"#,
        ]);

        in_chat("126-36a91d");
        let error =
            create(&client(&address), "/home/codexis/wf.js", OutputFormat::Json).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert_eq!(messages, vec!["script has no exported meta"])
            }
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn start_without_wait_returns_as_soon_as_the_run_is_proposed() {
        let (address, requests) = stub_daemon(vec![
            r#"{"data":{"startWorkflow":{"__typename":"StartWorkflowSuccess","workflowRunId":"Workflow:7"}}}"#,
        ]);

        in_chat("126-36a91d");
        start(
            &client(&address),
            StartArgs {
                script_path: "/home/codexis/wf.js",
                args_json: Some(r#"{"question":"a"}"#),
                wait: false,
                timeout_secs: 5400,
            },
            OutputFormat::Json,
        )
        .unwrap();

        let body = request_body(&requests.recv().unwrap());
        assert_eq!(
            body["variables"]["input"]["argsJson"],
            json!(r#"{"question":"a"}"#)
        );
        assert!(requests.try_recv().is_err());
    }

    #[test]
    fn start_with_wait_polls_the_run_and_finishes_when_it_completes() {
        let (address, requests) = stub_daemon(vec![
            r#"{"data":{"startWorkflow":{"__typename":"StartWorkflowSuccess","workflowRunId":"Workflow:7"}}}"#,
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"COMPLETE","phases":[{"title":"Research","status":"COMPLETE"}],"resultJson":"{\"answer\":\"42\"}"}}}"#,
        ]);

        in_chat("126-36a91d");
        start(
            &client(&address),
            StartArgs {
                script_path: "/home/codexis/wf.js",
                args_json: None,
                wait: true,
                timeout_secs: 5400,
            },
            OutputFormat::Json,
        )
        .unwrap();

        requests.recv().unwrap();
        let poll = request_body(&requests.recv().unwrap());
        assert_eq!(
            poll["variables"]["id"],
            json!(crate::commands::encode_node_id("Workflow", "Workflow:7"))
        );
    }

    #[test]
    fn start_with_wait_ends_with_changes_requested_when_the_user_declines() {
        let (address, _requests) = stub_daemon(vec![
            r#"{"data":{"startWorkflow":{"__typename":"StartWorkflowSuccess","workflowRunId":"Workflow:7"}}}"#,
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"PROPOSED","phases":[],"feedback":"Split the research phase"}}}"#,
        ]);

        in_chat("126-36a91d");
        let error = start(
            &client(&address),
            StartArgs {
                script_path: "/home/codexis/wf.js",
                args_json: None,
                wait: true,
                timeout_secs: 5400,
            },
            OutputFormat::Json,
        )
        .unwrap_err();

        assert_eq!(error.exit_code(), crate::error::EXIT_CHANGES_REQUESTED);
        assert!(error.to_string().contains("Split the research phase"));
    }

    #[test]
    fn status_reads_one_run_by_its_id() {
        let (address, requests) = stub_daemon(vec![
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"RUNNING","phases":[{"title":"Research","status":"RUNNING"}]}}}"#,
        ]);

        status(&client(&address), "Workflow:7", OutputFormat::Json).unwrap();

        let body = request_body(&requests.recv().unwrap());
        assert_eq!(
            body["variables"]["id"],
            json!(crate::commands::encode_node_id("Workflow", "Workflow:7"))
        );
    }

    #[test]
    fn a_wait_that_outlives_its_timeout_says_the_run_is_still_going() {
        let (address, _requests) = stub_daemon(vec![
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"RUNNING","phases":[]}}}"#,
        ]);

        let error =
            wait_for_run(&client(&address), "Workflow:7", Duration::from_secs(0)).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert!(messages[0].contains("still running"));
                assert!(messages[0].contains("cdxctl workflow status Workflow:7"));
            }
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn a_wait_on_a_failing_run_ends_with_the_script_error() {
        let (address, _requests) = stub_daemon(vec![
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"FAILED","phases":[],"error":"script threw TypeError"}}}"#,
        ]);

        let error =
            wait_for_run(&client(&address), "Workflow:7", Duration::from_secs(60)).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert_eq!(
                    messages,
                    vec!["workflow Workflow:7 failed: script threw TypeError"]
                )
            }
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn a_wait_on_a_stopped_chat_ends_as_interrupted() {
        let (address, _requests) = stub_daemon(vec![
            r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"INTERRUPTED","phases":[]}}}"#,
        ]);

        let error =
            wait_for_run(&client(&address), "Workflow:7", Duration::from_secs(60)).unwrap_err();

        match error {
            CdxctlError::GraphQL(messages) => {
                assert_eq!(messages, vec!["workflow Workflow:7 was interrupted"])
            }
            other => panic!("expected a GraphQL error, got {other:?}"),
        }
    }

    #[test]
    fn a_run_that_is_still_going_is_polled_again_after_the_interval() {
        let still_running = r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"RUNNING","phases":[]}}}"#;
        let (address, requests) = stub_daemon(vec![still_running, still_running]);

        let error =
            wait_for_run(&client(&address), "Workflow:7", Duration::from_secs(1)).unwrap_err();

        assert!(matches!(error, CdxctlError::GraphQL(_)));
        requests.recv().unwrap();
        requests.recv().unwrap();
    }

    #[test]
    fn a_transient_poll_failure_does_not_end_the_wait() {
        let (address, requests) = stub_daemon_with(vec![
            ("HTTP/1.1 502 Bad Gateway", "{}"),
            (
                "HTTP/1.1 200 OK",
                r#"{"data":{"workflowRun":{"__typename":"WorkflowRun","id":"Workflow:7","status":"COMPLETE","phases":[],"resultJson":"{\"answer\":\"42\"}"}}}"#,
            ),
        ]);

        wait_for_run(&client(&address), "Workflow:7", Duration::from_secs(1)).unwrap();

        requests.recv().unwrap();
        requests.recv().unwrap();
    }

    #[test]
    fn a_failed_poll_is_retried_until_the_budget_is_spent() {
        let state = RunState {
            status: Some("RUNNING".to_string()),
            phases: Vec::new(),
        };
        let error = CdxctlError::Network("connection reset".to_string());

        let retry = poll_failure("Workflow:7", &state, &error, MAX_POLL_FAILURES - 1).unwrap();
        assert!(retry.contains("9/10 failed, retrying"));
        assert!(retry.contains("connection reset"));

        let gave_up = poll_failure("Workflow:7", &state, &error, MAX_POLL_FAILURES).unwrap_err();
        let message = gave_up.to_string();
        assert!(message.contains("gave up reading workflow Workflow:7"));
        assert!(message.contains("last seen RUNNING"));
        assert!(message.contains("cdxctl workflow status Workflow:7"));
    }

    #[test]
    fn giving_up_before_the_first_reading_still_names_the_run() {
        let error = CdxctlError::GraphQL(vec!["The workflow run could not be read.".to_string()]);

        let message = poll_failure(
            "Workflow:7",
            &RunState::default(),
            &error,
            MAX_POLL_FAILURES,
        )
        .unwrap_err()
        .to_string();

        assert!(message.contains("last seen unknown"));
        assert!(message.contains("Workflow:7"));
    }

    #[test]
    fn every_terminal_message_names_the_run_so_the_wait_can_be_resumed() {
        assert!(matches!(
            ending("Workflow:7", Terminal::Pending),
            Ending::Keep
        ));
        assert!(matches!(
            ending("Workflow:7", Terminal::Complete("{}".to_string())),
            Ending::Done(result) if result == "{}"
        ));

        for terminal in [Terminal::Failed("boom".to_string()), Terminal::Interrupted] {
            match ending("Workflow:7", terminal) {
                Ending::Stop(error) => assert!(error.to_string().contains("Workflow:7")),
                other => panic!("expected the wait to stop, got {other:?}"),
            }
        }

        match ending(
            "Workflow:7",
            Terminal::ChangesRequested("split it".to_string()),
        ) {
            Ending::Stop(error) => {
                assert_eq!(error.exit_code(), crate::error::EXIT_CHANGES_REQUESTED);
                assert!(error.to_string().contains("split it"));
            }
            other => panic!("expected the wait to stop, got {other:?}"),
        }
    }

    fn in_chat(session: &str) {
        std::env::set_var(SESSION_ENV, session);
    }

    fn client(address: &str) -> GraphQLClient {
        GraphQLClient::with_endpoint(
            format!("http://{address}/graphql"),
            "Bearer test-token".to_string(),
        )
    }

    fn request_body(request: &str) -> Value {
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap()
    }

    fn stub_daemon(responses: Vec<&'static str>) -> (String, std::sync::mpsc::Receiver<String>) {
        stub_daemon_with(
            responses
                .into_iter()
                .map(|body| ("HTTP/1.1 200 OK", body))
                .collect(),
        )
    }

    /// Answers each request from its own connection (`connection: close`), so the
    /// scripted responses stay in order however the client pools sockets.
    fn stub_daemon_with(
        responses: Vec<(&'static str, &'static str)>,
    ) -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for (status_line, body) in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let request = read_request(&mut stream);
                if sender.send(request).is_err() {
                    return;
                }
                let response = format!(
                    "{status_line}\r\ncontent-type: application/json\r\nconnection: close\r\ncontent-length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        (address, receiver)
    }

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut raw = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&raw).to_string();
            let Some(header_end) = text.find("\r\n\r\n") else {
                continue;
            };
            if raw.len() >= header_end + 4 + content_length(&text[..header_end]) {
                return text;
            }
        }
        String::from_utf8_lossy(&raw).to_string()
    }

    fn content_length(headers: &str) -> usize {
        headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())?
            })
            .unwrap_or(0)
    }

    #[test]
    fn printed_output_keeps_the_daemon_fields_without_the_typename() {
        let stripped = strip_typename(json!({
            "__typename": "WorkflowMetaResult",
            "name": "Legal research",
            "phases": [{ "__typename": "WorkflowPhaseMeta", "title": "Research" }],
        }));

        assert_eq!(
            stripped,
            json!({ "name": "Legal research", "phases": [{ "title": "Research" }] })
        );
    }
}
