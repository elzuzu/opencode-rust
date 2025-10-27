use crate::tool::core::Tool;
use crate::util::error::Result;
use async_trait::async_trait;
use tokio::fs;
use walkdir::WalkDir;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Reads the entire content of a file and returns it as a string."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        if args.len() != 1 {
            return Ok("Usage: read_file <path>".to_string());
        }
        let path = &args[0];
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Writes content to a file, overwriting it if it exists or creating it if it doesn't. The first argument is the file path, and the second is the content to write."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        if args.len() != 2 {
            return Ok("Usage: write_file <path> <content>".to_string());
        }
        let path = &args[0];
        let content = &args[1];
        fs::write(path, content).await?;
        Ok(format!("File {} written successfully.", path))
    }
}

pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "Lists all files and directories under the given directory (defaults to current directory). Directories in the output will have a trailing slash."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        let path = args.get(0).map_or(".", |s| s.as_str());
        let mut result = String::new();
        for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy();
            if entry.file_type().is_dir() {
                result.push_str(&format!("{}/\n", file_name));
            } else {
                result.push_str(&format!("{}\n", file_name));
            }
        }
        Ok(result)
    }
}
