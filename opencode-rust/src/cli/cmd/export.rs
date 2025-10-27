use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct ExportCommand {
    /// Session id to export
    pub session_id: Option<String>,
}

pub async fn execute(cmd: &ExportCommand) -> anyhow::Result<()> {
    info!(session = ?cmd.session_id, "export command");
    Ok(())
}
