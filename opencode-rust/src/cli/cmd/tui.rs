use std::path::PathBuf;

use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct TuiCommand {
    /// Project directory
    pub project: Option<PathBuf>,
    /// Model to use in provider/model format
    #[arg(short, long)]
    pub model: Option<String>,
    /// Continue the last session
    #[arg(short, long = "continue")]
    pub r#continue: bool,
    /// Session id to continue
    #[arg(short, long)]
    pub session: Option<String>,
    /// Prompt to use
    #[arg(short, long)]
    pub prompt: Option<String>,
    /// Agent to use
    #[arg(long)]
    pub agent: Option<String>,
    /// Port to bind
    #[arg(long, default_value_t = 0)]
    pub port: u16,
    /// Hostname to bind
    #[arg(short, long, default_value = "127.0.0.1")]
    pub hostname: String,
}

pub async fn execute(cmd: &TuiCommand) -> anyhow::Result<()> {
    info!(
        project = ?cmd.project,
        model = ?cmd.model,
        continue_session = cmd.r#continue,
        session = ?cmd.session,
        prompt = ?cmd.prompt,
        agent = ?cmd.agent,
        port = cmd.port,
        hostname = %cmd.hostname,
        "tui command"
    );
    Ok(())
}
