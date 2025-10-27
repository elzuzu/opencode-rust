use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::de::{DeserializeOwned, Error as DeError};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{self, Map as JsonMap, Number, Value as JsonValue};
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Info {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,

    pub theme: Option<String>,

    #[serde(default)]
    pub keybinds: Option<Keybinds>,

    #[serde(default)]
    pub tui: Option<Tui>,

    #[serde(default)]
    pub command: Option<HashMap<String, Command>>,

    #[serde(default)]
    pub watcher: Option<WatcherSettings>,

    #[serde(default)]
    pub plugin: Option<Vec<String>>,

    pub snapshot: Option<bool>,

    pub share: Option<ShareMode>,

    pub autoshare: Option<bool>,

    pub autoupdate: Option<bool>,

    #[serde(default)]
    pub disabled_providers: Option<Vec<String>>,

    pub model: Option<String>,

    pub small_model: Option<String>,

    pub username: Option<String>,

    #[serde(default)]
    pub mode: Option<HashMap<String, AgentConfig>>,

    #[serde(default)]
    pub agent: Option<HashMap<String, AgentConfig>>,

    #[serde(default)]
    pub provider: Option<HashMap<String, ProviderConfig>>,

    #[serde(default)]
    pub mcp: Option<HashMap<String, McpConfig>>,

    #[serde(default)]
    pub formatter: Option<HashMap<String, FormatterConfig>>,

    #[serde(default)]
    pub lsp: Option<HashMap<String, LspConfig>>,

    #[serde(default)]
    pub instructions: Option<Vec<String>>,

    pub layout: Option<Layout>,

    #[serde(default)]
    pub permission: Option<PermissionMatrix>,

    #[serde(default)]
    pub tools: Option<HashMap<String, bool>>,

    #[serde(default)]
    pub experimental: Option<ExperimentalConfig>,
}

impl Info {
    pub fn watcher_ignore_patterns(&self) -> Vec<String> {
        self.watcher
            .as_ref()
            .map(|w| w.ignore.clone())
            .unwrap_or_default()
    }

