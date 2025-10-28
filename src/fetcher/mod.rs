pub mod gitea;
pub mod github;
pub mod gitlab;
pub mod sourcehut;

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
  base_url: &str,
  token: Option<&str>,
) -> Result<Box<dyn Fetcher>> {
  match provider {
    "github" => Ok(Box::new(github::GitHubFetcher::new()?)),
    "gitlab" => Ok(Box::new(gitlab::GitLabFetcher::new(base_url, token)?)),
    "gitea" => Ok(Box::new(gitea::GiteaFetcher::new(base_url, token)?)),
    "sourcehut" => Ok(Box::new(sourcehut::SourcehutFetcher::new(base_url, token)?)),
    _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider)),
  }
}
