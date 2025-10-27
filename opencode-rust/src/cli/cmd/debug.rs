use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args, Debug)]
pub struct DebugCommand {
    #[command(subcommand)]
    pub action: DebugAction,
}

#[derive(Subcommand, Debug)]
pub enum DebugAction {
    /// Wait for a debugger to attach
    #[command(name = "wait")]
    Wait,
    /// Print important filesystem paths
    #[command(name = "paths")]
    Paths,
    /// Inspect configuration
    #[command(name = "config")]
    Config,
    /// Inspect project data
    #[command(name = "scrap")]
    Scrap,
    /// Snapshot helpers
    #[command(name = "snapshot")]
    Snapshot(DebugSnapshotArgs),
    /// Ripgrep helpers
    #[command(name = "rg")]
    Ripgrep(DebugRipgrepArgs),
    /// File helpers
    #[command(name = "file")]
    File(DebugFileArgs),
    /// LSP helpers
    #[command(name = "lsp")]
    Lsp(DebugLspArgs),
}

#[derive(Args, Debug)]
pub struct DebugSnapshotArgs {
    #[command(subcommand)]
    pub command: DebugSnapshotCommand,
}

#[derive(Subcommand, Debug)]
pub enum DebugSnapshotCommand {
    #[command(name = "track")]
    Track,
    #[command(name = "patch")]
    Patch(DebugHashArg),
    #[command(name = "diff")]
    Diff(DebugHashArg),
}

#[derive(Args, Debug)]
pub struct DebugHashArg {
    pub hash: String,
}

#[derive(Args, Debug)]
pub struct DebugRipgrepArgs {
    #[command(subcommand)]
    pub command: DebugRipgrepCommand,
}

#[derive(Subcommand, Debug)]
pub enum DebugRipgrepCommand {
    #[command(name = "tree")]
    Tree(DebugTreeArgs),
    #[command(name = "files")]
    Files(DebugFilesArgs),
    #[command(name = "search")]
    Search(DebugSearchArgs),
}

#[derive(Args, Debug)]
pub struct DebugTreeArgs {
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Args, Debug)]
pub struct DebugFilesArgs {
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub glob: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Args, Debug)]
pub struct DebugSearchArgs {
    pub pattern: String,
    #[arg(long, value_name = "GLOB")]
    pub glob: Vec<String>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Args, Debug)]
pub struct DebugFileArgs {
    #[command(subcommand)]
    pub command: DebugFileCommand,
}

#[derive(Subcommand, Debug)]
pub enum DebugFileCommand {
    #[command(name = "search")]
    Search(DebugFileSearchArgs),
    #[command(name = "read")]
    Read(DebugFilePathArg),
    #[command(name = "status")]
    Status,
    #[command(name = "list")]
    List(DebugFilePathArg),
}

#[derive(Args, Debug)]
pub struct DebugFileSearchArgs {
    pub query: String,
}

#[derive(Args, Debug)]
pub struct DebugFilePathArg {
    pub path: String,
}

#[derive(Args, Debug)]
pub struct DebugLspArgs {
    #[command(subcommand)]
    pub command: DebugLspCommand,
}

#[derive(Subcommand, Debug)]
pub enum DebugLspCommand {
    #[command(name = "diagnostics")]
    Diagnostics(DebugFilePathArg),
    #[command(name = "symbols")]
    Symbols(DebugFileSearchArgs),
    #[command(name = "document-symbols")]
    DocumentSymbols(DebugUriArg),
}

#[derive(Args, Debug)]
pub struct DebugUriArg {
    pub uri: String,
}

pub async fn execute(cmd: &DebugCommand) -> anyhow::Result<()> {
    match &cmd.action {
        DebugAction::Wait => {
            info!("debug wait");
        }
        DebugAction::Paths => {
            info!("debug paths");
        }
        DebugAction::Config => {
            info!("debug config");
        }
        DebugAction::Scrap => {
            info!("debug scrap");
        }
        DebugAction::Snapshot(snapshot) => match &snapshot.command {
            DebugSnapshotCommand::Track => {
                info!("debug snapshot track");
            }
            DebugSnapshotCommand::Patch(hash) => {
                info!(hash = %hash.hash, "debug snapshot patch");
            }
            DebugSnapshotCommand::Diff(hash) => {
                info!(hash = %hash.hash, "debug snapshot diff");
            }
        },
        DebugAction::Ripgrep(rg) => match &rg.command {
            DebugRipgrepCommand::Tree(args) => {
                info!(limit = ?args.limit, "debug rg tree");
            }
            DebugRipgrepCommand::Files(args) => {
                info!(query = ?args.query, glob = ?args.glob, limit = ?args.limit, "debug rg files");
            }
            DebugRipgrepCommand::Search(args) => {
                info!(pattern = %args.pattern, glob = ?args.glob, limit = ?args.limit, "debug rg search");
            }
        },
        DebugAction::File(file) => match &file.command {
            DebugFileCommand::Search(args) => {
                info!(query = %args.query, "debug file search");
            }
            DebugFileCommand::Read(path) => {
                info!(path = %path.path, "debug file read");
            }
            DebugFileCommand::Status => {
                info!("debug file status");
            }
            DebugFileCommand::List(path) => {
                info!(path = %path.path, "debug file list");
            }
        },
        DebugAction::Lsp(lsp) => match &lsp.command {
            DebugLspCommand::Diagnostics(path) => {
                info!(path = %path.path, "debug lsp diagnostics");
            }
            DebugLspCommand::Symbols(query) => {
                info!(query = %query.query, "debug lsp symbols");
            }
            DebugLspCommand::DocumentSymbols(uri) => {
                info!(uri = %uri.uri, "debug lsp document symbols");
            }
        },
    }
    Ok(())
}
