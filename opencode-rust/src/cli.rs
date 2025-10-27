use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "opencode-rust", version, about = "The AI coding agent built for the terminal.")]
pub struct Opts {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run opencode with a message
    Run {
        /// Message to send
        message: Vec<String>,

        /// The command to run, use message for args
        #[clap(long)]
        command: Option<String>,

        /// Continue the last session
        #[clap(short, long("continue"))]
        r#continue: bool,

        /// Session id to continue
        #[clap(short, long)]
        session: Option<String>,

        /// Share the session
        #[clap(long)]
        share: bool,

        /// Model to use in the format of provider/model
        #[clap(short, long)]
        model: Option<String>,

        /// Agent to use
        #[clap(long)]
        agent: Option<String>,

        /// Format: default (formatted) or json (raw JSON events)
        #[clap(long, default_value = "default")]
        format: String,

        /// File(s) to attach to message
        #[clap(short, long)]
        file: Vec<String>,
    },
    /// Generate OpenAPI spec
    Generate,
}
