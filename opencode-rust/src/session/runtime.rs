use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time;
use tracing::{debug, info};
use uuid::Uuid;

use crate::agent::registry::AgentRegistry;
use crate::agent::session::Session;
use crate::agent::spec::{AgentBudgets, AgentSpec, ModelHandle, resolve_model, resolve_tools};
use crate::session::prompt_builder::{ProjectContext, PromptBuilder};
use crate::tool::core::Tool;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Started {
        session_id: Uuid,
        agent: String,
        objective: String,
    },
    Completed {
        session_id: Uuid,
        agent: String,
        summary: String,
    },
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub agent: String,
    pub model: ModelHandle,
    pub prompt: String,
    pub objective: String,
    pub budgets: AgentBudgets,
    pub tool_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub summary: String,
    pub raw_output: String,
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}

#[derive(Debug, Clone, Default)]
pub struct LocalModel;

#[async_trait]
impl LanguageModel for LocalModel {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut summary = format!(
            "{} completed objective: {}",
            request.agent,
            request.objective.trim()
        );
        if let Some(max_tokens) = request.budgets.max_tokens {
            summary.push_str(&format!(" (limit {max_tokens} tokens)"));
        }
        if let Some(timeout) = request.budgets.tool_timeout {
            summary.push_str(&format!(" (tool timeout {}s)", timeout.as_secs()));
        }
        Ok(CompletionResponse {
            summary,
            raw_output: request.prompt,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SubagentInvocation {
    pub agent: String,
    pub objective: String,
}

impl SubagentInvocation {
    pub fn new(agent: impl Into<String>, objective: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            objective: objective.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubagentOutcome {
    pub agent: String,
    pub objective: String,
    pub session_id: Uuid,
    pub summary: String,
    pub model: ModelHandle,
    pub raw_output: String,
}

#[derive(Debug, Clone)]
pub struct SessionRequest {
    pub agent: Option<String>,
    pub objective: String,
    pub subtasks: Vec<SubagentInvocation>,
}

impl SessionRequest {
    pub fn new(objective: impl Into<String>) -> Self {
        Self {
            agent: None,
            objective: objective.into(),
            subtasks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionResult {
    pub primary: SubagentOutcome,
    pub subtasks: Vec<SubagentOutcome>,
}

struct SpawnArtifacts {
    outcome: SubagentOutcome,
    tools: Vec<Arc<dyn Tool>>,
}

pub struct SessionRuntime {
    context: Arc<ProjectContext>,
    registry: Arc<AgentRegistry>,
    model: Arc<dyn LanguageModel>,
    tools: Arc<Vec<Arc<dyn Tool>>>,
    event_tx: mpsc::Sender<AgentEvent>,
    default_model: ModelHandle,
}

impl SessionRuntime {
    pub fn new(
        context: Arc<ProjectContext>,
        registry: Arc<AgentRegistry>,
        model: Arc<dyn LanguageModel>,
        tools: Vec<Arc<dyn Tool>>,
        event_tx: mpsc::Sender<AgentEvent>,
        default_model: ModelHandle,
    ) -> Self {
        Self {
            context,
            registry,
            model,
            tools: Arc::new(tools),
            event_tx,
            default_model,
        }
    }

    pub async fn execute(&self, request: SessionRequest) -> Result<SessionResult> {
        let agent_name = request
            .agent
            .as_deref()
            .unwrap_or(self.registry.default_agent_name());
        let spec = self.registry.require_spec(agent_name)?;
        let spawn = self
            .spawn_agent(
                spec,
                request.objective.clone(),
                self.default_model.clone(),
                (*self.tools).clone(),
            )
            .await?;
        let parent_model = spawn.outcome.model.clone();
        let parent_tools = spawn.tools.clone();
        let mut subtasks = Vec::new();

        if !request.subtasks.is_empty() {
            let mut set = JoinSet::new();
            for invocation in request.subtasks {
                let spec = self.registry.require_spec(&invocation.agent)?;
                let runtime = self.clone();
                let objective = invocation.objective.clone();
                let model = parent_model.clone();
                let tools = parent_tools.clone();
                set.spawn(async move {
                    runtime
                        .spawn_agent(spec, objective, model, tools)
                        .await
                        .map(|artifacts| artifacts.outcome)
                });
            }

            while let Some(result) = set.join_next().await {
                subtasks.push(result??);
            }
        }

        Ok(SessionResult {
            primary: spawn.outcome,
            subtasks,
        })
    }

    async fn spawn_agent(
        &self,
        spec: Arc<AgentSpec>,
        objective: String,
        parent_model: ModelHandle,
        parent_tools: Vec<Arc<dyn Tool>>,
    ) -> Result<SpawnArtifacts> {
        let session = Session::new();
        let session_id = session.id();
        let model = resolve_model(&spec, &parent_model);
        let tools = resolve_tools(&spec, &parent_tools);
        let tool_names = tools.iter().map(|tool| tool.name().to_string()).collect();
        let builder = PromptBuilder::new(&spec, &self.context, &objective);
        let prompt = builder.build();
        let budgets = spec.budgets.clone();
        let timeout = budgets.wall_clock_or(Duration::from_secs(60));

        debug!(
            agent = %spec.name,
            ?model,
            tool_count = tools.len(),
            objective = objective,
            "spawning agent"
        );

        let _ = self
            .event_tx
            .send(AgentEvent::Started {
                session_id,
                agent: spec.name.clone(),
                objective: objective.clone(),
            })
            .await;

        let request = CompletionRequest {
            agent: spec.name.clone(),
            model: model.clone(),
            prompt,
            objective: objective.clone(),
            budgets: budgets.clone(),
            tool_names,
        };

        let response = time::timeout(timeout, self.model.complete(request)).await??;

        let outcome = SubagentOutcome {
            agent: spec.name.clone(),
            objective,
            session_id,
            summary: response.summary.clone(),
            model: model.clone(),
            raw_output: response.raw_output,
        };

        let _ = self
            .event_tx
            .send(AgentEvent::Completed {
                session_id,
                agent: spec.name.clone(),
                summary: outcome.summary.clone(),
            })
            .await;

        info!(agent = %spec.name, session_id = %session_id, "agent completed");

        Ok(SpawnArtifacts { outcome, tools })
    }
}

impl Clone for SessionRuntime {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            registry: self.registry.clone(),
            model: self.model.clone(),
            tools: self.tools.clone(),
            event_tx: self.event_tx.clone(),
            default_model: self.default_model.clone(),
        }
    }
}
