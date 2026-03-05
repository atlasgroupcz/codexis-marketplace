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
    /// Tabular data extraction from files
    Tabular {
        #[command(subcommand)]
        command: TabularCommands,
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
        /// Marketplace name
        name: String,
    },
    /// Update marketplace(s) — pull latest from git
    Update {
        /// Marketplace name (omit to update all)
        name: Option<String>,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List plugins
    List {
        /// Filter by marketplace name
        #[arg(long)]
        marketplace: Option<String>,
        /// Show available (not installed) plugins
        #[arg(long, default_value = "false")]
        available: bool,
    },
    /// Install a plugin
    Install {
        /// Marketplace name
        #[arg(long)]
        marketplace: String,
        /// Plugin name
        #[arg(long)]
        name: String,
    },
    /// Uninstall a plugin
    Uninstall {
        /// Marketplace name
        #[arg(long)]
        marketplace: String,
        /// Plugin name
        #[arg(long)]
        name: String,
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
            AutomationCommands::Delete { id } => {
                commands::automation::delete(&client, &id, format)
            }
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
            MarketplaceCommands::Remove { name } => {
                commands::marketplace::remove(&client, &name, format)
            }
            MarketplaceCommands::Update { name } => {
                commands::marketplace::update(&client, name.as_deref(), format)
            }
        },
        Commands::Plugin { command } => match command {
            PluginCommands::List {
                marketplace,
                available,
            } => commands::plugin::list(&client, marketplace.as_deref(), available, format),
            PluginCommands::Install { marketplace, name } => {
                commands::plugin::install(&client, &marketplace, &name, format)
            }
            PluginCommands::Uninstall { marketplace, name } => {
                commands::plugin::uninstall(&client, &marketplace, &name, format)
            }
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
            TabularCommands::Start { folder } => {
                commands::tabular::start(&client, &folder, format)
            }
            TabularCommands::Results { folder } => {
                commands::tabular::results(&client, &folder, format)
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }
}
