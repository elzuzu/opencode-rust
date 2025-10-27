use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use opencode_rust::agent::registry::{AgentRegistry, parse_agents_json};
use opencode_rust::agent::spec::{AgentSpec, ModelHandle, resolve_model, resolve_tools};
use opencode_rust::session::{ProjectContext, PromptBuilder};
use opencode_rust::tool::core::Tool;
use opencode_rust::tool::echo::EchoTool;
use opencode_rust::util::config::{AgentConfig, AgentMode, Info};
use serde_json::Value;
use tempfile::tempdir;

struct FakeTool;

#[async_trait]
impl Tool for FakeTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "shell executor"
    }

    async fn execute(&self, _args: &[String]) -> opencode_rust::util::error::Result<String> {
        Ok(String::new())
    }
}

#[test]
fn merges_runtime_overrides() {
    let mut agent_config = AgentConfig {
        model: Some("config-model".to_string()),
        temperature: None,
        top_p: None,
        prompt: Some("Base prompt".to_string()),
        tools: Some(HashMap::from([
            ("echo".to_string(), true),
            ("shell".to_string(), false),
        ])),
        disable: None,
        description: Some("Primary agent".to_string()),
        mode: Some(AgentMode::Primary),
        permission: None,
        extra: HashMap::new(),
    };
    agent_config
        .extra
        .insert("reportFormat".into(), Value::String("Summaries".into()));

    let mut info = Info::default();
    info.agent = Some(HashMap::from([("primary".to_string(), agent_config)]));

    let mut registry = AgentRegistry::from_info(&info);
    let overrides = parse_agents_json(
        r#"{
        "primary": {
            "model": "cli-model",
            "promptSections": ["Runtime prompt"],
            "budgets": {"maxTokens": 400},
            "tools": {"allow": ["echo"]}
        },
        "builder": {
            "model": "builder-model",
            "prompt": "Build artifacts",
            "mode": "subagent"
        }
    }"#,
    )
    .expect("valid json");
    registry.apply_runtime_map(&overrides);
    registry.ensure_primary();

    let primary = registry.require_spec("primary").expect("primary spec");
    assert_eq!(primary.model.as_deref(), Some("cli-model"));
    assert_eq!(primary.prompt_sections, vec!["Runtime prompt".to_string()]);
    assert_eq!(primary.budgets.max_tokens, Some(400));

    let parent_model = ModelHandle::new("fallback/model");
    let resolved_model = resolve_model(&primary, &parent_model);
    assert_eq!(resolved_model.id(), "cli-model");

    let toolset: Vec<Arc<dyn Tool>> = vec![Arc::new(EchoTool), Arc::new(FakeTool)];
    let resolved_tools = resolve_tools(&primary, &toolset);
    assert_eq!(resolved_tools.len(), 1);
    assert_eq!(resolved_tools[0].name(), "echo");

    let builder = registry.require_spec("builder").expect("builder spec");
    let child_model = resolve_model(&builder, &resolved_model);
    assert_eq!(child_model.id(), "builder-model");
}

#[test]
fn prompt_builder_collects_rules() -> Result<()> {
    let temp = tempdir()?;
    std::fs::write(temp.path().join("CLAUDE.md"), "Global guidance")?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(temp.path().join("docs/AGENTS.md"), "Docs agent")?;
    std::fs::create_dir_all(temp.path().join("migration"))?;
    std::fs::write(temp.path().join("migration/task.md"), "Migration steps")?;

    let mut info = Info::default();
    info.instructions = Some(vec!["Stay focused".to_string()]);

    let context = ProjectContext::gather(temp.path(), &info)?;

    let mut spec = AgentSpec::new("primary");
    spec.prompt_sections = vec!["Handle user requests".to_string()];
    spec.description = Some("Primary orchestrator".to_string());
    spec.budgets.max_tokens = Some(200);

    let builder = PromptBuilder::new(&spec, &context, "Implement feature");
    let prompt = builder.build();

    assert!(prompt.contains("<ROLE>"));
    assert!(prompt.contains("Handle user requests"));
    assert!(prompt.contains("Stay focused"));
    assert!(prompt.contains("Global guidance"));
    assert!(prompt.contains("Migration steps"));
    assert!(prompt.contains("Limit responses to 200 tokens."));
    assert!(prompt.contains("<REPORT_FORMAT>"));

    Ok(())
}

#[test]
fn inherits_parent_model_when_unspecified() {
    let spec = AgentSpec::new("child");
    let parent = ModelHandle::new("parent/model");
    let resolved = resolve_model(&spec, &parent);
    assert_eq!(resolved.id(), "parent/model");
}
