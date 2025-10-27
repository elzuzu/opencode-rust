use opencode_rust::util::config;
use serde::Deserialize;
use std::collections::HashMap;

#[test]
fn parses_minimal_configuration() {
    let info = config::parse_info("{}").unwrap();
    assert!(info.schema.is_none());
    assert!(info.theme.is_none());
    assert!(info.agent.is_none());
}

#[test]
fn parses_jsonc_with_comments_and_trailing_commas() {
    let source = r#"
    {
        // comment that should be ignored
        "theme": "dark",
        "watcher": {
            "ignore": ["**/*.tmp",]
        },
    }
    "#;

    let info = config::parse_info(source).unwrap();
    assert_eq!(info.theme.as_deref(), Some("dark"));
    assert_eq!(info.watcher.unwrap().ignore, vec!["**/*.tmp".to_string()]);
}

#[test]
fn rejects_invalid_validation_rules() {
    let source = r#"
    {
        "tui": {
            "scroll_speed": 0
        }
    }
    "#;

    let error = config::parse_info(source).unwrap_err();
    assert!(error.to_string().contains("scroll_speed"));
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct CommandMetadata {
    name: String,
    agent: String,
}

#[test]
fn parses_markdown_front_matter() {
    let doc = r#"
---
name: build
agent: builder
---
echo "{{input}}"
"#;

    let parsed = config::parse_front_matter::<CommandMetadata>(doc).unwrap();
    assert_eq!(
        parsed.data,
        CommandMetadata {
            name: "build".into(),
            agent: "builder".into()
        }
    );
    assert_eq!(parsed.content, "echo \"{{input}}\"");
}

#[derive(Debug, Deserialize, PartialEq)]
struct ExtendedMetadata {
    tags: Vec<String>,
    retries: u32,
    enabled: bool,
    script: String,
    env: Option<HashMap<String, String>>,
}

#[test]
fn parses_front_matter_with_lists_and_scalars() {
    let doc = r#"
---
tags:
  - build
  - deploy
retries: 3
enabled: true
script: 'deploy.sh'
env: {"NODE_ENV": "production"}
---
npm run build
"#;

    let parsed = config::parse_front_matter::<ExtendedMetadata>(doc).unwrap();
    assert_eq!(parsed.data.tags, vec!["build", "deploy"]);
    assert_eq!(parsed.data.retries, 3);
    assert!(parsed.data.enabled);
    assert_eq!(parsed.data.script, "deploy.sh");
    assert_eq!(
        parsed
            .data
            .env
            .as_ref()
            .and_then(|env| env.get("NODE_ENV"))
            .map(String::as_str),
        Some("production"),
    );
    assert_eq!(parsed.content, "npm run build");
}

#[test]
fn merge_updates_watcher_ignore_patterns() {
    let mut base =
        config::parse_info(r#"{"watcher": {"ignore": ["**/*.log"]}, "theme": "dark"}"#).unwrap();

    base.merge(config::parse_info("{}").unwrap());
    assert_eq!(base.watcher_ignore_patterns(), vec!["**/*.log"]);

    base.merge(config::parse_info(r#"{"watcher": {"ignore": ["**/*.tmp"]}}"#).unwrap());
    assert_eq!(base.watcher_ignore_patterns(), vec!["**/*.tmp"]);
}
