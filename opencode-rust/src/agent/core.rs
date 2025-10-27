use crate::tool::core::Tool;
use crate::util::error::{OpenCodeError, Result};

pub struct Agent {
    tools: Vec<Box<dyn Tool + Send + Sync>>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
        }
    }

    pub fn add_tool<T: Tool + Send + Sync + 'static>(&mut self, tool: T) {
        self.tools.push(Box::new(tool));
    }

    pub async fn run_tool(&self, tool_name: &str, args: &[String]) -> Result<String> {
        for tool in &self.tools {
            if tool.name() == tool_name {
                return tool.execute(args).await;
            }
        }
        Err(OpenCodeError::Config(format!("Tool '{}' not found", tool_name)))
    }
}
