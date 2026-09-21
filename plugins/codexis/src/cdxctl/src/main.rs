mod client;
mod commands;
mod error;
mod graphql;
mod guard;
mod output;
#[cfg(test)]
mod test_support;

use clap::{Parser, Subcommand};
use client::GraphQLClient;
use output::OutputFormat;
use std::process;

/// cdxctl - kubectl-style CLI for cdx-daemon platform management
#[derive(Parser)]
#[command(name = "cdxctl", version, about)]
struct Cli {
    /// Output format: json (default) or table
    #[arg(long, global = true)]
    table: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Manage Ubuntu packages on the underlying VM")]
    Package {
        #[command(subcommand)]
        command: commands::package::PackageCommands,
    },
    #[command(about = "Set or delete a plugin's env values (~/.cdx/apps/<app>/.env)")]
    Appenv {
        /// Plugin name, e.g. katastr
        app: String,
        #[command(subcommand)]
        command: commands::appenv::AppEnvCommands,
    },
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
    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
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
    /// Report bugs in the platform
    Bug {
        #[command(subcommand)]
        command: BugCommands,
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
        /// How often it repeats: DAILY, WEEKLY or MONTHLY
        #[arg(long)]
        repeat: Option<String>,
        /// Six-hour window it runs in: NIGHT, MORNING, AFTERNOON or EVENING (default MORNING)
        #[arg(long)]
        day_part: Option<String>,
        /// Days of week for WEEKLY (0=Sunday..6=Saturday), repeatable
        #[arg(long)]
        days_of_week: Vec<u32>,
        /// Day of month for MONTHLY (1-28)
        #[arg(long)]
        day_of_month: Option<u32>,
        /// Deprecated: 5-field unix cron, translated to the nearest day-part window
        #[arg(long)]
        cron: Option<String>,
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
        /// Model the run uses (enum name, e.g. CLAUDE_SONNET_5). Defaults to the active chat's model
        #[arg(long)]
        model: Option<String>,
        /// Jurisdiction the run operates in (CZ, SK, EU, ...), repeatable. Defaults to the active chat's jurisdictions
        #[arg(long = "jurisdiction")]
        jurisdiction: Vec<String>,
        /// Project (name or Node ID) every run works in; its files are mounted at ~/project/files. Defaults to the active chat's project
        #[arg(long)]
        project: Option<String>,
        /// Connection (cloud drive or drive name, or Node ID) mounted into every run, repeatable. Defaults to the active chat's connections
        #[arg(long = "connection")]
        connection: Vec<String>,
        #[arg(
            long = "file",
            help = "File from this VM to store in the automation's persistent workspace, repeatable"
        )]
        file: Vec<String>,
        /// Create in disabled state
        #[arg(long, default_value = "false")]
        disabled: bool,
    },
    /// Create a new COMMAND automation
    CreateCommand {
        /// Automation title
        #[arg(long)]
        title: String,
        /// How often it repeats: DAILY, WEEKLY or MONTHLY
        #[arg(long)]
        repeat: Option<String>,
        /// Six-hour window it runs in: NIGHT, MORNING, AFTERNOON or EVENING (default MORNING)
        #[arg(long)]
        day_part: Option<String>,
        /// Days of week for WEEKLY (0=Sunday..6=Saturday), repeatable
        #[arg(long)]
        days_of_week: Vec<u32>,
        /// Day of month for MONTHLY (1-28)
        #[arg(long)]
        day_of_month: Option<u32>,
        /// Deprecated: 5-field unix cron, translated to the nearest day-part window
        #[arg(long)]
        cron: Option<String>,
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
        /// How often it repeats: DAILY, WEEKLY or MONTHLY
        #[arg(long)]
        repeat: Option<String>,
        /// Six-hour window it runs in: NIGHT, MORNING, AFTERNOON or EVENING (default MORNING)
        #[arg(long)]
        day_part: Option<String>,
        /// Days of week for WEEKLY (0=Sunday..6=Saturday), repeatable
        #[arg(long)]
        days_of_week: Vec<u32>,
        /// Day of month for MONTHLY (1-28)
        #[arg(long)]
        day_of_month: Option<u32>,
        /// Deprecated: 5-field unix cron, translated to the nearest day-part window
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
        /// Model the run uses (enum name, e.g. CLAUDE_SONNET_5)
        #[arg(long)]
        model: Option<String>,
        /// Jurisdiction the run operates in (CZ, SK, EU, ...), repeatable; replaces the stored set
        #[arg(long = "jurisdiction")]
        jurisdiction: Vec<String>,
        /// Project (name or Node ID) every run works in
        #[arg(long)]
        project: Option<String>,
        /// Connection (cloud drive or drive name, or Node ID) mounted into every run, repeatable; replaces the stored set
        #[arg(long = "connection")]
        connection: Option<Vec<String>>,
        /// File from this VM to add to the automation's files, repeatable
        #[arg(long = "file")]
        file: Vec<String>,
    },
    /// Enable an automation (works for shipped plugin automations too)
    Enable {
        /// Automation ID (Node ID or UUID)
        id: String,
    },
    /// Disable an automation (works for shipped plugin automations too)
    Disable {
        /// Automation ID (Node ID or UUID)
        id: String,
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
    #[command(about = "List completed runs of an automation")]
    Runs {
        #[arg(help = "Automation ID; defaults to the current automation inside a run")]
        id: Option<String>,
        #[arg(long, default_value_t = 10, help = "Maximum runs to return")]
        limit: usize,
    },
    #[command(about = "Retrieve a run's stored conversation")]
    Conversation {
        #[arg(help = "Automation run Node ID returned by `automation runs`")]
        run_id: String,
        #[arg(
            long,
            help = "Include the raw run/chat envelope with every message and part type"
        )]
        full: bool,
    },
    #[command(about = "Files stored in an automation's persistent workspace")]
    Files {
        #[command(subcommand)]
        command: AutomationFileCommands,
    },
}