    pub fn merge(&mut self, other: Info) {
        merge_info_map(&mut self.command, other.command);
        merge_info_map(&mut self.mode, other.mode);
        merge_info_map(&mut self.agent, other.agent);
        merge_info_map(&mut self.provider, other.provider);
        merge_info_map(&mut self.mcp, other.mcp);
        merge_info_map(&mut self.formatter, other.formatter);
        merge_info_map(&mut self.lsp, other.lsp);
        merge_info_map(&mut self.tools, other.tools);

        merge_vec(&mut self.plugin, other.plugin);
        merge_vec(&mut self.instructions, other.instructions);

        overwrite_if_some(&mut self.schema, other.schema);
        overwrite_if_some(&mut self.theme, other.theme);
        overwrite_if_some(&mut self.keybinds, other.keybinds);
        overwrite_if_some(&mut self.tui, other.tui);
        overwrite_if_some(&mut self.watcher, other.watcher);
        overwrite_if_some(&mut self.snapshot, other.snapshot);
        overwrite_if_some(&mut self.share, other.share);
        overwrite_if_some(&mut self.autoshare, other.autoshare);
        overwrite_if_some(&mut self.autoupdate, other.autoupdate);
        overwrite_if_some(&mut self.disabled_providers, other.disabled_providers);
        overwrite_if_some(&mut self.model, other.model);
        overwrite_if_some(&mut self.small_model, other.small_model);
        overwrite_if_some(&mut self.username, other.username);
        overwrite_if_some(&mut self.layout, other.layout);
        overwrite_if_some(&mut self.permission, other.permission);
        overwrite_if_some(&mut self.experimental, other.experimental);
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct WatcherSettings {
    #[serde(default)]
    #[validate(custom(function = "validate_glob_patterns"))]
    pub ignore: Vec<String>,
}

impl Default for WatcherSettings {
    fn default() -> Self {
        Self { ignore: Vec::new() }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Validate)]
pub struct Keybinds {
    #[serde(default)]
    #[validate(length(min = 1))]
    pub leader: Option<String>,
    #[serde(default)]
    pub app_help: Option<String>,
    #[serde(default)]
    pub app_exit: Option<String>,
    #[serde(default)]
    pub editor_open: Option<String>,
    #[serde(default)]
    pub theme_list: Option<String>,
    #[serde(default)]
    pub project_init: Option<String>,
    #[serde(default)]
    pub tool_details: Option<String>,
    #[serde(default)]
    pub thinking_blocks: Option<String>,
    #[serde(default)]
    pub session_export: Option<String>,
    #[serde(default)]
    pub session_new: Option<String>,
    #[serde(default)]
    pub session_list: Option<String>,
    #[serde(default)]
    pub session_timeline: Option<String>,
    #[serde(default)]
    pub session_share: Option<String>,
    #[serde(default)]
    pub session_unshare: Option<String>,
    #[serde(default)]
    pub session_interrupt: Option<String>,
    #[serde(default)]
    pub session_compact: Option<String>,
    #[serde(default)]
    pub session_child_cycle: Option<String>,
    #[serde(default)]
    pub session_child_cycle_reverse: Option<String>,
    #[serde(default)]
    pub messages_page_up: Option<String>,
    #[serde(default)]
    pub messages_page_down: Option<String>,
    #[serde(default)]
    pub messages_half_page_up: Option<String>,
    #[serde(default)]
    pub messages_half_page_down: Option<String>,
    #[serde(default)]
    pub messages_first: Option<String>,
    #[serde(default)]
    pub messages_last: Option<String>,
    #[serde(default)]
    pub messages_copy: Option<String>,
    #[serde(default)]
    pub messages_undo: Option<String>,
    #[serde(default)]
    pub messages_redo: Option<String>,
    #[serde(default)]
    pub model_list: Option<String>,
    #[serde(default)]
    pub model_cycle_recent: Option<String>,
    #[serde(default)]
    pub model_cycle_recent_reverse: Option<String>,
    #[serde(default)]
    pub agent_list: Option<String>,
    #[serde(default)]
    pub agent_cycle: Option<String>,
    #[serde(default)]
    pub agent_cycle_reverse: Option<String>,
    #[serde(default)]
    pub input_clear: Option<String>,
    #[serde(default)]
    pub input_paste: Option<String>,
    #[serde(default)]
    pub input_submit: Option<String>,
    #[serde(default)]
    pub input_newline: Option<String>,
    #[serde(default)]
    pub switch_mode: Option<String>,
    #[serde(default)]
    pub switch_mode_reverse: Option<String>,
    #[serde(default)]
    pub switch_agent: Option<String>,
    #[serde(default)]
    pub switch_agent_reverse: Option<String>,
    #[serde(default)]
    pub file_list: Option<String>,
    #[serde(default)]
    pub file_close: Option<String>,
    #[serde(default)]
    pub file_search: Option<String>,
    #[serde(default)]
    pub file_diff_toggle: Option<String>,
    #[serde(default)]
    pub messages_previous: Option<String>,
    #[serde(default)]
    pub messages_next: Option<String>,
    #[serde(default)]
    pub messages_layout_toggle: Option<String>,
    #[serde(default)]
    pub messages_revert: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Tui {
    #[serde(default = "default_scroll_speed")]
    #[validate(range(min = 1))]
    pub scroll_speed: u32,
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            scroll_speed: default_scroll_speed(),
        }
    }
}

const fn default_scroll_speed() -> u32 {
    2
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareMode {
    Manual,
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Auto,
    Stretch,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Command {
    #[validate(length(min = 1))]
    pub template: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub agent: Option<String>,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub subtask: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AgentConfig {
    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub temperature: Option<f64>,

    #[serde(default)]
    pub top_p: Option<f64>,

    #[serde(default)]
    pub prompt: Option<String>,

    #[serde(default)]
    pub tools: Option<HashMap<String, bool>>,

    #[serde(default)]
    pub disable: Option<bool>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub mode: Option<AgentMode>,

    #[serde(default)]
    pub permission: Option<PermissionMatrix>,

    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Subagent,
    Primary,
    All,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub models: Option<HashMap<String, JsonValue>>,

    #[serde(default)]
    pub options: Option<ProviderOptions>,

    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Deserialize, Validate)]
pub struct ProviderOptions {
    #[serde(rename = "apiKey", default)]
    #[validate(length(min = 1))]
    pub api_key: Option<String>,

    #[serde(rename = "baseURL", default)]
    #[validate(length(min = 1))]
    pub base_url: Option<String>,

    #[serde(default, deserialize_with = "deserialize_timeout")]
    #[validate(custom(function = "validate_timeout_option"))]
    pub timeout: Option<Timeout>,

    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Timeout {
    Millis(u64),
    Disabled,
}

fn deserialize_timeout<'de, D>(deserializer: D) -> std::result::Result<Option<Timeout>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = JsonValue::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    if let Some(number) = value.as_u64() {
        return Ok(Some(Timeout::Millis(number)));
    }
    if let Some(boolean) = value.as_bool() {
        if !boolean {
            return Ok(Some(Timeout::Disabled));
        }
        return Err(DeError::custom("timeout boolean must be false to disable"));
    }
    Err(DeError::custom(
        "timeout must be positive integer milliseconds or false",
    ))
}

fn validate_timeout_option(value: &Timeout) -> std::result::Result<(), ValidationError> {
    if let Timeout::Millis(ms) = value {
        if *ms == 0 {
            let mut error = ValidationError::new("timeout");
            error.message = Some("timeout must be greater than zero".into());
            return Err(error);
        }
    }
    Ok(())
}

fn validate_glob_patterns(patterns: &Vec<String>) -> std::result::Result<(), ValidationError> {
    for pattern in patterns {
        if pattern.trim().is_empty() {
            let mut error = ValidationError::new("glob");
            error.message = Some("ignore patterns must not be empty".into());
            return Err(error);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum McpConfig {
    #[serde(rename = "local")]
    Local(LocalMcpConfig),

    #[serde(rename = "remote")]
    Remote(RemoteMcpConfig),
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LocalMcpConfig {
    #[validate(length(min = 1))]
    pub command: Vec<String>,

    #[serde(default)]
    pub environment: Option<HashMap<String, String>>,

    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RemoteMcpConfig {
    #[validate(url)]
    pub url: String,

    #[serde(default)]
    pub enabled: Option<bool>,

    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionKind {
    Ask,
    Allow,
    Deny,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionMatrix {
    #[serde(default)]
    pub edit: Option<PermissionKind>,

    #[serde(default)]
    pub bash: Option<BashPermission>,

    #[serde(default)]
    pub webfetch: Option<PermissionKind>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BashPermission {
    Single(PermissionKind),
    Matrix(HashMap<String, PermissionKind>),
}

#[derive(Debug, Clone, Default, Deserialize, Validate)]
pub struct FormatterConfig {
    #[serde(default)]
    pub disabled: Option<bool>,

    #[serde(default)]
    #[validate(length(min = 1))]
    pub command: Option<Vec<String>>,

    #[serde(default)]
    pub environment: Option<HashMap<String, String>>,

    #[serde(default)]
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LspConfig {
    Disabled(LspDisabledConfig),
    Configurable(LspServerConfig),
}

impl Default for LspConfig {
    fn default() -> Self {
        LspConfig::Disabled(LspDisabledConfig { disabled: true })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspDisabledConfig {
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LspServerConfig {
    #[validate(length(min = 1))]
    pub command: Vec<String>,

    #[serde(default)]
    #[validate(length(min = 1))]
    pub extensions: Option<Vec<String>>,

    #[serde(default)]
    pub disabled: Option<bool>,

    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    #[serde(default)]
    pub initialization: Option<HashMap<String, JsonValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentalConfig {
    #[serde(default)]
    pub hook: Option<HookConfig>,

    #[serde(default)]
    pub disable_paste_summary: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HookConfig {
    #[serde(default, rename = "file_edited")]
    pub file_edited: Option<HashMap<String, Vec<HookCommand>>>,

    #[serde(default, rename = "session_completed")]
    pub session_completed: Option<Vec<HookCommand>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct HookCommand {
    #[validate(length(min = 1))]
    pub command: Vec<String>,

    #[serde(default)]
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct FrontMatter<T> {
    pub data: T,
    pub content: String,
}

pub fn parse_jsonc<T>(text: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let stripped = strip_json_comments(text);
    let cleaned = remove_trailing_commas(&stripped);
    let parsed = serde_json::from_str(&cleaned)?;
    Ok(parsed)
}

pub fn parse_info(text: &str) -> Result<Info> {
    let info: Info = parse_jsonc(text)?;
    info.validate()?;
    Ok(info)
}

pub async fn load_info_from_path(path: &Path) -> Result<Info> {
    let data = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    parse_info(&data)
}

pub fn parse_front_matter<T>(input: &str) -> Result<FrontMatter<T>>
where
    T: DeserializeOwned,
{
    let trimmed = input.trim_start_matches('\u{feff}').trim_start();
    if !trimmed.starts_with("---") {
        let data: T = serde_json::from_str("{}")?;
        return Ok(FrontMatter {
            data,
            content: trimmed.trim().to_string(),
        });
    }

    let mut lines = trimmed.lines();
    let first_line = lines.next().unwrap_or("");
    if first_line.trim() != "---" {
        bail!("front matter must start with '---'");
    }

    let mut yaml_lines = Vec::new();
    let mut content_lines = Vec::new();
    let mut in_yaml = true;

    for line in lines {
        if in_yaml && line.trim() == "---" {
            in_yaml = false;
            continue;
        }
        if in_yaml {
            yaml_lines.push(line);
        } else {
            content_lines.push(line);
        }
    }

    if in_yaml {
        bail!("unterminated front matter, expected closing '---'");
    }

    let yaml_src = yaml_lines.join("\n");
    let yaml_value = if yaml_src.trim().is_empty() {
        JsonValue::Null
    } else {
        parse_yaml_like(&yaml_src)?
    };

    let data: T = serde_json::from_value(yaml_value)?;

    let content = content_lines.join("\n");
    Ok(FrontMatter {
        data,
        content: content.trim().to_string(),
    })
}

fn strip_json_comments(input: &str) -> String {
    use std::iter::Peekable;

    let mut output = String::with_capacity(input.len());
    let mut chars: Peekable<_> = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut in_single_comment = false;
    let mut in_multi_comment = false;

    while let Some(ch) = chars.next() {
        if in_single_comment {
            if ch == '\r' {
                output.push(ch);
                continue;
            }
            if ch == '\n' {
                in_single_comment = false;
                output.push(ch);
            }
            continue;
        }

        if in_multi_comment {
            if ch == '*' {
                if matches!(chars.peek(), Some('/')) {
                    chars.next();
                    in_multi_comment = false;
                }
            }
            if ch == '\n' || ch == '\r' {
                output.push(ch);
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            escaped = false;
            output.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    in_single_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_multi_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        output.push(ch);
    }

    output
}

fn remove_trailing_commas(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in input.chars() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                output.push(ch);
            }
            ']' | '}' => {
                trim_trailing_comma(&mut output);
                output.push(ch);
            }
            _ => output.push(ch),
        }
    }

    output
}

fn trim_trailing_comma(buffer: &mut String) {
    let mut whitespace = Vec::new();
    while let Some(ch) = buffer.pop() {
        if ch.is_whitespace() {
            whitespace.push(ch);
            continue;
        }
        if ch == ',' {
            while let Some(ws) = whitespace.pop() {
                buffer.push(ws);
            }
            return;
        }
        buffer.push(ch);
        while let Some(ws) = whitespace.pop() {
            buffer.push(ws);
        }
        return;
    }
    while let Some(ws) = whitespace.pop() {
        buffer.push(ws);
    }
}

fn push_field_error(
    errors: &mut ValidationErrors,
    field: &'static str,
    code: &'static str,
    message: &'static str,
) {
    let mut error = ValidationError::new(code);
    error.message = Some(message.into());
    errors.add(field, error);
}

fn push_struct_error(
    errors: &mut ValidationErrors,
    field: impl Into<Cow<'static, str>>,
    err: ValidationErrors,
) {
    errors
        .0
        .insert(field.into(), ValidationErrorsKind::Struct(Box::new(err)));
}

impl Validate for LspConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            LspConfig::Disabled(_) => Ok(()),
            LspConfig::Configurable(config) => {
                config.validate()?;
                if config.disabled.unwrap_or(false) {
                    return Ok(());
                }
                if let Some(extensions) = &config.extensions {
                    if extensions.is_empty() {
                        let mut errors = ValidationErrors::new();
                        push_field_error(
                            &mut errors,
                            "extensions",
                            "length",
                            "extensions must not be empty for enabled LSP servers",
                        );
                        return Err(errors);
                    }
                } else {
                    let mut errors = ValidationErrors::new();
                    push_field_error(
                        &mut errors,
                        "extensions",
                        "required",
                        "extensions are required for enabled LSP servers",
                    );
                    return Err(errors);
                }
                Ok(())
            }
        }
    }
}
fn merge_info_map<T>(target: &mut Option<HashMap<String, T>>, source: Option<HashMap<String, T>>) {
    if let Some(mut source_map) = source {
        let target_map = target.get_or_insert_with(HashMap::new);
        for (key, value) in source_map.drain() {
            target_map.insert(key, value);
        }
    }
}

fn merge_vec<T>(target: &mut Option<Vec<T>>, source: Option<Vec<T>>) {
    if let Some(mut source_vec) = source {
        let target_vec = target.get_or_insert_with(Vec::new);
        target_vec.append(&mut source_vec);
    }
}

fn overwrite_if_some<T>(target: &mut Option<T>, source: Option<T>) {
    if source.is_some() {
        *target = source;
    }
}

fn parse_yaml_like(input: &str) -> Result<JsonValue> {
    let mut map = JsonMap::new();
    let mut pending_key: Option<String> = None;
    let mut pending_values: Vec<JsonValue> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(key) = pending_key.clone() {
            if trimmed.starts_with('-') {
                let value = trimmed.trim_start_matches('-').trim();
                pending_values.push(parse_scalar(value));
                continue;
            }
            map.insert(key, JsonValue::Array(std::mem::take(&mut pending_values)));
            pending_key = None;
        }

        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let remainder = line[idx + 1..].trim();
            if remainder.is_empty() {
                pending_key = Some(key);
                pending_values.clear();
                continue;
            }
            let value = parse_scalar(remainder);
            map.insert(key, value);
            continue;
        }

        if trimmed.starts_with('-') {
            bail!("unexpected list item without key in front matter");
        }
    }

    if let Some(key) = pending_key {
        map.insert(key, JsonValue::Array(pending_values));
    }

    Ok(JsonValue::Object(map))
}

fn parse_scalar(value: &str) -> JsonValue {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return JsonValue::Null;
    }

    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        if let Ok(parsed) = serde_json::from_str::<String>(trimmed) {
            return JsonValue::String(parsed);
        }
    }

    let apostrophe = '\u{27}';
    if trimmed.starts_with(apostrophe) && trimmed.ends_with(apostrophe) && trimmed.len() >= 2 {
        return JsonValue::String(trimmed[1..trimmed.len() - 1].to_string());
    }

    if trimmed.eq_ignore_ascii_case("true") {
        return JsonValue::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return JsonValue::Bool(false);
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return JsonValue::Null;
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_array_literal(trimmed);
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        if let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) {
            return value;
        }
    }

    if let Ok(int) = trimmed.parse::<i64>() {
        return JsonValue::Number(Number::from(int));
    }

    if let Ok(float) = trimmed.parse::<f64>() {
        if let Some(number) = Number::from_f64(float) {
            return JsonValue::Number(number);
        }
    }

    JsonValue::String(trimmed.to_string())
}

fn parse_array_literal(value: &str) -> JsonValue {
    if let Ok(parsed) = serde_json::from_str::<JsonValue>(value) {
        return parsed;
    }

    let inner = &value[1..value.len() - 1];
    let mut result = Vec::new();
    for item in inner.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        result.push(parse_scalar(trimmed));
    }
    JsonValue::Array(result)
}

impl Validate for Info {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        let mut has_error = false;

        if let Some(schema) = &self.schema {
            if schema.trim().is_empty() {
                push_field_error(
                    &mut errors,
                    "$schema",
                    "length",
                    "$schema must not be empty",
                );
                has_error = true;
            }
        }
        if let Some(theme) = &self.theme {
            if theme.trim().is_empty() {
                push_field_error(&mut errors, "theme", "length", "theme must not be empty");
                has_error = true;
            }
        }
        if let Some(username) = &self.username {
            if username.trim().is_empty() {
                push_field_error(
                    &mut errors,
                    "username",
                    "length",
                    "username must not be empty",
                );
                has_error = true;
            }
        }
        if let Some(keybinds) = &self.keybinds {
            if let Err(err) = keybinds.validate() {
                push_struct_error(&mut errors, "keybinds", err);
                has_error = true;
            }
        }
        if let Some(tui) = &self.tui {
            if let Err(err) = tui.validate() {
                push_struct_error(&mut errors, "tui", err);
                has_error = true;
            }
        }
        if let Some(watcher) = &self.watcher {
            if let Err(err) = watcher.validate() {
                push_struct_error(&mut errors, "watcher", err);
                has_error = true;
            }
        }
        if let Some(providers) = &self.provider {
            for (name, config) in providers {
                if let Err(err) = config.validate() {
                    push_struct_error(&mut errors, format!("provider.{name}"), err);
                    has_error = true;
                }
            }
        }
        if let Some(mcp) = &self.mcp {
            for (name, config) in mcp {
                if let Err(err) = config.validate() {
                    push_struct_error(&mut errors, format!("mcp.{name}"), err);
                    has_error = true;
                }
            }
        }
        if let Some(formatters) = &self.formatter {
            for (name, config) in formatters {
                if let Err(err) = config.validate() {
                    push_struct_error(&mut errors, format!("formatter.{name}"), err);
                    has_error = true;
                }
            }
        }
        if let Some(lsps) = &self.lsp {
            for (name, config) in lsps {
                if let Err(err) = config.validate() {
                    push_struct_error(&mut errors, format!("lsp.{name}"), err);
                    has_error = true;
                }
            }
        }
        if let Some(experimental) = &self.experimental {
            if let Err(err) = experimental.validate() {
                push_struct_error(&mut errors, "experimental", err);
                has_error = true;
            }
        }

        if has_error { Err(errors) } else { Ok(()) }
    }
}

impl Validate for ProviderConfig {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        if let Some(options) = &self.options {
            options.validate()?;
        }
        Ok(())
    }
}

impl Validate for McpConfig {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        match self {
            McpConfig::Local(cfg) => cfg.validate(),
            McpConfig::Remote(cfg) => cfg.validate(),
        }
    }
}

impl Validate for ExperimentalConfig {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        if let Some(hook) = &self.hook {
            hook.validate()?;
        }
        Ok(())
    }
}

impl Validate for HookConfig {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        let mut has_error = false;

        if let Some(map) = &self.file_edited {
            for (name, commands) in map {
                for (idx, command) in commands.iter().enumerate() {
                    if let Err(err) = command.validate() {
                        push_struct_error(&mut errors, format!("file_edited.{name}[{idx}]"), err);
                        has_error = true;
                    }
                }
            }
        }

        if let Some(commands) = &self.session_completed {
            if commands.is_empty() {
                push_field_error(
                    &mut errors,
                    "session_completed",
                    "length",
                    "session_completed must contain at least one hook",
                );
                has_error = true;
            } else {
                for (idx, command) in commands.iter().enumerate() {
                    if let Err(err) = command.validate() {
                        push_struct_error(&mut errors, format!("session_completed[{idx}]"), err);
                        has_error = true;
                    }
                }
            }
        }

        if has_error { Err(errors) } else { Ok(()) }
    }
}
