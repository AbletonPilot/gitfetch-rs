pub mod github;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Fetcher: Send + Sync {
  async fn get_authenticated_user(&self) -> Result<String>;
  async fn fetch_user_data(&self, username: &str) -> Result<Value>;
  async fn fetch_user_stats(&self, username: &str, user_data: Option<&Value>) -> Result<Value>;
}

pub fn create_fetcher(
  provider: &str,
  _base_url: &str,
  _token: Option<&str>,
) -> Result<Box<dyn Fetcher>> {
  match provider {
    "github" => Ok(Box::new(github::GitHubFetcher::new()?)),
    _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider)),
  }
}