#[derive(Subcommand)]
enum AutomationFileCommands {
    /// List the files stored on an automation
    List {
        /// Automation ID (Node ID or UUID)
        id: String,
    },
    /// Add files from this VM to an automation
    Add {
        /// Automation ID (Node ID or UUID)
        id: String,
        /// File from this VM to add, repeatable
        #[arg(long = "file")]
        file: Vec<String>,
    },
    /// Remove one file from an automation
    Remove {
        /// Automation ID (Node ID or UUID)
        id: String,
        /// File name as `automation files list` prints it
        name: String,
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
    /// Update marketplace(s) - pull latest from git
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
    /// Create an agent. Needs --name and --description; the model and jurisdictions
    /// come from the active chat unless you pass them
    Create {
        /// Agent name: lowercase letters, digits and hyphens
        #[arg(long)]
        name: Option<String>,
        /// What the agent is for; the orchestrator picks agents by it
        #[arg(long)]
        description: Option<String>,
        /// Tool the agent may call, e.g. shell or readFile; repeat for more
        #[arg(long = "tool")]
        tools: Vec<String>,
        /// Tool the agent may never call; repeat for more
        #[arg(long = "disallowed-tool")]
        disallowed_tools: Vec<String>,
        /// Skill granted to the agent by full name, e.g. codexis:cdxctl; repeat for more.
        /// `cdxctl skill list` shows the names
        #[arg(long = "skill")]
        skills: Vec<String>,
        /// Jurisdiction the agent works in, e.g. CZ; repeat for more. Taken from the
        /// active chat when omitted
        #[arg(long = "jurisdiction")]
        jurisdictions: Vec<String>,
        /// Model to run on, e.g. CLAUDE_OPUS_4_8; taken from the active chat when omitted
        #[arg(long)]
        model: Option<String>,
        /// Turn budget for one run; the server applies its own when omitted
        #[arg(long)]
        max_turns: Option<u32>,
        /// The agent's system prompt, as markdown
        #[arg(long)]
        instructions: Option<String>,
        /// Read the system prompt from a markdown file instead of inline --instructions -
        /// avoids shell-escaping a long multi-line prompt. Still uses the structured save.
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Short label shown on the Agents page, as lang:text (e.g. cs:"Personalista pro smlouvy").
        /// Repeat once per language
        #[arg(long = "display-name")]
        display_name: Vec<String>,
        /// One user-facing sentence, as lang:text (e.g. cs:"Připraví a zkontroluje smlouvu.").
        /// Repeat once per language
        #[arg(long = "summary")]
        summary: Vec<String>,
        /// A ready-made prompt shown on the agent, as lang:text (e.g. cs:"Zkontroluj tuto
        /// smlouvu a shrň rizika."). Repeat once per prompt and language; order is kept.
        /// Give three per language
        #[arg(long = "example-prompt")]
        example_prompts: Vec<String>,
    },
    /// Update an agent; unset flags keep their stored value
    Update {
        /// Agent ID, node ID, or raw agent name
        id: String,
        /// Agent name: lowercase letters, digits and hyphens
        #[arg(long)]
        name: Option<String>,
        /// What the agent is for; the orchestrator picks agents by it
        #[arg(long)]
        description: Option<String>,
        /// Replaces the whole tool list; repeat for more
        #[arg(long = "tool")]
        tools: Option<Vec<String>>,
        /// Replaces the whole disallowed-tool list; repeat for more
        #[arg(long = "disallowed-tool")]
        disallowed_tools: Option<Vec<String>>,
        /// Replaces the whole skill list; repeat for more
        #[arg(long = "skill")]
        skills: Option<Vec<String>>,
        /// Replaces the whole jurisdiction list; repeat for more
        #[arg(long = "jurisdiction")]
        jurisdictions: Option<Vec<String>>,
        /// Model to run on, e.g. CLAUDE_OPUS_4_8
        #[arg(long)]
        model: Option<String>,
        /// Turn budget for one run
        #[arg(long)]
        max_turns: Option<u32>,
        /// The agent's system prompt, as markdown
        #[arg(long)]
        instructions: Option<String>,
        /// Read the system prompt from a markdown file instead of inline --instructions -
        /// avoids shell-escaping a long multi-line prompt. Still uses the structured save.
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Replaces the display name for the given languages, as lang:text. Repeat once per
        /// language. Languages you do not pass keep their stored value
        #[arg(long = "display-name")]
        display_name: Vec<String>,
        /// Replaces the summary for the given languages, as lang:text. Repeat once per language.
        /// Languages you do not pass keep their stored value
        #[arg(long = "summary")]
        summary: Vec<String>,
        /// Replaces the example prompts for the given languages, as lang:text. Repeat once per
        /// prompt and language; order is kept. Languages you do not pass keep their stored value
        #[arg(long = "example-prompt")]
        example_prompts: Vec<String>,
    },
    /// Delete an agent
    Delete {
        /// Agent ID, node ID, or raw agent name
        id: String,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// List all projects with their shared folders
    List,
    /// Show one project in full, including its instructions and folders
    Get {
        /// Project name or Node ID
        id: String,
    },
    /// Create a project
    Create {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// What the project is about
        #[arg(long)]
        description: Option<String>,
        /// Instructions the project's chats follow, inline
        #[arg(long)]
        instructions: Option<String>,
        /// Read the instructions from a file instead of inline --instructions
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Shared folder to attach, by drive name or Node ID; repeat for more
        #[arg(long = "drive")]
        drives: Vec<String>,
    },
    /// Update a project; unset flags keep their stored value
    Update {
        /// Project name or Node ID
        id: String,
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// What the project is about
        #[arg(long)]
        description: Option<String>,
        /// Instructions the project's chats follow, inline
        #[arg(long)]
        instructions: Option<String>,
        /// Read the instructions from a file instead of inline --instructions
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Replaces the whole shared-folder list, by drive name or Node ID; repeat for more
        #[arg(long = "drive")]
        drives: Vec<String>,
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
    /// Create a skill. Describe it with the field flags (--name, --description,
    /// --instructions-file …)
    Create {
        /// Skill name: lowercase letters, digits and hyphens
        #[arg(long)]
        name: Option<String>,
        /// What the skill is for; this is the AI trigger text (write it as "Use when …")
        #[arg(long)]
        description: Option<String>,
        /// The skill body (SKILL.md content below the frontmatter), inline
        #[arg(long)]
        instructions: Option<String>,
        /// Read the skill body from a file instead of inline --instructions
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Short label shown on the Skills page, as lang:text (e.g. cs:"Kontrola výpovědi").
        /// Repeat once per language
        #[arg(long = "display-name")]
        display_name: Vec<String>,
        /// One user-facing sentence, as lang:text. Repeat once per language
        #[arg(long = "summary")]
        summary: Vec<String>,
        /// A ready-made prompt shown on the skill, as lang:text. Repeat once per prompt and
        /// language; order is kept
        #[arg(long = "example-prompt")]
        example_prompts: Vec<String>,
        /// Jurisdiction the skill covers, e.g. CZ; repeat for more
        #[arg(long = "jurisdiction")]
        jurisdictions: Vec<String>,
        /// Mark the skill required for its jurisdiction(s)
        #[arg(long = "jurisdiction-required", default_value = "false")]
        jurisdiction_required: bool,
        /// Tool the skill may use; repeat for more. Leave out to allow every tool
        #[arg(long = "allowed-tool")]
        allowed_tools: Vec<String>,
    },
    /// Update a skill; unset flags keep their stored value
    Update {
        /// Skill ID, node ID, or raw skill name
        id: String,
        /// Skill name: lowercase letters, digits and hyphens
        #[arg(long)]
        name: Option<String>,
        /// What the skill is for; the AI trigger text
        #[arg(long)]
        description: Option<String>,
        /// The skill body, inline
        #[arg(long)]
        instructions: Option<String>,
        /// Read the skill body from a file instead of inline --instructions
        #[arg(long = "instructions-file")]
        instructions_file: Option<String>,
        /// Replaces the display name for the given languages, as lang:text. Repeat per language
        #[arg(long = "display-name")]
        display_name: Vec<String>,
        /// Replaces the summary for the given languages, as lang:text. Repeat per language
        #[arg(long = "summary")]
        summary: Vec<String>,
        /// Replaces the example prompts for the given languages, as lang:text. Repeat once per
        /// prompt and language; order is kept
        #[arg(long = "example-prompt")]
        example_prompts: Vec<String>,
        /// Replaces the whole jurisdiction list; repeat for more
        #[arg(long = "jurisdiction")]
        jurisdictions: Vec<String>,
        /// Mark the skill required for its jurisdiction(s)
        #[arg(long = "jurisdiction-required")]
        jurisdiction_required: Option<bool>,
        /// Replaces the whole allowed-tool list; repeat for more
        #[arg(long = "allowed-tool")]
        allowed_tools: Vec<String>,
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
        /// Folder path inside the isolation (sandbox path)
        folder: String,
        /// Read the folder in this project's files instead of the current chat
        #[arg(long)]
        project: Option<String>,
    },
    /// Add a column to the extraction
    AddColumn {
        /// Folder path inside the isolation (sandbox path)
        folder: String,
        /// Read the folder in this project's files instead of the current chat
        #[arg(long)]
        project: Option<String>,
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
        /// Folder path inside the isolation (sandbox path)
        folder: String,
        /// Read the folder in this project's files instead of the current chat
        #[arg(long)]
        project: Option<String>,
        /// Column ID
        #[arg(long)]
        column_id: String,
    },
    /// Start the extraction process
    Start {
        /// Folder path inside the isolation (sandbox path)
        folder: String,
        /// Read the folder in this project's files instead of the current chat
        #[arg(long)]
        project: Option<String>,
        /// Block until the extraction reaches DONE or FAILED instead of returning immediately
        #[arg(long, default_value = "false")]
        wait: bool,
    },
    /// Get extraction results (flattened rows)
    Results {
        /// Folder path inside the isolation (sandbox path)
        folder: String,
        /// Read the folder in this project's files instead of the current chat
        #[arg(long)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum NotificationCommands {
    /// Create a new notification
    Create {
        /// Notification message
        #[arg(short, long)]
        message: String,
        /// Optional link URL (clicking navigates here and marks as confirmed)
        #[arg(short, long)]
        link: Option<String>,
        /// Extra key-value pairs (e.g. --extra key=value)
        #[arg(long, value_parser = parse_key_value)]
        extra: Vec<(String, String)>,
    },
    /// List notifications
    List {
        /// How many to fetch, newest first (default 50, the daemon's own page size)
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Skip this many before listing, to reach older ones
        #[arg(long, default_value = "0")]
        offset: u32,
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
    /// Outbound email notifications
    Email {
        #[command(subcommand)]
        command: EmailCommands,
    },
}

#[derive(Subcommand)]
enum EmailCommands {
    /// Send an email; omit --to to default to the signed-in user (recipient from JWT)
    Send {
        /// Recipient email address (repeatable); when omitted the daemon falls back to the signed-in user
        #[arg(long)]
        to: Vec<String>,
        /// Optional CC recipient (repeatable)
        #[arg(long)]
        cc: Vec<String>,
        /// Optional BCC recipient (repeatable)
        #[arg(long)]
        bcc: Vec<String>,
        /// Subject line
        #[arg(long)]
        subject: String,
        /// Inline body text (mutually exclusive with --body-file)
        #[arg(long)]
        body: Option<String>,
        /// Read body from file ("-" reads stdin); mutually exclusive with --body
        #[arg(long)]
        body_file: Option<String>,
        /// Optional HTML body (sent alongside text body for multipart/alternative)
        #[arg(long)]
        body_html: Option<String>,
        /// Attach a local file (repeatable)
        #[arg(long = "attach")]
        attachments: Vec<String>,
        /// Do not thread the mail into the current chat (a reply will start a new chat instead)
        #[arg(long)]
        no_thread: bool,
    },
}

#[derive(Subcommand)]
enum BugCommands {
    /// File a bug report. Append-only: it cannot be edited, listed or deleted afterwards
    Report {
        /// One-line summary of what is broken
        #[arg(long)]
        title: String,
        /// What happened, what was expected, and how to reproduce it
        #[arg(long)]
        description: String,
        /// How bad it is: LOW, MEDIUM, HIGH or CRITICAL
        #[arg(long)]
        severity: String,
    },
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// A repeatable flag that was never given arrives as an empty Vec, which on create
/// must stay "unset" rather than "explicitly empty".
fn none_if_empty(values: Vec<String>) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = preflight_guard(&cli.command) {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }

    let client = GraphQLClient::new();
    let format = OutputFormat::from_flag(cli.table);

    let result = match cli.command {
        Commands::Package { command } => commands::package::execute(&client, command, format),
        Commands::Appenv { app, command } => {
            commands::appenv::execute(&client, &app, command, format)
        }
        Commands::Automation { command } => match command {
            AutomationCommands::List => commands::automation::list(&client, format),
            AutomationCommands::Create {
                title,
                repeat,
                day_part,
                days_of_week,
                day_of_month,
                cron,
                prompt,
                description,
                agent,
                skill,
                max_turns,
                model,
                jurisdiction,
                project,
                connection,
                file,
                disabled,
            } => commands::automation::create(
                &client,
                commands::automation::CreateArgs {
                    title: &title,
                    schedule: commands::schedule::ScheduleArgs {
                        repeat: repeat.as_deref(),
                        day_part: day_part.as_deref(),
                        days_of_week: &days_of_week,
                        day_of_month,
                        cron: cron.as_deref(),
                    },
                    prompt: &prompt,
                    description: description.as_deref(),
                    agent: agent.as_deref(),
                    skills: &skill,
                    max_turns,
                    model: model.as_deref(),
                    jurisdictions: if jurisdiction.is_empty() {
                        None
                    } else {
                        Some(&jurisdiction)
                    },
                    project: project.as_deref(),
                    connections: &connection,
                    files: &file,
                    disabled,
                },
                format,
            ),
            AutomationCommands::CreateCommand {
                title,
                repeat,
                day_part,
                days_of_week,
                day_of_month,
                cron,
                command,
                description,
                disabled,
            } => commands::automation::create_command(
                &client,
                &title,
                commands::schedule::ScheduleArgs {
                    repeat: repeat.as_deref(),
                    day_part: day_part.as_deref(),
                    days_of_week: &days_of_week,
                    day_of_month,
                    cron: cron.as_deref(),
                },
                &command,
                description.as_deref(),
                disabled,
                format,
            ),
            AutomationCommands::Update {
                id,
                title,
                repeat,
                day_part,
                days_of_week,
                day_of_month,
                cron,
                prompt,
                description,
                agent,
                skill,
                max_turns,
                model,
                jurisdiction,
                project,
                connection,
                file,
            } => commands::automation::update(
                &client,
                commands::automation::UpdateArgs {
                    id: &id,
                    title: title.as_deref(),
                    schedule: commands::schedule::ScheduleArgs {
                        repeat: repeat.as_deref(),
                        day_part: day_part.as_deref(),
                        days_of_week: &days_of_week,
                        day_of_month,
                        cron: cron.as_deref(),
                    },
                    prompt: prompt.as_deref(),
                    description: description.as_deref(),
                    agent: agent.as_deref(),
                    skills: skill.as_deref(),
                    max_turns,
                    model: model.as_deref(),
                    jurisdictions: if jurisdiction.is_empty() {
                        None
                    } else {
                        Some(&jurisdiction)
                    },
                    project: project.as_deref(),
                    connections: connection.as_deref(),
                    files: &file,
                },
                format,
            ),
            AutomationCommands::Enable { id } => {
                commands::automation::set_enabled(&client, &id, true, format)
            }
            AutomationCommands::Disable { id } => {
                commands::automation::set_enabled(&client, &id, false, format)
            }
            AutomationCommands::Delete { id } => commands::automation::delete(&client, &id, format),
            AutomationCommands::Trigger { id } => {
                commands::automation::trigger(&client, &id, format)
            }
            AutomationCommands::Runs { id, limit } => {
                commands::automation::runs(&client, id.as_deref(), limit, format)
            }
            AutomationCommands::Conversation { run_id, full } => {
                commands::automation::conversation(&client, &run_id, full, format)
            }
            AutomationCommands::Files { command } => match command {
                AutomationFileCommands::List { id } => {
                    commands::automation::list_files(&client, &id, format)
                }
                AutomationFileCommands::Add { id, file } => {
                    commands::automation::add_files(&client, &id, &file, format)
                }
                AutomationFileCommands::Remove { id, name } => {
                    commands::automation::remove_file(&client, &id, &name, format)
                }
            },
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
            PluginCommands::Install { id } => commands::plugin::install(&client, &id, format),
            PluginCommands::Uninstall { id } => commands::plugin::uninstall(&client, &id, format),
        },
        Commands::Agent { command } => match command {
            AgentCommands::List { editable_only } => {
                commands::agent::list(&client, editable_only, format)
            }
            AgentCommands::Create {
                name,
                description,
                tools,
                disallowed_tools,
                skills,
                jurisdictions,
                model,
                max_turns,
                instructions,
                instructions_file,
                display_name,
                summary,
                example_prompts,
            } => commands::agent::create(
                &client,
                instructions_file.as_deref(),
                commands::agent::AgentFields {
                    name,
                    description,
                    tools: none_if_empty(tools),
                    disallowed_tools: none_if_empty(disallowed_tools),
                    skills: none_if_empty(skills),
                    jurisdictions: none_if_empty(jurisdictions),
                    model,
                    max_turns,
                    instructions,
                    display_name,
                    summary,
                    example_prompts,
                },
                format,
            ),
            AgentCommands::Update {
                id,
                name,
                description,
                tools,
                disallowed_tools,
                skills,
                jurisdictions,
                model,
                max_turns,
                instructions,
                instructions_file,
                display_name,
                summary,
                example_prompts,
            } => commands::agent::update(
                &client,
                &id,
                instructions_file.as_deref(),
                commands::agent::AgentFields {
                    name,
                    description,
                    tools,
                    disallowed_tools,
                    skills,
                    jurisdictions,
                    model,
                    max_turns,
                    instructions,
                    display_name,
                    summary,
                    example_prompts,
                },
                format,
            ),
            AgentCommands::Delete { id } => commands::agent::delete(&client, &id, format),
        },
        Commands::Skill { command } => match command {
            SkillCommands::List { editable_only } => {
                commands::skill::list(&client, editable_only, format)
            }
            SkillCommands::Create {
                name,
                description,
                instructions,
                instructions_file,
                display_name,
                summary,
                example_prompts,
                jurisdictions,
                jurisdiction_required,
                allowed_tools,
            } => commands::skill::create(
                &client,
                instructions_file.as_deref(),
                commands::skill::SkillFields {
                    name,
                    description,
                    instructions,
                    display_name,
                    summary,
                    example_prompts,
                    jurisdictions: none_if_empty(jurisdictions),
                    jurisdiction_required: if jurisdiction_required {
                        Some(true)
                    } else {
                        None
                    },
                    allowed_tools: none_if_empty(allowed_tools),
                },
                format,
            ),
            SkillCommands::Update {
                id,
                name,
                description,
                instructions,
                instructions_file,
                display_name,
                summary,
                example_prompts,
                jurisdictions,
                jurisdiction_required,
                allowed_tools,
            } => commands::skill::update(
                &client,
                &id,
                instructions_file.as_deref(),
                commands::skill::SkillFields {
                    name,
                    description,
                    instructions,
                    display_name,
                    summary,
                    example_prompts,
                    jurisdictions: none_if_empty(jurisdictions),
                    jurisdiction_required,
                    allowed_tools: none_if_empty(allowed_tools),
                },
                format,
            ),
            SkillCommands::Delete { id } => commands::skill::delete(&client, &id, format),
        },
        Commands::Project { command } => match command {
            ProjectCommands::List => commands::project::list(&client, format),
            ProjectCommands::Get { id } => commands::project::get(&client, &id, format),
            ProjectCommands::Create {
                name,
                description,
                instructions,
                instructions_file,
                drives,
            } => commands::project::create(
                &client,
                instructions_file.as_deref(),
                commands::project::ProjectFields {
                    name,
                    description,
                    instructions,
                    drives: none_if_empty(drives),
                },
                format,
            ),
            ProjectCommands::Update {
                id,
                name,
                description,
                instructions,
                instructions_file,
                drives,
            } => commands::project::update(
                &client,
                &id,
                instructions_file.as_deref(),
                commands::project::ProjectFields {
                    name,
                    description,
                    instructions,
                    drives: none_if_empty(drives),
                },
                format,
            ),
        },
        Commands::Tabular { command } => {
            match command {
                TabularCommands::Status { folder, project } => {
                    commands::tabular::Folder::resolve(&folder, project.as_deref())
                        .and_then(|folder| commands::tabular::status(&client, &folder, format))
                }
                TabularCommands::AddColumn {
                    folder,
                    project,
                    name,
                    col_type,
                    description,
                    option,
                } => commands::tabular::Folder::resolve(&folder, project.as_deref()).and_then(
                    |folder| {
                        commands::tabular::add_column(
                            &client,
                            &folder,
                            &name,
                            &col_type,
                            description.as_deref(),
                            &option,
                            format,
                        )
                    },
                ),
                TabularCommands::RemoveColumn {
                    folder,
                    project,
                    column_id,
                } => commands::tabular::Folder::resolve(&folder, project.as_deref()).and_then(
                    |folder| commands::tabular::remove_column(&client, &folder, &column_id, format),
                ),
                TabularCommands::Start {
                    folder,
                    project,
                    wait,
                } => commands::tabular::Folder::resolve(&folder, project.as_deref())
                    .and_then(|folder| commands::tabular::start(&client, &folder, wait, format)),
                TabularCommands::Results { folder, project } => {
                    commands::tabular::Folder::resolve(&folder, project.as_deref())
                        .and_then(|folder| commands::tabular::results(&client, &folder, format))
                }
            }
        }
        Commands::Notification { command } => match command {
            NotificationCommands::Create {
                message,
                link,
                extra,
            } => commands::notification::create(&client, &message, link.as_deref(), &extra, format),
            NotificationCommands::List {
                limit,
                offset,
                unseen,
            } => commands::notification::list(&client, limit, offset, unseen, format),
            NotificationCommands::Seen { id } => commands::notification::seen(&client, &id, format),
            NotificationCommands::Confirm { id } => {
                commands::notification::confirm(&client, &id, format)
            }
            NotificationCommands::Email { command } => match command {
                EmailCommands::Send {
                    to,
                    cc,
                    bcc,
                    subject,
                    body,
                    body_file,
                    body_html,
                    attachments,
                    no_thread,
                } => commands::notification_email::send(
                    &client,
                    &to,
                    &cc,
                    &bcc,
                    &subject,
                    body.as_deref(),
                    body_file.as_deref(),
                    body_html.as_deref(),
                    &attachments,
                    no_thread,
                    format,
                ),
            },
        },
        Commands::Bug { command } => match command {
            BugCommands::Report {
                title,
                description,
                severity,
            } => commands::bug::report(&client, &title, &description, &severity, format),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }
}

/// Refuse recursive/invalid operations before any network or auth setup runs.
fn preflight_guard(command: &Commands) -> Result<(), error::CdxctlError> {
    if let Commands::Automation { command } = command {
        let action = match command {
            AutomationCommands::Create { .. } => Some("automation create"),
            AutomationCommands::CreateCommand { .. } => Some("automation create-command"),
            _ => None,
        };
        if let Some(action) = action {
            guard::refuse_if_in_automation(action)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_project_create_with_repeated_shared_folders() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "project",
            "create",
            "--name",
            "Audit 2026",
            "--instructions-file",
            "./pokyny.md",
            "--drive",
            "Smlouvy",
            "--drive",
            "Judikatura",
        ])
        .unwrap();

        match cli.command {
            Commands::Project {
                command:
                    ProjectCommands::Create {
                        name,
                        instructions_file,
                        drives,
                        ..
                    },
            } => {
                assert_eq!(name.as_deref(), Some("Audit 2026"));
                assert_eq!(instructions_file.as_deref(), Some("./pokyny.md"));
                assert_eq!(drives, vec!["Smlouvy", "Judikatura"]);
            }
            _ => panic!("expected project create"),
        }
    }

    #[test]
    fn project_get_and_update_need_the_project_they_act_on() {
        assert!(Cli::try_parse_from(["cdxctl", "project", "get"]).is_err());
        assert!(Cli::try_parse_from(["cdxctl", "project", "update", "--name", "x"]).is_err());
        assert!(Cli::try_parse_from(["cdxctl", "project", "list"]).is_ok());
    }

    #[test]
    fn parses_appenv_set_and_delete() {
        let cli =
            Cli::try_parse_from(["cdxctl", "appenv", "katastr", "set", "API_KEY", "abc"]).unwrap();
        match cli.command {
            Commands::Appenv { app, command } => {
                assert_eq!(app, "katastr");
                assert!(matches!(
                    command,
                    commands::appenv::AppEnvCommands::Set { ref name, ref value }
                        if name == "API_KEY" && value == "abc"
                ));
            }
            _ => panic!("expected appenv"),
        }
        assert!(Cli::try_parse_from(["cdxctl", "appenv", "katastr", "delete", "API_KEY"]).is_ok());
        assert!(Cli::try_parse_from(["cdxctl", "appenv", "katastr", "set", "API_KEY"]).is_err());
        assert!(Cli::try_parse_from(["cdxctl", "appenv", "set", "API_KEY", "abc"]).is_err());
    }

    #[test]
    fn notification_list_pages_instead_of_filtering_by_age() {
        // --days was accepted and dropped: the daemon has no date filter, and
        // stopped windowing to seven days when it moved to offset/limit.
        assert!(Cli::try_parse_from(["cdxctl", "notification", "list", "--days", "7"]).is_err());

        let cli = Cli::try_parse_from(["cdxctl", "notification", "list", "--limit", "200"])
            .expect("--limit should parse");

        match cli.command {
            Commands::Notification {
                command: NotificationCommands::List { limit, offset, .. },
            } => {
                assert_eq!(limit, 200);
                assert_eq!(offset, 0);
            }
            _ => panic!("expected notification list"),
        }
    }

    #[test]
    fn accepts_the_table_flag_after_the_subcommand() {
        // Where users put it, and where the docs imply it goes.
        for argv in [
            ["cdxctl", "skill", "list", "--table"],
            ["cdxctl", "automation", "list", "--table"],
        ] {
            let cli = Cli::try_parse_from(argv).expect("--table should parse after the subcommand");
            assert!(cli.table);
        }
    }

    #[test]
    fn accepts_the_table_flag_before_the_subcommand() {
        let cli = Cli::try_parse_from(["cdxctl", "--table", "skill", "list"])
            .expect("--table should parse before the subcommand");

        assert!(cli.table);
    }

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
    fn parses_agent_update_with_field_flags() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "agent",
            "update",
            "local-agent",
            "--description",
            "New description",
        ])
        .expect("agent update should parse");

        match cli.command {
            Commands::Agent {
                command:
                    AgentCommands::Update {
                        id, description, ..
                    },
            } => {
                assert_eq!(id, "local-agent");
                assert_eq!(description.as_deref(), Some("New description"));
            }
            _ => panic!("expected agent update command"),
        }
    }

    #[test]
    fn repeated_example_prompt_flags_keep_their_order_on_agent_and_skill() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "agent",
            "create",
            "--name",
            "hr-vypoved",
            "--description",
            "Posuzuje zakonnost vypovedi",
            "--example-prompt",
            "cs:Zkontroluj tuto smlouvu a shrn rizika.",
            "--example-prompt",
            "cs:Priprav dodatek ke smlouve.",
            "--example-prompt",
            "en:Review this contract and summarize the risks.",
        ])
        .expect("agent create should parse");

