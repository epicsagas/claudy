use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "claudy", version, about = "Multi-provider launcher for Claude CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List configured providers
    #[command(name = "ls", alias = "list")]
    Ls,

    /// Interactive provider configuration
    #[command(name = "setup", alias = "config")]
    Setup {
        /// Provider ID to configure
        provider: Option<String>,
    },

    /// Show provider details
    #[command(name = "show", alias = "info")]
    Show {
        /// Profile name
        profile: String,
    },

    /// Test connectivity to providers
    #[command(name = "ping", alias = "test")]
    Ping {
        /// Profile name to test (all if omitted)
        profile: Option<String>,
    },

    /// Check system status and paths
    #[command(name = "doctor", alias = "status")]
    Doctor,

    /// Synchronize claudy and claude shim
    #[command(name = "sync", alias = "install")]
    Sync,

    /// Update claudy to the latest version
    Update,

    /// Uninstall claudy and remove all files
    Uninstall,

    /// Manage Claude configuration modes
    #[command(name = "mode")]
    Mode {
        /// Action: create, ls, rm
        action: String,
        /// Mode name (for create/rm)
        name: Option<String>,
    },

    /// Manage the remote code channel
    #[command(subcommand)]
    Channel(ChannelCommands),

    /// Manage MCP server for Claude Code (agent bridge)
    #[command(subcommand)]
    Mcp(McpCommands),

    /// Usage analytics and recommendations dashboard
    #[command(subcommand)]
    Analytics(AnalyticsCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum McpCommands {
    /// Run the MCP server (called by Claude Code)
    Run,
    /// Register claudy as an MCP server in Claude Code settings
    Install,
    /// Remove claudy from Claude Code MCP settings
    Uninstall,
}

#[derive(Subcommand, Debug)]
pub enum ChannelCommands {
    /// Run the channel server in the foreground
    Serve {
        /// Profile to use for Claude sessions
        #[arg(long)]
        profile: Option<String>,
        /// Listen address (overrides config)
        #[arg(long)]
        listen: Option<String>,
    },
    /// Start the channel server in the background
    Start {
        /// Profile to use for Claude sessions
        #[arg(long)]
        profile: Option<String>,
        /// Listen address (overrides config)
        #[arg(long)]
        listen: Option<String>,
    },
    /// Stop the running channel server
    Stop,
    /// Restart the channel server
    Restart {
        /// Profile to use for Claude sessions
        #[arg(long)]
        profile: Option<String>,
        /// Listen address (overrides config)
        #[arg(long)]
        listen: Option<String>,
    },
    /// Show channel server status
    Status,
    /// Add a channel platform (telegram, slack, discord)
    Add {
        /// Platform to add
        platform: String,
    },
    /// Remove a channel platform
    Remove {
        /// Platform to remove
        platform: String,
    },
    /// Enable the channel service (auto-start on login)
    Enable,
    /// Disable the channel service (stop auto-starting on login)
    Disable,
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommands {
    /// Open the analytics dashboard
    Dashboard,
    /// Ingest session data from ~/.claude/projects/
    Ingest {
        /// Re-ingest all files, ignoring checkpoints
        #[arg(long)]
        full: bool,
        /// Filter by project name
        #[arg(long)]
        project: Option<String>,
    },
    /// Show usage recommendations
    Recommend,
    /// Export analytics data
    Export {
        /// Output format: csv or json
        #[arg(long, default_value = "json")]
        format: String,
        /// Filter by project name
        #[arg(long)]
        project: Option<String>,
        /// Number of days to include
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// Sync model pricing from models.dev and Anthropic pricing page
    SyncPricing,
    /// Generate a compact JSON insights summary for LLM analysis
    Insights {
        /// Number of days to analyze (default: 7)
        #[arg(long, default_value = "7")]
        days: u32,
        /// Start date (YYYY-MM-DD), overrides --days
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD), overrides --days
        #[arg(long)]
        to: Option<String>,
        /// Filter by project name
        #[arg(long)]
        project: Option<String>,
    },
    /// Recalculate all costs using the latest pricing data
    Recalculate,
}
