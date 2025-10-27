use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args, Debug)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub action: AgentAction,
}

#[derive(Subcommand, Debug)]
pub enum AgentAction {
    /// Create a new agent configuration
    #[command(name = "create")]
    Create,
}

pub async fn execute(cmd: &AgentCommand) -> anyhow::Result<()> {
    match &cmd.action {
        AgentAction::Create => {
            info!("agent create");
        }
    }
    Ok(())
}
