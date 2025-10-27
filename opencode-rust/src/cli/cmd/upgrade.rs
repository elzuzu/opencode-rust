use clap::{Args, ValueEnum};
use tracing::info;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum UpgradeMethod {
    Curl,
    Npm,
    Pnpm,
    Bun,
    Brew,
}

#[derive(Args, Debug)]
pub struct UpgradeCommand {
    /// Version to upgrade to
    pub target: Option<String>,
    /// Installation method to use
    #[arg(short, long, value_enum)]
    pub method: Option<UpgradeMethod>,
}

pub async fn execute(cmd: &UpgradeCommand) -> anyhow::Result<()> {
    info!(target = ?cmd.target, method = ?cmd.method, "upgrade command");
    Ok(())
}
