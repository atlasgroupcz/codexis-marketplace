mod client;
mod commands;
mod error;
mod graphql;
mod output;

use clap::{Parser, Subcommand};
use client::GraphQLClient;
use output::OutputFormat;
use std::process;

/// cdxctl — kubectl-style CLI for cdx-daemon platform management
#[derive(Parser)]
#[command(name = "cdxctl", version, about)]
struct Cli {
    /// Output format: json (default) or table
    #[arg(long, default_value = "false")]
    table: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage automations
    Automation {
        #[command(subcommand)]
        command: AutomationCommands,
    },
    /// Manage marketplaces
    Marketplace {
        #[command(subcommand)]
        command: MarketplaceCommands,
    },
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
    /// Manage agents
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// Manage skills
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    /// Tabular data extraction from files
    Tabular {
        #[command(subcommand)]
        command: TabularCommands,
    },
    /// Manage notifications
    Notification {
        #[command(subcommand)]
        command: NotificationCommands,
    },
}

#[derive(Subcommand)]
enum AutomationCommands {
    /// List all automations
    List,
    /// Create a new automation
    Create {
        /// Automation title
        #[arg(long)]
        title: String,
        /// Cron expression (5-field unix cron)
        #[arg(long)]
        cron: String,
        /// Prompt to execute
        #[arg(long)]
        prompt: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Agent full name (plugin:agent)
        #[arg(long)]
        agent: Option<String>,
        /// Skill full names (plugin:skill), repeatable
        #[arg(long)]
        skill: Vec<String>,
        /// Maximum agent turns
        #[arg(long)]
        max_turns: Option<u32>,
        /// Working directory path
        #[arg(long)]
        work_dir: Option<String>,
        /// Create in disabled state
        #[arg(long, default_value = "false")]
        disabled: bool,
    },
    /// Create a new COMMAND automation
    CreateCommand {
        /// Automation title
        #[arg(long)]
        title: String,
        /// Cron expression (5-field unix cron)
        #[arg(long)]
        cron: String,
        /// Shell command to execute
        #[arg(long)]
        command: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Create in disabled state
        #[arg(long, default_value = "false")]
        disabled: bool,
    },
    /// Update an existing automation (partial update)
    Update {
        /// Automation ID (Node ID or UUID)
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        cron: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        skill: Option<Vec<String>>,
        #[arg(long)]
        max_turns: Option<u32>,
        #[arg(long)]
        work_dir: Option<String>,
        /// Enable the automation
        #[arg(long)]
        enabled: Option<bool>,
    },
    /// Delete an automation
    Delete {
        /// Automation ID (Node ID or UUID)
        id: String,
    },
    /// Manually trigger an automation run
    Trigger {
        /// Automation ID (Node ID or UUID)
        id: String,
    },
}

