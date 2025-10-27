use clap::Parser;
use opencode_rust::cli::{Command, Opts};
use opencode_rust::config::Info;
use opencode_rust::watcher;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let config_path = Path::new("opencode.json");
    let config: Info = if config_path.exists() {
        let config_str = tokio::fs::read_to_string(config_path).await?;
        serde_json::from_str(&config_str)?
    } else {
        serde_json::from_str("{}")?
    };

    println!("Loaded config: {:?}", config);

    match opts.command {
        Command::Run { message, .. } => {
            println!("Running with message: {:?}", message);
            let watch_task = tokio::spawn(async move {
                let _ = watcher::watch(Path::new(".")).await;
            });
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            watch_task.abort();
        }
        Command::Generate => {
            println!("Generating OpenAPI spec");
        }
    }

    Ok(())
}
