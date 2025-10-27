use std::path::PathBuf;
use std::sync::Arc;

use crate::agent::registry::{AgentRegistry, parse_agents_source};
use crate::agent::spec::ModelHandle;
use crate::session::{
    AgentEvent, LocalModel, ProjectContext, SessionPrompts, SessionRequest, SessionResult,
    SessionRuntime, SubagentOutcome,
};
use crate::tool::core::Tool;
use crate::tool::echo::EchoTool;
use crate::util::config::Info;
use clap::{Args, ValueEnum};
use serde::Serialize;
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
    if matches!(cmd.format, OutputFormat::Json) {
        let report = RunReport::from(&result);
        let serialized = serde_json::to_string_pretty(&report)?;
        println!("{}", serialized);
        return Ok(());
    }

    let SessionResult { primary, subtasks } = result;
    println!("{}", primary.summary);
    for outcome in subtasks {
        println!("[{}] {}", outcome.agent, outcome.summary);
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

    if let Some(agent) = cmd.agent.as_deref() {
        if agent == "plan" {
            if !objective.is_empty() {
                objective.push_str("\n\n");
            }
            objective.push_str(SessionPrompts::plan_reminder().trim());
        }
        if agent == "build" && cmd.r#continue {
            if !objective.is_empty() {
                objective.push_str("\n\n");
            }
            objective.push_str(SessionPrompts::build_switch().trim());
        }
    }

    objective
}

#[derive(Debug, Serialize)]
struct RunReport {
    primary: SubtaskReport,
    subtasks: Vec<SubtaskReport>,
}

impl From<&SessionResult> for RunReport {
    fn from(result: &SessionResult) -> Self {
        let subtasks = result.subtasks.iter().map(SubtaskReport::from).collect();
        Self {
            primary: SubtaskReport::from(&result.primary),
            subtasks,
        }
    }
}

#[derive(Debug, Serialize)]
struct SubtaskReport {
    agent: String,
    objective: String,
    session_id: String,
    summary: String,
    model: String,
    raw_output: String,
}

impl From<&SubagentOutcome> for SubtaskReport {
    fn from(outcome: &SubagentOutcome) -> Self {
        Self {
            agent: outcome.agent.clone(),
            objective: outcome.objective.clone(),
            session_id: outcome.session_id.to_string(),
            summary: outcome.summary.clone(),
            model: outcome.model.id().to_string(),
            raw_output: outcome.raw_output.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::spec::ModelHandle;
    use uuid::Uuid;

    #[test]
    fn serializes_run_report_to_json() {
        let outcome = SubagentOutcome {
            agent: "primary".to_string(),
            objective: "Ship feature".to_string(),
            session_id: Uuid::nil(),
            summary: "done".to_string(),
            model: ModelHandle::new("test/model"),
            raw_output: "<prompt>".to_string(),
        };
        let result = SessionResult {
            primary: outcome.clone(),
            subtasks: vec![SubagentOutcome {
                agent: "builder".to_string(),
                objective: "Compile".to_string(),
                session_id: Uuid::new_v4(),
                summary: "compiled".to_string(),
                model: ModelHandle::new("child/model"),
                raw_output: "child".to_string(),
            }],
        };

        let report = RunReport::from(&result);
        let value = serde_json::to_value(&report).expect("json value");
        assert_eq!(value["primary"]["agent"], "primary");
        assert_eq!(value["primary"]["model"], "test/model");
        assert_eq!(value["subtasks"].as_array().map(|a| a.len()), Some(1));
    }

    #[test]
    fn plan_agent_appends_plan_reminder() {
        let cmd = Run {
            message: vec!["Plan the work".to_string()],
            command: None,
            r#continue: false,
            session: None,
            share: false,
            model: None,
            agent: Some("plan".to_string()),
            format: OutputFormat::Default,
            file: Vec::new(),
            agents_json: None,
        };

        let objective = build_objective(&cmd, &cmd.joined_message());
        assert!(objective.contains(SessionPrompts::plan_reminder().trim()));
    }

    #[test]
    fn build_agent_includes_switch_when_continuing() {
        let cmd = Run {
            message: vec!["Implement feature".to_string()],
            command: None,
            r#continue: true,
            session: Some("session-1".to_string()),
            share: false,
            model: None,
            agent: Some("build".to_string()),
            format: OutputFormat::Default,
            file: Vec::new(),
            agents_json: None,
        };

        let objective = build_objective(&cmd, &cmd.joined_message());
        assert!(objective.contains(SessionPrompts::build_switch().trim()));
    }
}
