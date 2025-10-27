use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args, Debug)]
pub struct McpCommand {
    #[command(subcommand)]
    pub action: McpAction,
}

#[derive(Subcommand, Debug)]
pub enum McpAction {
    /// Add an MCP server
    #[command(name = "add")]
    Add,
}

pub async fn execute(cmd: &McpCommand) -> anyhow::Result<()> {
    match &cmd.action {
        McpAction::Add => {
            info!("mcp add");
        }
    }
    Ok(())
}
