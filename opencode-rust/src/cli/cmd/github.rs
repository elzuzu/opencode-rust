use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args, Debug)]
pub struct GithubCommand {
    #[command(subcommand)]
    pub action: GithubAction,
}

#[derive(Subcommand, Debug)]
pub enum GithubAction {
    /// Install the GitHub agent
    #[command(name = "install")]
    Install,
    /// Run the GitHub agent locally
    #[command(name = "run")]
    Run(GithubRunArgs),
}

#[derive(Args, Debug)]
pub struct GithubRunArgs {
    /// GitHub mock event payload
    #[arg(long)]
    pub event: Option<String>,
    /// GitHub personal access token
    #[arg(long)]
    pub token: Option<String>,
}

pub async fn execute(cmd: &GithubCommand) -> anyhow::Result<()> {
    match &cmd.action {
        GithubAction::Install => {
            info!("github install");
        }
        GithubAction::Run(args) => {
            info!(event = ?args.event, token = args.token.is_some(), "github run");
        }
    }
    Ok(())
}
