use clap::{Parser, Subcommand};
mod cmd;

#[derive(Parser, Debug)]
#[clap(name = "opencode-rust", version, about = "The AI coding agent built for the terminal.")]
pub struct Opts {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run opencode with a message
    Run(cmd::run::Run),
    /// Generate OpenAPI spec
    Generate,
}
