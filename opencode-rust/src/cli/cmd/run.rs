use std::path::PathBuf;
use std::sync::Arc;

use crate::agent::registry::{AgentRegistry, parse_agents_source};
use crate::agent::spec::ModelHandle;
use crate::session::{AgentEvent, LocalModel, ProjectContext, SessionRequest, SessionRuntime};
use crate::tool::core::Tool;
use crate::tool::echo::EchoTool;
use crate::util::config::Info;
use clap::{Args, ValueEnum};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Default,
    Json,
}

#[derive(Args, Debug)]
pub struct Run {
    /// Message to send
    #[arg()]
    pub message: Vec<String>,

    /// The command to run, use message for args
    #[arg(long)]
    pub command: Option<String>,

    /// Continue the last session
    #[arg(short, long = "continue")]
    pub r#continue: bool,

    /// Session id to continue
    #[arg(short, long)]
    pub session: Option<String>,

    /// Share the session
    #[arg(long)]
    pub share: bool,

    /// Model to use in the format of provider/model
    #[arg(short, long)]
    pub model: Option<String>,

    /// Agent to use
    #[arg(long)]
    pub agent: Option<String>,

    /// Format: default (formatted) or json (raw JSON events)
    #[arg(long, value_enum, default_value_t = OutputFormat::Default)]
    pub format: OutputFormat,

    /// File(s) to attach to message
    #[arg(short, long)]
    pub file: Vec<PathBuf>,

    /// Path or JSON string describing agents to merge at runtime
    #[arg(long = "agents-json")]
    pub agents_json: Option<String>,
}

impl Run {
    pub fn joined_message(&self) -> String {
        self.message.join(" ")
    }
}

pub async fn execute(cmd: &Run, config: &Info) -> anyhow::Result<()> {
    let message = cmd.joined_message();
    if message.is_empty() && cmd.command.is_none() {
        anyhow::bail!("message or command required");
    }

    info!(
        format = ?cmd.format,
        command = ?cmd.command,
        continue_session = cmd.r#continue,
        session = ?cmd.session,
        share = cmd.share,
        model = ?cmd.model,
        agent = ?cmd.agent,
        files = ?cmd.file,
        message,
        "run command"
    );

    let tools: Vec<Arc<dyn Tool>> = vec![Arc::new(EchoTool)];

    if let Some((tool_name, args)) = cmd.message.split_first() {
        if let Some(tool) = tools.iter().find(|tool| tool.name() == tool_name) {
            info!(tool = tool.name(), "executing tool invocation");
            let output = tool.execute(args).await?;
            println!("{}", output);
            return Ok(());
        }
    }

    let mut registry = AgentRegistry::from_info(config);
    if let Some(source) = &cmd.agents_json {
        let overrides = parse_agents_source(source)?;
        registry.apply_runtime_map(&overrides);
    }
    registry.ensure_primary();

    let default_model = cmd
        .model
        .clone()
        .or_else(|| config.model.clone())
        .unwrap_or_else(|| "openai/gpt-4o".to_string());

    let project_root = std::env::current_dir()?;
    let context = Arc::new(ProjectContext::gather(project_root, config)?);
    let (event_tx, mut event_rx) = mpsc::channel(32);
    let runtime = SessionRuntime::new(
        context,
        Arc::new(registry),
        Arc::new(LocalModel::default()),
        tools.clone(),
        event_tx.clone(),
        ModelHandle::new(default_model),
    );

    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                AgentEvent::Started {
                    session_id,
                    agent,
                    objective,
                } => {
                    info!(%session_id, %agent, %objective, "subagent started");
                }
                AgentEvent::Completed {
                    session_id,
                    agent,
                    summary,
                } => {
                    info!(%session_id, %agent, %summary, "subagent completed");
                }
            }
        }
    });

    let objective = build_objective(cmd, &message);

    let request = SessionRequest {
        agent: cmd.agent.clone(),
        objective,
        subtasks: Vec::new(),
    };

    let result = runtime.execute(request).await?;
    println!("{}", result.primary.summary);
    if !result.subtasks.is_empty() {
        for outcome in result.subtasks {
            println!("[{}] {}", outcome.agent, outcome.summary);
        }
    }

    Ok(())
}

fn build_objective(cmd: &Run, message: &str) -> String {
    let mut objective = String::new();
    if let Some(command) = &cmd.command {
        objective.push_str(command);
        if !message.is_empty() {
            objective.push_str(" -- ");
            objective.push_str(message);
        }
    } else {
        objective.push_str(message);
    }

    if !cmd.file.is_empty() {
        let attachments: Vec<String> = cmd
            .file
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        if !attachments.is_empty() {
            objective.push_str(" [attachments: ");
            objective.push_str(&attachments.join(", "));
            objective.push(']');
        }
    }

    objective
}
