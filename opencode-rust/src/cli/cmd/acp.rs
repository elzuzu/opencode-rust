use std::path::PathBuf;

use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct AcpCommand {
    /// Working directory
    #[arg(long = "cwd", value_name = "PATH", default_value = ".")]
    pub cwd: PathBuf,
}

pub async fn execute(cmd: &AcpCommand) -> anyhow::Result<()> {
    info!(cwd = ?cmd.cwd, "acp command");
    Ok(())
}
