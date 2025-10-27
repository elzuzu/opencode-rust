use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::tool::core::Tool;
use crate::util::config::{AgentConfig, AgentMode as ConfigAgentMode};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Primary,
    Subagent,
    All,
}

impl Default for AgentMode {
    fn default() -> Self {
        AgentMode::All
    }
}

impl From<ConfigAgentMode> for AgentMode {
    fn from(value: ConfigAgentMode) -> Self {
        match value {
            ConfigAgentMode::Primary => AgentMode::Primary,
            ConfigAgentMode::Subagent => AgentMode::Subagent,
            ConfigAgentMode::All => AgentMode::All,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolRules {
    allow: Option<Vec<String>>,
    deny: Vec<String>,
}

impl ToolRules {
    pub fn inherit() -> Self {
        Self {
            allow: None,
            deny: Vec::new(),
        }
    }

    pub fn allow_list(&self) -> Option<&[String]> {
        self.allow.as_deref()
    }

    pub fn deny_list(&self) -> &[String] {
        &self.deny
    }

    pub fn update_from_map(&mut self, map: &HashMap<String, bool>) {
        let mut allow = Vec::new();
        let mut deny = Vec::new();
        for (name, enabled) in map {
            if *enabled {
                allow.push(name.clone());
            } else {
                deny.push(name.clone());
            }
        }
        if !allow.is_empty() {
            allow.sort();
            allow.dedup();
            self.allow = Some(allow);
        }
        if !deny.is_empty() {
            deny.sort();
            deny.dedup();
            self.deny = deny;
        }
    }

    pub fn update_from_definition(&mut self, definition: &AgentToolDefinition) {
        if !definition.allow.is_empty() {
            let mut allow = definition.allow.clone();
            allow.sort();
            allow.dedup();
            self.allow = Some(allow);
        }
        if !definition.deny.is_empty() {
            let mut deny = definition.deny.clone();
            deny.sort();
            deny.dedup();
            self.deny = deny;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentBudgets {
    pub max_tokens: Option<u32>,
    pub tool_timeout: Option<Duration>,
    pub wall_clock: Option<Duration>,
}

impl AgentBudgets {
    pub fn merge(&mut self, other: &AgentBudgets) {
        if other.max_tokens.is_some() {
            self.max_tokens = other.max_tokens;
        }
        if other.tool_timeout.is_some() {
            self.tool_timeout = other.tool_timeout;
        }
        if other.wall_clock.is_some() {
            self.wall_clock = other.wall_clock;
        }
    }

    pub fn merge_definition(&mut self, definition: &AgentBudgetsDefinition) {
        if let Some(tokens) = definition.max_tokens {
            self.max_tokens = Some(tokens);
        }
        if let Some(ms) = definition.tool_timeout_ms {
            self.tool_timeout = Some(Duration::from_millis(ms));
        }
        if let Some(ms) = definition.wall_clock_ms {
            self.wall_clock = Some(Duration::from_millis(ms));
        }
    }

    pub fn describe_constraints(&self) -> Vec<String> {
        let mut constraints = Vec::new();
        if let Some(tokens) = self.max_tokens {
            constraints.push(format!("Limit responses to {tokens} tokens."));
        }
        if let Some(timeout) = self.tool_timeout {
            let seconds = timeout.as_secs();
            constraints.push(format!(
                "Each tool call must complete within {seconds} seconds."
            ));
        }
        if let Some(limit) = self.wall_clock {
            let seconds = limit.as_secs();
            constraints.push(format!(
                "Finish the task within {seconds} seconds of wall-clock time."
            ));
        }
        constraints
    }

    pub fn wall_clock_or(&self, fallback: Duration) -> Duration {
        self.wall_clock.unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSpec {
    pub name: String,
    pub description: Option<String>,
    pub mode: AgentMode,
    pub model: Option<String>,
    pub prompt_sections: Vec<String>,
    pub tool_rules: ToolRules,
    pub budgets: AgentBudgets,
    pub report_format: Option<String>,
}

impl AgentSpec {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            description: None,
            mode: AgentMode::All,
            model: None,
            prompt_sections: vec![format!(
                "You are the {name} agent. Respond with actionable assistance."
            )],
            tool_rules: ToolRules::inherit(),
            budgets: AgentBudgets::default(),
            report_format: None,
        }
    }

    pub fn apply_config(&mut self, config: &AgentConfig) {
        if let Some(model) = &config.model {
            self.model = Some(model.clone());
        }
        if let Some(description) = &config.description {
            self.description = Some(description.clone());
        }
        if let Some(prompt) = &config.prompt {
            self.prompt_sections = vec![prompt.clone()];
        }
        if let Some(tools) = &config.tools {
            self.tool_rules.update_from_map(tools);
        }
        if let Some(mode) = &config.mode {
            self.mode = AgentMode::from(mode.clone());
        }
        self.extract_extra(&config.extra);
    }

    pub fn apply_runtime(&mut self, definition: &AgentRuntimeDefinition) {
        if let Some(model) = &definition.model {
            self.model = Some(model.clone());
        }
        if let Some(description) = &definition.description {
            self.description = Some(description.clone());
        }
        if let Some(mode) = &definition.mode {
            self.mode = mode.clone();
        }
        if let Some(report_format) = &definition.report_format {
            self.report_format = Some(report_format.clone());
        }
        if let Some(prompt) = &definition.prompt {
            if !prompt.trim().is_empty() {
                self.prompt_sections = vec![prompt.clone()];
            }
        }
        if !definition.prompt_sections.is_empty() {
            self.prompt_sections = definition.prompt_sections.clone();
        }
        if let Some(tool_definition) = &definition.tools {
            self.tool_rules.update_from_definition(tool_definition);
        }
        if let Some(budget_definition) = &definition.budgets {
            self.budgets.merge_definition(budget_definition);
        }
    }

    fn extract_extra(&mut self, extra: &HashMap<String, JsonValue>) {
        if let Some(value) = extra.get("promptSections") {
            if let Some(sections) = Self::parse_string_array(value) {
                if !sections.is_empty() {
                    self.prompt_sections = sections;
                }
            }
        }
        if let Some(value) = extra.get("budgets") {
            if let Some(definition) = AgentBudgetsDefinition::from_json(value) {
                self.budgets.merge_definition(&definition);
            }
        }
        if let Some(value) = extra.get("reportFormat") {
            if let Some(report_format) = value.as_str() {
                if !report_format.trim().is_empty() {
                    self.report_format = Some(report_format.to_string());
                }
            }
        }
    }

    fn parse_string_array(value: &JsonValue) -> Option<Vec<String>> {
        let array = value.as_array()?;
        let mut result = Vec::new();
        for item in array {
            if let Some(text) = item.as_str() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    result.push(trimmed.to_string());
                }
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentRuntimeDefinition {
    pub model: Option<String>,
    #[serde(default, rename = "promptSections")]
    pub prompt_sections: Vec<String>,
    pub prompt: Option<String>,
    pub description: Option<String>,
    pub report_format: Option<String>,
    #[serde(default)]
    pub tools: Option<AgentToolDefinition>,
    #[serde(default)]
    pub budgets: Option<AgentBudgetsDefinition>,
    pub mode: Option<AgentMode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentToolDefinition {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentBudgetsDefinition {
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<u32>,
    #[serde(rename = "toolTimeoutMs")]
    pub tool_timeout_ms: Option<u64>,
    #[serde(rename = "wallClockLimitMs")]
    pub wall_clock_ms: Option<u64>,
}

impl AgentBudgetsDefinition {
    pub fn from_json(value: &JsonValue) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}

pub fn resolve_model(spec: &AgentSpec, parent: &ModelHandle) -> ModelHandle {
    spec.model
        .as_ref()
        .map(|model| ModelHandle::new(model.clone()))
        .unwrap_or_else(|| parent.clone())
}

pub fn resolve_tools(spec: &AgentSpec, parent_tools: &[Arc<dyn Tool>]) -> Vec<Arc<dyn Tool>> {
    let mut tools = if let Some(allow) = spec.tool_rules.allow_list() {
        let allow: HashSet<&str> = allow.iter().map(|s| s.as_str()).collect();
        parent_tools
            .iter()
            .filter(|tool| allow.contains(tool.name()))
            .cloned()
            .collect()
    } else {
        parent_tools.to_vec()
    };

    if !spec.tool_rules.deny_list().is_empty() {
        let deny: HashSet<&str> = spec
            .tool_rules
            .deny_list()
            .iter()
            .map(|s| s.as_str())
            .collect();
        tools.retain(|tool| !deny.contains(tool.name()));
    }

    tools
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelHandle {
    id: String,
}

impl ModelHandle {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl std::fmt::Display for ModelHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl std::ops::Deref for ModelHandle {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl From<&str> for ModelHandle {
    fn from(value: &str) -> Self {
        Self {
            id: value.to_string(),
        }
    }
}
