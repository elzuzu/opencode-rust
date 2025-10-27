use clap::Parser;
use opencode_rust::cli::{Command, Opts, cmd};
use opencode_rust::util::config::{self, Info};
use opencode_rust::util::log::{self, LogConfig};
use std::path::Path;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let level = opts.log_level.unwrap_or_default().as_filter();
    log::init(LogConfig::new(level, opts.print_logs))?;

    let config_path = Path::new("opencode.json");
    let config: Info = if config_path.exists() {
        let config_str = tokio::fs::read_to_string(config_path).await?;
        config::parse_info(&config_str)?
    } else {
        Info::default()
    };

    info!(?config, "Loaded config");

    match opts.command {
        Command::Run(run_cmd) => {
            cmd::run::execute(&run_cmd, &config).await?;
        }
        Command::Generate => {
            info!("Generating OpenAPI spec");
        }
        Command::Auth(auth_cmd) => {
            cmd::auth::execute(&auth_cmd).await?;
        }
        Command::Agent(agent_cmd) => {
            cmd::agent::execute(&agent_cmd).await?;
        }
        Command::Upgrade(upgrade_cmd) => {
            cmd::upgrade::execute(&upgrade_cmd).await?;
        }
        Command::Models => {
            info!("Listing models");
        }
        Command::Serve(serve_cmd) => {
            cmd::serve::execute(&serve_cmd).await?;
        }
        Command::Stats(stats_cmd) => {
            cmd::stats::execute(&stats_cmd).await?;
        }
        Command::Export(export_cmd) => {
            cmd::export::execute(&export_cmd).await?;
        }
        Command::Attach(attach_cmd) => {
            cmd::attach::execute(&attach_cmd).await?;
        }
        Command::Acp(acp_cmd) => {
            cmd::acp::execute(&acp_cmd).await?;
        }
        Command::Mcp(mcp_cmd) => {
            cmd::mcp::execute(&mcp_cmd).await?;
        }
        Command::Tui(tui_cmd) => {
            cmd::tui::execute(&tui_cmd).await?;
        }
        Command::Debug(debug_cmd) => {
            cmd::debug::execute(&debug_cmd).await?;
        }
        Command::Github(github_cmd) => {
            cmd::github::execute(&github_cmd).await?;
        }
    }

    Ok(())
}
