use crate::tool::core::Tool;
use crate::util::error::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

pub struct WebFetchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetches the content of a URL and returns the text content of the body."
    }

    async fn execute(&self, args: &[String]) -> Result<String> {
        if args.len() != 1 {
            return Ok("Usage: web_fetch <url>".to_string());
        }
        let url = &args[0];
        let response = reqwest::get(url).await?.text().await?;
        let document = Html::parse_document(&response);
        let selector = Selector::parse("body").unwrap();
        let body = document.select(&selector).next().unwrap();
        let text = body.text().collect::<Vec<_>>().join("\n");
        Ok(text)
    }
}
