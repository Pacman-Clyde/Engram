pub mod completions;
pub mod context;
pub mod decision;
pub mod export;
pub mod file;
pub mod init;
pub mod search;
pub mod session;
pub mod status;
pub mod task;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "engram", version, about = "AI agent memory system")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize engram in current project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Project description
        #[arg(long)]
        description: Option<String>,
    },

    /// Manage architectural decisions
    Decision {
        #[command(subcommand)]
        action: DecisionAction,
    },

    /// Manage project tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// Manage file summaries
    File {
        #[command(subcommand)]
        action: FileAction,
    },

    /// Manage work sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Search across all memory
    Search {
        /// Search query
        query: String,
        /// Filter by type: decision, task, file, session
        #[arg(long, short = 't')]
        entity_type: Option<String>,
    },

    /// Get token-efficient project context
    Context {
        /// Role: build, review, debug, resume
        #[arg(long, short, default_value = "build")]
        role: String,
        /// Level: minimal, standard, full
        #[arg(long, short, default_value = "standard")]
        level: String,
    },

    /// Compact project status overview
    Status,

    /// Export all engram data to JSON
    Export {
        /// Output file path (prints to stdout if omitted)
        path: Option<String>,
    },

    /// Import engram data from JSON
    Import {
        /// Input file path
        path: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell: bash, zsh, fish, elvish, powershell
        shell: String,
    },

    /// Start MCP server (stdio transport)
    Serve,
}

#[derive(Subcommand)]
pub enum DecisionAction {
    /// Record a new decision
    Add {
        /// Decision title
        title: String,
        /// Context / problem statement
        #[arg(long)]
        context: String,
        /// The decision made
        #[arg(long)]
        decision: String,
        /// Alternatives considered (comma-separated)
        #[arg(long)]
        alternatives: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// List decisions
    List {
        /// Filter by status: active, superseded, reverted
        #[arg(long)]
        status: Option<String>,
    },
    /// Show decision details
    Show {
        /// Decision ID (prefix match)
        id: String,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Task description
        #[arg(long, default_value = "")]
        description: String,
        /// Priority: low, medium, high, critical
        #[arg(long, short, default_value = "medium")]
        priority: String,
        /// Phase label
        #[arg(long)]
        phase: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Update task status
    Update {
        /// Task ID (prefix match)
        id: String,
        /// New status: todo, in_progress, done, blocked
        #[arg(long)]
        status: String,
    },
    /// List tasks
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum FileAction {
    /// Add or update a file summary
    Summarize {
        /// File path
        path: String,
        /// Summary text
        #[arg(long)]
        summary: String,
        /// Key types (comma-separated)
        #[arg(long)]
        key_types: Option<String>,
        /// Dependencies (comma-separated)
        #[arg(long)]
        deps: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// List file summaries
    List,
    /// Check if a file summary is stale
    Check {
        /// File path
        path: String,
    },
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Start a new work session
    Start {
        /// Agent name
        #[arg(long, default_value = "unknown")]
        agent: String,
        /// Session goal
        goal: String,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// End current session with handoff notes
    End {
        /// Handoff notes for next agent
        handoff: String,
    },
    /// List recent sessions
    List {
        /// Max sessions to show
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

pub fn parse_csv(s: &Option<String>) -> Vec<String> {
    s.as_deref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default()
}

pub fn dispatch(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init { name, description } => init::run(name, description),
        Commands::Decision { action } => decision::run(action),
        Commands::Task { action } => task::run(action),
        Commands::File { action } => file::run(action),
        Commands::Session { action } => session::run(action),
        Commands::Search { query, entity_type } => search::run(&query, entity_type.as_deref()),
        Commands::Context { role, level } => context::run(&role, &level),
        Commands::Status => status::run(),
        Commands::Export { path } => export::run_export(path.as_deref()),
        Commands::Import { path } => export::run_import(&path),
        Commands::Completions { shell } => completions::run(&shell),
        Commands::Serve => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(crate::mcp::serve())
        }
    }
}

