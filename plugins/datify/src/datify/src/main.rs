//! `datify-cli` - Datify public REST API from the command line.
//!
//! Every command prints the API's JSON to stdout unchanged, so the agent reads
//! Datify's own shapes rather than a translation of them. Errors go to stderr
//! with a non-zero exit.

use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use datify_core::client::UploadTarget;
use datify_core::{settings, DatifyClient, DatifyError, Result};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "datify-cli",
    about = "Datify - kolekce, datasety, záznamy, přílohy.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List every collection and its datasets
    Collections,
    /// Dataset metadata including its properties
    Dataset { dataset_id: String },
    /// Work with records
    Records {
        #[command(subcommand)]
        cmd: RecordCmd,
    },
    /// One record property's value (paginated)
    Property {
        record_id: String,
        property_id: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Attachment metadata
    Attachment { attachment_id: String },
    /// Upload a file as an attachment
    Upload {
        /// File to upload
        file: PathBuf,
        /// Attach straight to this record (needs --property)
        #[arg(long, requires = "property")]
        record: Option<String>,
        /// Attachment property on --record (needs --record)
        ///
        /// The requirement runs both ways: on its own this used to fall through
        /// to an unattached upload, and an unattached attachment can never be
        /// deleted through this API.
        #[arg(long, requires = "record")]
        property: Option<String>,
        /// Upload into this dataset's attachment pool
        #[arg(long, conflicts_with_all = ["record", "property"])]
        dataset: Option<String>,
        /// Ask Datify to run extraction over the file (not for --record)
        ///
        /// The record endpoint does not take the flag, so accepting it here
        /// would silently drop it and imply extraction that never ran.
        #[arg(long, conflicts_with_all = ["record", "property"])]
        ai: bool,
    },
    /// User detail
    User { user_id: String },
    /// API token and endpoint settings
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
}

#[derive(Subcommand)]
enum RecordCmd {
    /// Page through a dataset's records
    List {
        dataset_id: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        /// createdAt,asc or createdAt,desc
        #[arg(long)]
        sort: Option<String>,
    },
    /// Filter a dataset's records. The filter is always a flat match/conditions
    /// group, even for one condition - Datify does not support nesting.
    Query {
        dataset_id: String,
        /// Filter JSON, or - to read it from stdin
        #[arg(long)]
        filter: String,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        sort: Option<String>,
    },
    /// One record
    Get { record_id: String },
    /// Create records. Body is {"values": {...}} for one, or
    /// {"records": [...]} for up to ten.
    Create {
        dataset_id: String,
        /// Body JSON, or - to read it from stdin
        #[arg(long)]
        data: String,
    },
    /// Update a record. Merges by default; --replace replaces it whole.
    Update {
        record_id: String,
        /// Body JSON, or - to read it from stdin
        #[arg(long)]
        data: String,
        #[arg(long)]
        replace: bool,
    },
    /// Delete one record. Irreversible.
    Delete { record_id: String },
    /// Delete several records in one call. Irreversible.
    DeleteMany {
        #[arg(required = true)]
        record_ids: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SettingsCmd {
    /// Show the effective configuration (the token is masked)
    Show,
    /// Store an API token, after checking Datify accepts it
    SetKey { token: String },
    /// Forget the stored token
    ClearKey,
    /// Check the stored token still works
    Test,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Collections => emit(DatifyClient::new()?.collections()?),
        Command::Dataset { dataset_id } => emit(DatifyClient::new()?.dataset(&dataset_id)?),
        Command::Records { cmd } => run_records(cmd),
        Command::Property {
            record_id,
            property_id,
            limit,
            cursor,
        } => emit(DatifyClient::new()?.record_property(
            &record_id,
            &property_id,
            limit,
            cursor.as_deref(),
        )?),
        Command::Attachment { attachment_id } => {
            emit(DatifyClient::new()?.get_attachment(&attachment_id)?)
        }
        Command::Upload {
            file,
            record,
            property,
            dataset,
            ai,
        } => {
            let target = match (record.as_deref(), property.as_deref(), dataset.as_deref()) {
                (Some(record_id), Some(property_id), _) => UploadTarget::RecordProperty {
                    record_id,
                    property_id,
                },
                (_, _, Some(dataset_id)) => UploadTarget::Dataset { dataset_id },
                _ => UploadTarget::Standalone,
            };
            emit(DatifyClient::new()?.upload_attachment(target, &file, ai)?)
        }
        Command::User { user_id } => emit(DatifyClient::new()?.user(&user_id)?),
        Command::Settings { cmd } => run_settings(cmd),
    }
}

fn run_records(cmd: RecordCmd) -> Result<()> {
    let client = DatifyClient::new()?;
    match cmd {
        RecordCmd::List {
            dataset_id,
            limit,
            cursor,
            sort,
        } => emit(client.list_records(&dataset_id, limit, cursor.as_deref(), sort.as_deref())?),
        RecordCmd::Query {
            dataset_id,
            filter,
            limit,
            cursor,
            sort,
        } => emit(client.query_records(
            &dataset_id,
            read_json_arg(&filter, "--filter")?,
            limit,
            cursor.as_deref(),
            sort.as_deref(),
        )?),
        RecordCmd::Get { record_id } => emit(client.get_record(&record_id)?),
        RecordCmd::Create { dataset_id, data } => {
            emit(client.create_records(&dataset_id, read_json_arg(&data, "--data")?)?)
        }
        RecordCmd::Update {
            record_id,
            data,
            replace,
        } => emit(client.update_record(&record_id, read_json_arg(&data, "--data")?, replace)?),
        RecordCmd::Delete { record_id } => emit(client.delete_record(&record_id)?),
        RecordCmd::DeleteMany { record_ids } => emit(client.delete_records(&record_ids)?),
    }
}

fn run_settings(cmd: SettingsCmd) -> Result<()> {
    match cmd {
        SettingsCmd::Show => emit(settings::payload()),
        SettingsCmd::SetKey { token } => emit(settings::set_key(&token)?),
        SettingsCmd::ClearKey => emit(settings::clear_key()?),
        SettingsCmd::Test => emit(settings::test()?),
    }
}

/// A JSON argument, or the whole of stdin when the caller passed `-`.
fn read_json_arg(raw: &str, flag: &str) -> Result<Value> {
    let text = if raw == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        raw.to_string()
    };
    if text.trim().is_empty() {
        return Err(DatifyError::Config(format!("{flag} je prázdný.")));
    }
    serde_json::from_str(&text)
        .map_err(|e| DatifyError::Config(format!("{flag} není platné JSON: {e}")))
}

fn emit(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
