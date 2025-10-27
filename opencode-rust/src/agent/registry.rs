use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::agent::spec::{AgentMode, AgentRuntimeDefinition, AgentSpec};
use crate::util::config::{AgentConfig, Info};

const DEFAULT_AGENT: &str = "primary";

#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    specs: HashMap<String, Arc<AgentSpec>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            specs: HashMap::new(),
        };
        registry.ensure_primary();
        registry
    }

    pub fn from_info(info: &Info) -> Self {
        let mut registry = Self::new();
        if let Some(agents) = &info.agent {
            for (name, config) in agents {
                registry.apply_config(name, config);
            }
        }
        registry
    }

    pub fn ensure_primary(&mut self) -> Arc<AgentSpec> {
        if !self.specs.contains_key(DEFAULT_AGENT) {
            let spec = AgentSpec::new(DEFAULT_AGENT);
            self.specs.insert(DEFAULT_AGENT.to_string(), Arc::new(spec));
        }
        self.specs
            .get(DEFAULT_AGENT)
            .cloned()
            .expect("primary agent must exist")
    }

    pub fn apply_config(&mut self, name: &str, config: &AgentConfig) {
        let entry = self
            .specs
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AgentSpec::new(name)));
        let spec = Arc::make_mut(entry);
        spec.apply_config(config);
    }

    pub fn apply_runtime_definition(&mut self, name: &str, definition: &AgentRuntimeDefinition) {
        let entry = self
            .specs
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AgentSpec::new(name)));
        let spec = Arc::make_mut(entry);
        spec.apply_runtime(definition);
    }

    pub fn apply_runtime_map(&mut self, definitions: &HashMap<String, AgentRuntimeDefinition>) {
        for (name, definition) in definitions {
            self.apply_runtime_definition(name, definition);
        }
    }

    pub fn spec(&self, name: &str) -> Option<Arc<AgentSpec>> {
        self.specs.get(name).cloned()
    }

    pub fn require_spec(&self, name: &str) -> Result<Arc<AgentSpec>> {
        self.spec(name)
            .ok_or_else(|| anyhow!("unknown agent '{name}'"))
    }

    pub fn default_agent_name(&self) -> &str {
        DEFAULT_AGENT
    }

    pub fn all(&self) -> HashMap<String, Arc<AgentSpec>> {
        self.specs.clone()
    }

    pub fn agents_in_mode(&self, mode: AgentMode) -> Vec<Arc<AgentSpec>> {
        self.specs
            .values()
            .filter(|spec| match (mode.clone(), &spec.mode) {
                (AgentMode::All, _) => true,
                (AgentMode::Primary, AgentMode::All) => true,
                (AgentMode::Primary, AgentMode::Primary) => true,
                (AgentMode::Primary, AgentMode::Subagent) => false,
                (AgentMode::Subagent, AgentMode::All) => true,
                (AgentMode::Subagent, AgentMode::Subagent) => true,
                (AgentMode::Subagent, AgentMode::Primary) => false,
            })
            .cloned()
            .collect()
    }
}

pub fn parse_agents_json(content: &str) -> Result<HashMap<String, AgentRuntimeDefinition>> {
    let definitions: HashMap<String, AgentRuntimeDefinition> = serde_json::from_str(content)?;
    Ok(definitions)
}

pub fn parse_agents_file(
    path: &std::path::Path,
) -> Result<HashMap<String, AgentRuntimeDefinition>> {
    let content = std::fs::read_to_string(path)?;
    parse_agents_json(&content)
}

pub fn parse_agents_source(value: &str) -> Result<HashMap<String, AgentRuntimeDefinition>> {
    let path = std::path::Path::new(value);
    if path.exists() {
        parse_agents_file(path)
    } else {
        parse_agents_json(value)
    }
}
