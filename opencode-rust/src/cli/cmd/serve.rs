use clap::Args;
use tracing::info;

#[derive(Args, Debug)]
pub struct ServeCommand {
    /// Port to listen on
    #[arg(short, long, default_value_t = 0)]
    pub port: u16,
    /// Hostname to bind
    #[arg(short, long, default_value = "127.0.0.1")]
    pub hostname: String,
}

pub async fn execute(cmd: &ServeCommand) -> anyhow::Result<()> {
    info!(port = cmd.port, hostname = %cmd.hostname, "serve command");
    Ok(())
}
