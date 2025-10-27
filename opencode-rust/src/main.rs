use clap::Parser;
use opencode_rust::agent::core::Agent;
use opencode_rust::agent::session::Session;
use opencode_rust::cli::{Command, Opts};
use opencode_rust::tool::echo::EchoTool;
use opencode_rust::util::config::Info;
use opencode_rust::util::log;
use std::path::Path;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init("info");

    let opts = Opts::parse();

    let config_path = Path::new("opencode.json");
    let config: Info = if config_path.exists() {
        let config_str = tokio::fs::read_to_string(config_path).await?;
        serde_json::from_str(&config_str)?
    } else {
        serde_json::from_str("{}")?
    };

    info!(?config, "Loaded config");

    match opts.command {
        Command::Run(run_cmd) => {
            let mut agent = Agent::new();
            agent.add_tool(EchoTool);

            let session = Session::new();
            info!(session_id = %session.id(), "Starting new session");
            info!(message = ?run_cmd.message, "Running with message");

            if let Some((tool_name, args)) = run_cmd.message.split_first() {
                match agent.run_tool(tool_name, args).await {
                    Ok(output) => {
                        info!(%output, "Tool executed successfully");
                        println!("{}", output);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
        Command::Generate => {
            info!("Generating OpenAPI spec");
        }
    }

    Ok(())
}