#[derive(Subcommand)]
enum MarketplaceCommands {
    /// List all marketplaces
    List,
    /// Add a marketplace source
    Add {
        /// Source URL (git) or path (local)
        #[arg(long)]
        source: String,
        /// Source type: git or local
        #[arg(long, name = "type")]
        source_type: String,
        /// Git branch or tag reference
        #[arg(long, name = "ref")]
        git_ref: Option<String>,
    },
    /// Remove a marketplace
    Remove {
        /// Marketplace ID
        id: String,
    },
    /// Update marketplace(s) — pull latest from git
    Update {
        /// Marketplace ID (omit to update all)
        id: Option<String>,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List plugins
    List {
        /// Filter by marketplace ID
        #[arg(long)]
        marketplace: Option<String>,
        /// Show available (not installed) plugins
        #[arg(long, default_value = "false")]
        available: bool,
    },
    /// Install a plugin
    Install {
        /// Plugin ID
        id: String,
    },
    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID
        id: String,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// List all agents
    List {
        /// Show only editable custom agents
        #[arg(long, default_value = "false")]
        editable_only: bool,
    },
    /// Create an agent from markdown content
    Create {
        /// Read agent markdown content from a file
        #[arg(long)]
        file: Option<String>,
        /// Read agent markdown content from stdin
        #[arg(long, default_value = "false")]
        stdin: bool,
    },
    /// Update an agent from markdown content
    Update {
        /// Agent ID, node ID, or raw agent name
        id: String,
        /// Read agent markdown content from a file
        #[arg(long)]
        file: Option<String>,
        /// Read agent markdown content from stdin
        #[arg(long, default_value = "false")]
        stdin: bool,
    },
    /// Delete an agent
    Delete {
        /// Agent ID, node ID, or raw agent name
        id: String,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    /// List all skills
    List {
        /// Show only editable custom skills
        #[arg(long, default_value = "false")]
        editable_only: bool,
    },
    /// Create a skill from SKILL.md content
    Create {
        /// Read SKILL.md content from a file
        #[arg(long)]
        file: Option<String>,
        /// Read SKILL.md content from stdin
        #[arg(long, default_value = "false")]
        stdin: bool,
    },
    /// Update a skill from SKILL.md content
    Update {
        /// Skill ID, node ID, or raw skill name
        id: String,
        /// Read SKILL.md content from a file
        #[arg(long)]
        file: Option<String>,
        /// Read SKILL.md content from stdin
        #[arg(long, default_value = "false")]
        stdin: bool,
    },
    /// Delete a skill
    Delete {
        /// Skill ID, node ID, or raw skill name
        id: String,
    },
}

#[derive(Subcommand)]
enum TabularCommands {
    /// Show extraction status, columns, and progress for a folder
    Status {
        /// Folder path (sandbox path)
        folder: String,
    },
    /// Add a column to the extraction
    AddColumn {
        /// Folder path (sandbox path)
        folder: String,
        /// Column name
        #[arg(long)]
        name: String,
        /// Column type: text, boolean, date, number, currency, list, tag, tags
        #[arg(long, name = "type")]
        col_type: String,
        /// Column description (guides AI extraction)
        #[arg(long)]
        description: Option<String>,
        /// Tag options in 'value:COLOR' format (for tag/tags types), repeatable
        #[arg(long)]
        option: Vec<String>,
    },
    /// Remove a column from the extraction
    RemoveColumn {
        /// Folder path (sandbox path)
        folder: String,
        /// Column ID
        #[arg(long)]
        column_id: String,
    },
    /// Start the extraction process
    Start {
        /// Folder path (sandbox path)
        folder: String,
    },
    /// Get extraction results (flattened rows)
    Results {
        /// Folder path (sandbox path)
        folder: String,
    },
}

#[derive(Subcommand)]
enum NotificationCommands {
    /// Create a new notification
    Create {
        /// Notification message
        #[arg(short, long)]
        message: String,
        /// Optional shell action to execute on refresh
        #[arg(short, long)]
        action: Option<String>,
        /// Optional link URL (clicking navigates here and marks as confirmed)
        #[arg(short, long)]
        link: Option<String>,
        /// Extra key-value pairs (e.g. --extra key=value)
        #[arg(long, value_parser = parse_key_value)]
        extra: Vec<(String, String)>,
    },
    /// List notifications
    List {
        /// Number of days to show (default: 7)
        #[arg(long, default_value = "7")]
        days: u32,
        /// Show only unseen notifications
        #[arg(long)]
        unseen: bool,
    },
    /// Mark a notification as seen
    Seen {
        /// Notification ID
        id: String,
    },
    /// Mark a notification as confirmed
    Confirm {
        /// Notification ID
        id: String,
    },
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() {
    let cli = Cli::parse();
    let client = GraphQLClient::new();
    let format = OutputFormat::from_flag(cli.table);

    let result = match cli.command {
        Commands::Automation { command } => match command {
            AutomationCommands::List => commands::automation::list(&client, format),
            AutomationCommands::Create {
                title,
                cron,
                prompt,
                description,
                agent,
                skill,
                max_turns,
                work_dir,
                disabled,
            } => commands::automation::create(
                &client,
                &title,
                &cron,
                &prompt,
                description.as_deref(),
                agent.as_deref(),
                &skill,
                max_turns,
                work_dir.as_deref(),
                disabled,
                format,
            ),
            AutomationCommands::CreateCommand {
                title,
                cron,
                command,
                description,
                disabled,
            } => commands::automation::create_command(
                &client,
                &title,
                &cron,
                &command,
                description.as_deref(),
                disabled,
                format,
            ),
            AutomationCommands::Update {
                id,
                title,
                cron,
                prompt,
                description,
                agent,
                skill,
                max_turns,
                work_dir,
                enabled,
            } => commands::automation::update(
                &client,
                &id,
                title.as_deref(),
                cron.as_deref(),
                prompt.as_deref(),
                description.as_deref(),
                agent.as_deref(),
                skill.as_deref(),
                max_turns,
                work_dir.as_deref(),
                enabled,
                format,
            ),
            AutomationCommands::Delete { id } => commands::automation::delete(&client, &id, format),
            AutomationCommands::Trigger { id } => {
                commands::automation::trigger(&client, &id, format)
            }
        },
        Commands::Marketplace { command } => match command {
            MarketplaceCommands::List => commands::marketplace::list(&client, format),
            MarketplaceCommands::Add {
                source,
                source_type,
                git_ref,
            } => commands::marketplace::add(
                &client,
                &source,
                &source_type,
                git_ref.as_deref(),
                format,
            ),
            MarketplaceCommands::Remove { id } => {
                commands::marketplace::remove(&client, &id, format)
            }
            MarketplaceCommands::Update { id } => {
                commands::marketplace::update(&client, id.as_deref(), format)
            }
        },
        Commands::Plugin { command } => match command {
            PluginCommands::List {
                marketplace,
                available,
            } => commands::plugin::list(&client, marketplace.as_deref(), available, format),
            PluginCommands::Install { id } => {
                commands::plugin::install(&client, &id, format)
            }
            PluginCommands::Uninstall { id } => {
                commands::plugin::uninstall(&client, &id, format)
            }
        },
        Commands::Agent { command } => match command {
            AgentCommands::List { editable_only } => {
                commands::agent::list(&client, editable_only, format)
            }
            AgentCommands::Create { file, stdin } => {
                commands::agent::create(&client, file.as_deref(), stdin, format)
            }
            AgentCommands::Update { id, file, stdin } => {
                commands::agent::update(&client, &id, file.as_deref(), stdin, format)
            }
            AgentCommands::Delete { id } => commands::agent::delete(&client, &id, format),
        },
        Commands::Skill { command } => match command {
            SkillCommands::List { editable_only } => {
                commands::skill::list(&client, editable_only, format)
            }
            SkillCommands::Create { file, stdin } => {
                commands::skill::create(&client, file.as_deref(), stdin, format)
            }
            SkillCommands::Update { id, file, stdin } => {
                commands::skill::update(&client, &id, file.as_deref(), stdin, format)
            }
            SkillCommands::Delete { id } => commands::skill::delete(&client, &id, format),
        },
        Commands::Tabular { command } => match command {
            TabularCommands::Status { folder } => {
                commands::tabular::status(&client, &folder, format)
            }
            TabularCommands::AddColumn {
                folder,
                name,
                col_type,
                description,
                option,
            } => commands::tabular::add_column(
                &client,
                &folder,
                &name,
                &col_type,
                description.as_deref(),
                &option,
                format,
            ),
            TabularCommands::RemoveColumn { folder, column_id } => {
                commands::tabular::remove_column(&client, &folder, &column_id, format)
            }
            TabularCommands::Start { folder } => commands::tabular::start(&client, &folder, format),
            TabularCommands::Results { folder } => {
                commands::tabular::results(&client, &folder, format)
            }
        },
        Commands::Notification { command } => match command {
            NotificationCommands::Create {
                message,
                action,
                link,
                extra,
            } => commands::notification::create(
                &client,
                &message,
                action.as_deref(),
                link.as_deref(),
                &extra,
                format,
            ),
            NotificationCommands::List { days, unseen } => {
                commands::notification::list(&client, days, unseen, format)
            }
            NotificationCommands::Seen { id } => {
                commands::notification::seen(&client, &id, format)
            }
            NotificationCommands::Confirm { id } => {
                commands::notification::confirm(&client, &id, format)
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_list_command() {
        let cli =
            Cli::try_parse_from(["cdxctl", "agent", "list"]).expect("agent list should parse");

        match cli.command {
            Commands::Agent {
                command: AgentCommands::List { editable_only },
            } => {
                assert!(!editable_only);
            }
            _ => panic!("expected agent list command"),
        }
    }

    #[test]
    fn parses_agent_update_with_file_input() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "agent",
            "update",
            "local-agent",
            "--file",
            "/tmp/local-agent.md",
        ])
        .expect("agent update should parse");

        match cli.command {
            Commands::Agent {
                command: AgentCommands::Update { id, file, stdin },
            } => {
                assert_eq!(id, "local-agent");
                assert_eq!(file.as_deref(), Some("/tmp/local-agent.md"));
                assert!(!stdin);
            }
            _ => panic!("expected agent update command"),
        }
    }
}
