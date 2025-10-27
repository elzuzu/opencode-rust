use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args, Debug)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub action: AuthAction,
}

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// List providers
    #[command(name = "list", alias = "ls")]
    List,
    /// Log in to a provider
    #[command(name = "login")]
    Login(AuthLoginCommand),
    /// Log out of a provider
    #[command(name = "logout")]
    Logout,
}

#[derive(Args, Debug)]
pub struct AuthLoginCommand {
    /// Optional authentication provider URL
    pub url: Option<String>,
}

pub async fn execute(cmd: &AuthCommand) -> anyhow::Result<()> {
    match &cmd.action {
        AuthAction::List => {
            info!("auth list");
        }
        AuthAction::Login(login) => {
            info!(url = ?login.url, "auth login");
        }
        AuthAction::Logout => {
            info!("auth logout");
        }
    }
    Ok(())
}
