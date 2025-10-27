use std::path::PathBuf;

use crate::agent::core::Agent;
use crate::agent::session::Session;
use crate::tool::echo::EchoTool;
use clap::{Args, ValueEnum};
use tracing::info;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Default,
    Json,
}

#[derive(Args, Debug)]
pub struct Run {
    /// Message to send
    #[arg()]
    pub message: Vec<String>,

    /// The command to run, use message for args
    #[arg(long)]
    pub command: Option<String>,

    /// Continue the last session
    #[arg(short, long = "continue")]
    pub r#continue: bool,

    /// Session id to continue
    #[arg(short, long)]
    pub session: Option<String>,

    /// Share the session
    #[arg(long)]
    pub share: bool,

    /// Model to use in the format of provider/model
    #[arg(short, long)]
    pub model: Option<String>,

    /// Agent to use
    #[arg(long)]
    pub agent: Option<String>,

    /// Format: default (formatted) or json (raw JSON events)
    #[arg(long, value_enum, default_value_t = OutputFormat::Default)]
    pub format: OutputFormat,

    /// File(s) to attach to message
    #[arg(short, long)]
    pub file: Vec<PathBuf>,
}

impl Run {
    pub fn joined_message(&self) -> String {
        self.message.join(" ")
    }
}

pub async fn execute(cmd: &Run) -> anyhow::Result<()> {
    let message = cmd.joined_message();
    if message.is_empty() && cmd.command.is_none() {
        anyhow::bail!("message or command required");
    }

    info!(
        format = ?cmd.format,
        command = ?cmd.command,
        continue_session = cmd.r#continue,
        session = ?cmd.session,
        share = cmd.share,
        model = ?cmd.model,
        agent = ?cmd.agent,
        files = ?cmd.file,
        message,
        "run command"
    );

    let mut agent = Agent::new();
    agent.add_tool(EchoTool);

    let session = Session::new();
    info!(session_id = %session.id(), "Starting new session");

    if let Some((tool_name, args)) = cmd.message.split_first() {
        match agent.run_tool(tool_name, args).await {
            Ok(output) => {
                info!(%output, "Tool executed successfully");
                println!("{}", output);
            }
            Err(error) => {
                anyhow::bail!(error);
            }
        }
    }

    Ok(())
}
