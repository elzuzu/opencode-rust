use clap::Args;

#[derive(Args, Debug)]
pub struct Run {
    /// Message to send
    #[clap(required = true)]
    pub message: Vec<String>,

    /// The command to run, use message for args
    #[clap(long)]
    pub command: Option<String>,

    /// Continue the last session
    #[clap(short, long("continue"))]
    pub r#continue: bool,

    /// Session id to continue
    #[clap(short, long)]
    pub session: Option<String>,

    /// Share the session
    #[clap(long)]
    pub share: bool,

    /// Model to use in the format of provider/model
    #[clap(short, long)]
    pub model: Option<String>,

    /// Agent to use
    #[clap(long)]
    pub agent: Option<String>,

    /// Format: default (formatted) or json (raw JSON events)
    #[clap(long, default_value = "default")]
    pub format: String,

    /// File(s) to attach to message
    #[clap(short, long)]
    pub file: Vec<String>,
}