        match cli.command {
            Commands::Agent {
                command:
                    AgentCommands::Create {
                        example_prompts, ..
                    },
            } => assert_eq!(
                example_prompts,
                vec![
                    "cs:Zkontroluj tuto smlouvu a shrn rizika.",
                    "cs:Priprav dodatek ke smlouve.",
                    "en:Review this contract and summarize the risks."
                ]
            ),
            _ => panic!("expected agent create command"),
        }

        let cli = Cli::try_parse_from([
            "cdxctl",
            "skill",
            "update",
            "vypoved",
            "--example-prompt",
            "cs:Zkontroluj vypoved z pracovniho pomeru.",
        ])
        .expect("skill update should parse");

        match cli.command {
            Commands::Skill {
                command:
                    SkillCommands::Update {
                        example_prompts, ..
                    },
            } => assert_eq!(
                example_prompts,
                vec!["cs:Zkontroluj vypoved z pracovniho pomeru."]
            ),
            _ => panic!("expected skill update command"),
        }
    }

    #[test]
    fn rejects_the_removed_markdown_file_input() {
        assert!(Cli::try_parse_from([
            "cdxctl",
            "agent",
            "update",
            "local-agent",
            "--file",
            "/tmp/local-agent.md",
        ])
        .is_err());
        assert!(Cli::try_parse_from(["cdxctl", "skill", "create", "--stdin"]).is_err());
    }

    #[test]
    fn parses_automation_create_with_model_and_jurisdiction() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "automation",
            "create",
            "--title",
            "Judikatura watch",
            "--repeat",
            "DAILY",
            "--day-part",
            "MORNING",
            "--prompt",
            "Report new rulings",
            "--model",
            "CLAUDE_SONNET_5",
            "--jurisdiction",
            "CZ",
            "--jurisdiction",
            "SK",
        ])
        .expect("automation create should parse");

        match cli.command {
            Commands::Automation {
                command:
                    AutomationCommands::Create {
                        title,
                        model,
                        jurisdiction,
                        ..
                    },
            } => {
                assert_eq!(title, "Judikatura watch");
                assert_eq!(model.as_deref(), Some("CLAUDE_SONNET_5"));
                assert_eq!(jurisdiction, vec!["CZ", "SK"]);
            }
            _ => panic!("expected automation create command"),
        }
    }

    #[test]
    fn parses_automation_run_history_commands() {
        let runs = Cli::try_parse_from([
            "cdxctl",
            "automation",
            "runs",
            "automation-id",
            "--limit",
            "4",
        ])
        .expect("automation runs should parse");
        match runs.command {
            Commands::Automation {
                command: AutomationCommands::Runs { id, limit },
            } => {
                assert_eq!(id.as_deref(), Some("automation-id"));
                assert_eq!(limit, 4);
            }
            _ => panic!("expected automation runs command"),
        }

        let conversation =
            Cli::try_parse_from(["cdxctl", "automation", "conversation", "run-node-id"])
                .expect("automation conversation should parse");
        match conversation.command {
            Commands::Automation {
                command: AutomationCommands::Conversation { run_id, full },
            } => {
                assert_eq!(run_id, "run-node-id");
                assert!(!full);
            }
            _ => panic!("expected automation conversation command"),
        }

        let full_conversation = Cli::try_parse_from([
            "cdxctl",
            "automation",
            "conversation",
            "run-node-id",
            "--full",
        ])
        .expect("full automation conversation should parse");
        assert!(matches!(
            full_conversation.command,
            Commands::Automation {
                command: AutomationCommands::Conversation { run_id, full: true }
            } if run_id == "run-node-id"
        ));
    }

    #[test]
    fn rejects_the_removed_automation_work_dir_flag() {
        // The daemon dropped the automation's working directory: it now runs in
        // the chat's current working directory, carried from shell history.
        assert!(Cli::try_parse_from([
            "cdxctl",
            "automation",
            "create",
            "--title",
            "Judikatura watch",
            "--repeat",
            "DAILY",
            "--prompt",
            "Report new rulings",
            "--work-dir",
            "/x",
        ])
        .is_err());
        assert!(
            Cli::try_parse_from(["cdxctl", "automation", "update", "a1", "--work-dir", "/x",])
                .is_err()
        );
    }

    #[test]
    fn parses_agent_create_with_field_flags() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "agent",
            "create",
            "--name",
            "hr-vypoved",
            "--description",
            "Posuzuje zakonnost vypovedi",
            "--tool",
            "shell",
            "--tool",
            "readFile",
            "--skill",
            "debugger",
            "--model",
            "CLAUDE_OPUS_4_8",
            "--max-turns",
            "7",
            "--instructions",
            "Posud zakonnost vypovedi.",
        ])
        .expect("agent create should parse");

        match cli.command {
            Commands::Agent {
                command:
                    AgentCommands::Create {
                        name,
                        description,
                        tools,
                        skills,
                        model,
                        max_turns,
                        instructions,
                        ..
                    },
            } => {
                assert_eq!(name.as_deref(), Some("hr-vypoved"));
                assert_eq!(description.as_deref(), Some("Posuzuje zakonnost vypovedi"));
                assert_eq!(tools, vec!["shell", "readFile"]);
                assert_eq!(skills, vec!["debugger"]);
                assert_eq!(model.as_deref(), Some("CLAUDE_OPUS_4_8"));
                assert_eq!(max_turns, Some(7));
                assert_eq!(instructions.as_deref(), Some("Posud zakonnost vypovedi."));
            }
            _ => panic!("expected agent create command"),
        }
    }

    #[test]
    fn parses_notification_email_send_command() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "notification",
            "email",
            "send",
            "--subject",
            "hi",
            "--body",
            "hello",
            "--attach",
            "/tmp/a.txt",
        ])
        .expect("notification email send should parse");

        match cli.command {
            Commands::Notification {
                command:
                    NotificationCommands::Email {
                        command:
                            EmailCommands::Send {
                                subject,
                                body,
                                attachments,
                                ..
                            },
                    },
            } => {
                assert_eq!(subject, "hi");
                assert_eq!(body.as_deref(), Some("hello"));
                assert_eq!(attachments, vec!["/tmp/a.txt".to_string()]);
            }
            _ => panic!("expected notification email send command"),
        }
    }

    #[test]
    fn threads_notification_email_into_the_chat_unless_no_thread_is_given() {
        assert!(!no_thread_flag(&[
            "cdxctl",
            "notification",
            "email",
            "send",
            "--subject",
            "hi"
        ]));
        assert!(no_thread_flag(&[
            "cdxctl",
            "notification",
            "email",
            "send",
            "--subject",
            "hi",
            "--no-thread",
        ]));
    }

    fn no_thread_flag(args: &[&str]) -> bool {
        let cli = Cli::try_parse_from(args).expect("notification email send should parse");
        match cli.command {
            Commands::Notification {
                command:
                    NotificationCommands::Email {
                        command: EmailCommands::Send { no_thread, .. },
                    },
            } => no_thread,
            _ => panic!("expected notification email send command"),
        }
    }

    #[test]
    fn parses_bug_report_command() {
        let cli = Cli::try_parse_from([
            "cdxctl",
            "bug",
            "report",
            "--title",
            "Chat hangs",
            "--description",
            "Steps: open chat, wait",
            "--severity",
            "HIGH",
        ])
        .expect("bug report should parse");
        match cli.command {
            Commands::Bug {
                command:
                    BugCommands::Report {
                        title,
                        description,
                        severity,
                    },
            } => {
                assert_eq!(title, "Chat hangs");
                assert_eq!(description, "Steps: open chat, wait");
                assert_eq!(severity, "HIGH");
            }
            _ => panic!("expected bug report command"),
        }
    }

    #[test]
    fn requires_every_bug_report_field() {
        assert!(Cli::try_parse_from([
            "cdxctl",
            "bug",
            "report",
            "--description",
            "d",
            "--severity",
            "LOW"
        ])
        .is_err());
        assert!(Cli::try_parse_from([
            "cdxctl",
            "bug",
            "report",
            "--title",
            "t",
            "--severity",
            "LOW"
        ])
        .is_err());
        assert!(Cli::try_parse_from([
            "cdxctl",
            "bug",
            "report",
            "--title",
            "t",
            "--description",
            "d"
        ])
        .is_err());
    }
}
