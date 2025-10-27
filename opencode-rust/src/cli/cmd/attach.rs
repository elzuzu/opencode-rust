use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct AttachCommand {
    /// Server to connect to
    #[arg(value_name = "URL")]
    pub server: String,
    /// Session id to continue
    #[arg(short, long)]
    pub session: Option<String>,
}

pub async fn execute(cmd: &AttachCommand) -> anyhow::Result<()> {
    info!(server = %cmd.server, session = ?cmd.session, "attach command");
    Ok(())
}
