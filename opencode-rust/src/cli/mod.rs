use clap::{Parser, Subcommand, ValueEnum};

pub mod cmd;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_filter(self) -> tracing_subscriber::filter::LevelFilter {
        match self {
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "opencode",
    version,
    about = "The AI coding agent built for the terminal.",
    bin_name = "opencode"
)]
pub struct Opts {
    /// Print logs to stderr
    #[arg(long = "print-logs", default_value_t = false)]
    pub print_logs: bool,
    /// Log level
    #[arg(long = "log-level", value_enum)]
    pub log_level: Option<LogLevel>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run opencode with a message
    Run(cmd::run::Run),
    /// Generate OpenAPI spec
    Generate,
    /// Manage credentials
    Auth(cmd::auth::AuthCommand),
    /// Manage agents
    Agent(cmd::agent::AgentCommand),
    /// Upgrade opencode to a newer version
    Upgrade(cmd::upgrade::UpgradeCommand),
    /// List all available models
    Models,
    /// Starts a headless opencode server
    Serve(cmd::serve::ServeCommand),
    /// Show usage statistics
    Stats(cmd::stats::StatsCommand),
    /// Export a session as JSON
    Export(cmd::export::ExportCommand),
    /// Attach to a running opencode server
    Attach(cmd::attach::AttachCommand),
    /// Start the ACP server bridge
    Acp(cmd::acp::AcpCommand),
    /// Manage MCP integrations
    Mcp(cmd::mcp::McpCommand),
    /// Start the TUI
    Tui(cmd::tui::TuiCommand),
    /// Debug utilities
    Debug(cmd::debug::DebugCommand),
    /// Manage GitHub agent
    Github(cmd::github::GithubCommand),
}
