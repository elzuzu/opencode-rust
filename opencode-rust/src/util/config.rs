use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct Info {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub theme: Option<String>,
    // #[validate]
    // pub keybinds: Option<Keybinds>,
    // #[validate]
    // pub tui: Option<TUI>,
    // #[validate]
    // pub command: Option<std::collections::HashMap<String, Command>>,
    // pub watcher: Option<Watcher>,
    // pub plugin: Option<Vec<String>>,
    // pub snapshot: Option<bool>,
    // pub share: Option<String>,
    // pub autoshare: Option<bool>,
    // pub autoupdate: Option<bool>,
    // pub disabled_providers: Option<Vec<String>>,
    // pub model: Option<String>,
    // pub small_model: Option<String>,
    // pub username: Option<String>,
    // #[validate]
    // pub mode: Option<std::collections::HashMap<String, Agent>>,
    // #[validate]
    // pub agent: Option<std::collections::HashMap<String, Agent>>,
    // pub provider: Option<std::collections::HashMap<String, Provider>>,
    // pub mcp: Option<std::collections::HashMap<String, Mcp>>,
    // pub formatter: Option<std::collections::HashMap<String, Formatter>>,
    // pub lsp: Option<std::collections::HashMap<String, Lsp>>,
    // pub instructions: Option<Vec<String>>,
    // pub layout: Option<String>,
    // pub permission: Option<Permission>,
    // pub tools: Option<std::collections::HashMap<String, bool>>,
    // pub experimental: Option<Experimental>,
}
