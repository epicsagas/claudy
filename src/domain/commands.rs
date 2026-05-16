#[derive(Debug, Clone, Default)]
pub struct Options {
    pub help: bool,
    pub version: bool,
}

#[derive(Debug, Clone)]
pub enum DomainCommand {
    List,
    Setup {
        provider: Option<String>,
    },
    Show {
        profile: String,
    },
    Ping {
        profile: Option<String>,
    },
    Doctor,
    Sync,
    Update,
    Uninstall,
    Mode {
        action: String,
        name: Option<String>,
    },
    Channel {
        action: ChannelAction,
        profile: Option<String>,
        listen: Option<String>,
    },
    Mcp(McpAction),
    Analytics(AnalyticsAction),
    Session(SessionAction),
}

#[derive(Debug, Clone)]
pub enum McpAction {
    Run,
    Install,
    Uninstall,
}

#[derive(Debug, Clone)]
pub enum ChannelAction {
    Serve,
    Start,
    Stop,
    Restart,
    Status,
    Add { platform: String },
    Remove { platform: String },
    Enable,
    Disable,
}

#[derive(Debug, Clone)]
pub enum AnalyticsAction {
    Dashboard,
    Ingest {
        full: bool,
        project: Option<String>,
    },
    Recommend,
    Export {
        format: String,
        project: Option<String>,
        days: u32,
    },
    SyncPricing,
    Insights {
        days: u32,
        from: Option<String>,
        to: Option<String>,
        project: Option<String>,
    },
    Recalculate,
}

#[derive(Debug, Clone)]
pub enum SessionAction {
    Sanitize {
        project: Option<String>,
        all: bool,
        yes: bool,
    },
}
