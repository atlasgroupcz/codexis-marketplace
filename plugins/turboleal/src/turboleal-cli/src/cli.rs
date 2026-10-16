use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use uuid::Uuid;

/// Command-line client for the Turboleal legislation watch.
#[derive(Debug, Parser)]
#[command(name = "turboleal-cli", version, about, propagate_version = true)]
pub struct Cli {
    /// Base URL of the Turboleal API.
    #[arg(
        long,
        global = true,
        env = "CODEXIS_PLUGIN_TURBOLEAL_API_URL",
        default_value = crate::config::DEFAULT_API_URL
    )]
    pub api_url: String,

    /// Turboleal API key (api-key-...); defaults to TURBOLEAL_API_KEY, then the key saved by `auth login`.
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Output format.
    #[arg(short, long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage the Turboleal API key this tool uses.
    #[command(subcommand)]
    Auth(AuthCommand),
    /// Show the signed-in user.
    Me,
    /// Manage watch sets.
    #[command(subcommand, visible_alias = "ws")]
    WatchSets(WatchSetCommand),
    /// Manage the files attached to a watch set.
    #[command(subcommand)]
    Files(FileCommand),
    /// Show the documents and sections a watch set watches.
    Documents {
        /// Watch set id.
        watch_set: Uuid,
    },
    /// List what was found in a watch set's brief and files.
    #[command(subcommand)]
    References(ReferenceCommand),
    /// Read and acknowledge alerts about changed legislation.
    #[command(subcommand)]
    Alerts(AlertCommand),
    /// Print a shell completion script.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Check that an API key is set and Turboleal accepts it; the key itself is never printed.
    Status,
    /// Verify an API key against Turboleal and save it for later runs.
    Login {
        /// The API key; omit it or pass - to read it from standard input.
        key: Option<String>,
    },
    /// Forget the saved API key.
    Logout,
}

#[derive(Debug, Args)]
pub struct PageArgs {
    /// Number of items per page (1-200).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=200))]
    pub limit: Option<u16>,
    /// Continue after this cursor (the nextCursor of the previous page).
    #[arg(long, conflicts_with = "all")]
    pub after: Option<Uuid>,
    /// Fetch every page.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Subcommand)]
pub enum WatchSetCommand {
    /// List your watch sets, newest first.
    List(#[command(flatten)] PageArgs),
    /// Show one watch set.
    Get { id: Uuid },
    /// Create a watch set.
    Create {
        /// Name of the watch set.
        #[arg(long)]
        name: String,
        /// What to watch, in your own words; @FILE reads it from a file.
        #[arg(long)]
        brief: String,
    },
    /// Change the name or the brief of a watch set.
    #[command(arg_required_else_help = true)]
    Update {
        id: Uuid,
        #[arg(long)]
        name: Option<String>,
        /// New brief; @FILE reads it from a file.
        #[arg(long)]
        brief: Option<String>,
    },
    /// Delete a watch set with its files, references and alerts.
    Delete { id: Uuid },
}

#[derive(Debug, Subcommand)]
pub enum FileCommand {
    /// List the files of a watch set, newest first.
    List {
        watch_set: Uuid,
        #[command(flatten)]
        page: PageArgs,
    },
    /// Upload files; directories are uploaded recursively and keep their folder structure.
    Upload {
        watch_set: Uuid,
        /// Files or directories to upload.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Folder to put the uploaded files under, for example contracts/2026.
        #[arg(long)]
        folder: Option<String>,
    },
    /// Remove a file from a watch set.
    Delete { watch_set: Uuid, file: Uuid },
}

#[derive(Debug, Subcommand)]
pub enum ReferenceCommand {
    /// List references, newest first.
    List {
        watch_set: Uuid,
        /// Only references that could not be matched to a CODEXIS document.
        #[arg(long)]
        unlinked: bool,
        #[command(flatten)]
        page: PageArgs,
    },
}

#[derive(Debug, Subcommand)]
pub enum AlertCommand {
    /// List alerts, newest first.
    List {
        /// Only alerts of this watch set.
        #[arg(long)]
        watch_set: Option<Uuid>,
        /// Only alerts that were not acknowledged yet.
        #[arg(long)]
        unacknowledged: bool,
        #[command(flatten)]
        page: PageArgs,
    },
    /// Show one alert with its diff.
    Get { id: Uuid },
    /// Mark an alert as handled; the section stays watched.
    #[command(visible_alias = "acknowledge")]
    Ack { id: Uuid },
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
