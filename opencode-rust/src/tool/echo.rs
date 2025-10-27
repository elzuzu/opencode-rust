use crate::tool::core::Tool;
use crate::util::error::Result;
use async_trait::async_trait;

pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "A simple tool that echoes the input back."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        Ok(args.join(" "))
    }
}
