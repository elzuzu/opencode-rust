use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct StatsCommand;

pub async fn execute(_cmd: &StatsCommand) -> anyhow::Result<()> {
    info!("stats command");
    Ok(())
}
