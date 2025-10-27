use crate::tool::core::Tool;
use crate::util::error::{OpenCodeError, Result};
use std::sync::Arc;

pub struct Agent {
    tools: Vec<Arc<dyn Tool>>,
}

impl Agent {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn add_tool<T>(&mut self, tool: T)
    where
        T: Tool + 'static,
    {
        self.tools.push(Arc::new(tool));
    }

    pub async fn run_tool(&self, tool_name: &str, args: &[String]) -> Result<String> {
        for tool in &self.tools {
            if tool.name() == tool_name {
                return tool.execute(args).await;
            }
        }
        Err(OpenCodeError::Config(format!(
            "Tool '{}' not found",
            tool_name
        )))
    }

    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}
