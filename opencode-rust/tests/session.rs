use std::sync::Arc;

use anyhow::Result;
use opencode_rust::agent::registry::{AgentRegistry, parse_agents_json};
use opencode_rust::agent::spec::ModelHandle;
use opencode_rust::session::{
    AgentEvent, LocalModel, ProjectContext, SessionRequest, SessionRuntime, SubagentInvocation,
};
use opencode_rust::tool::core::Tool;
use opencode_rust::tool::echo::EchoTool;
use opencode_rust::util::config::Info;
use tempfile::tempdir;
use tokio::sync::{Mutex, mpsc};

#[tokio::test]
async fn executes_subagents_and_emits_events() -> Result<()> {
    let temp = tempdir()?;
    let info = Info::default();
    let context = Arc::new(ProjectContext::gather(temp.path(), &info)?);

    let mut registry = AgentRegistry::new();
    let overrides = parse_agents_json(
        r#"{
        "primary": {
            "promptSections": ["Coordinate work"],
            "budgets": {"maxTokens": 128}
        },
        "builder": {
            "prompt": "Synthesize build output",
            "mode": "subagent"
        }
    }"#,
    )?;
    registry.apply_runtime_map(&overrides);
    registry.ensure_primary();
    let registry = Arc::new(registry);

    let (event_tx, event_rx) = mpsc::channel(16);
    let tools: Vec<Arc<dyn Tool>> = vec![Arc::new(EchoTool)];
    let runtime = SessionRuntime::new(
        context,
        registry,
        Arc::new(LocalModel::default()),
        tools,
        event_tx,
        ModelHandle::new("baseline/model"),
    );

    let events = Arc::new(Mutex::new(Vec::new()));
    let mut rx = event_rx;
    let event_sink = events.clone();
    let collector = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            event_sink.lock().await.push(event);
        }
    });

    let request = SessionRequest {
        agent: Some("primary".to_string()),
        objective: "Coordinate build".to_string(),
        subtasks: vec![
            SubagentInvocation::new("builder", "Compile artifacts"),
            SubagentInvocation::new("builder", "Write report"),
        ],
    };

    let result = runtime.execute(request).await?;
    drop(runtime);
    collector.await.unwrap();

    assert_eq!(result.subtasks.len(), 2);
    let captured = events.lock().await;
    assert!(
        captured
            .iter()
            .any(|event| matches!(event, AgentEvent::Started { agent, .. } if agent == "builder"))
    );
    assert!(
        captured.iter().any(
            |event| matches!(event, AgentEvent::Completed { agent, .. } if agent == "primary")
        )
    );

    Ok(())
}
