use crate::tool::core::Tool;
use crate::util::error::Result;
use async_trait::async_trait;
use tokio::process::Command;

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Executes a shell command and returns its output. The first argument is the command to execute, and the following are its arguments."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        if args.is_empty() {
            return Ok("Usage: bash <command> [args...]".to_string());
        }

        let command = &args[0];
        let command_args = &args[1..];

        let output = Command::new(command)
            .args(command_args)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str("---STDOUT---\n");
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("---STDERR---\n");
            result.push_str(&stderr);
        }

        if result.is_empty() {
            result.push_str("[Command executed successfully with no output]");
        }

        Ok(result)
    }
}
