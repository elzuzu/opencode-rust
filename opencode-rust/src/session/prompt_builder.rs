use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::agent::spec::{AgentBudgets, AgentSpec};
use crate::util::config::Info;

const DEFAULT_REPORT_FORMAT: &str =
    "Return a concise summary of the task outcome, including key decisions and follow-up steps.";
const DEFAULT_CONSTRAINT: &str = "Stream only final summaries back to the parent session.";

#[derive(Debug, Clone)]
pub struct ProjectContext {
    root: PathBuf,
    rules: String,
}

impl ProjectContext {
    pub fn gather(root: impl Into<PathBuf>, info: &Info) -> Result<Self> {
        let root_path = root.into();
        let mut sections = Vec::new();

        if let Some(instructions) = &info.instructions {
            if !instructions.is_empty() {
                sections.push(format!(
                    "# Project instructions\n{}",
                    instructions.join("\n")
                ));
            }
        }

        for file in collect_rule_sources(&root_path) {
            if let Ok(content) = fs::read_to_string(&file) {
                let relative = file
                    .strip_prefix(&root_path)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .to_string();
                if !content.trim().is_empty() {
                    sections.push(format!("# Source: {relative}\n{content}"));
                }
            }
        }

        if sections.is_empty() {
            sections.push("No repository-specific instructions were discovered.".to_string());
        }

        Ok(Self {
            root: root_path,
            rules: sections.join("\n\n"),
        })
    }

    pub fn rules(&self) -> &str {
        &self.rules
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

pub struct PromptBuilder<'a> {
    spec: &'a AgentSpec,
    context: &'a ProjectContext,
    objective: &'a str,
}

impl<'a> PromptBuilder<'a> {
    pub fn new(spec: &'a AgentSpec, context: &'a ProjectContext, objective: &'a str) -> Self {
        Self {
            spec,
            context,
            objective,
        }
    }

    pub fn build(&self) -> String {
        let mut role_section = self.spec.prompt_sections.join("\n\n");
        if let Some(description) = &self.spec.description {
            if !description.trim().is_empty() {
                if !role_section.is_empty() {
                    role_section.push_str("\n\n");
                }
                role_section.push_str(description.trim());
            }
        }
        if role_section.is_empty() {
            role_section.push_str("You are a focussed coding agent.");
        }

        let constraints = compose_constraints(&self.spec.budgets);
        let report_format = self
            .spec
            .report_format
            .as_deref()
            .unwrap_or(DEFAULT_REPORT_FORMAT);

        format!(
            "<ROLE>\n{role_section}\n</ROLE>\n\n<OBJECTIVE>\n{}\n</OBJECTIVE>\n\n<PROJECT_RULES>\n{}\n</PROJECT_RULES>\n\n<CONSTRAINTS>\n{}\n</CONSTRAINTS>\n\n<REPORT_FORMAT>\n{}\n</REPORT_FORMAT>\n",
            self.objective.trim(),
            self.context.rules(),
            constraints,
            report_format
        )
    }
}

fn compose_constraints(budgets: &AgentBudgets) -> String {
    let mut items = budgets.describe_constraints();
    items.push(DEFAULT_CONSTRAINT.to_string());
    if items.is_empty() {
        items.push("Operate within normal completion limits.".to_string());
    }
    items
        .into_iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_rule_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_lowercase();
        if file_name == "claude.md" || file_name == "agents.md" {
            files.push(entry.path().to_path_buf());
            continue;
        }
        if entry
            .path()
            .components()
            .any(|component| component.as_os_str() == "migration")
        {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("md") {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    files.sort();
    files
}
