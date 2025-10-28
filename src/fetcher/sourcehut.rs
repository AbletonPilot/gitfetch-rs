use super::Fetcher;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct SourcehutFetcher {
  client: reqwest::Client,
  base_url: String,
  token: Option<String>,
}

impl SourcehutFetcher {
  pub fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
    Ok(Self {
      client: reqwest::Client::new(),
      base_url: base_url.trim_end_matches('/').to_string(),
      token: token.map(String::from),
    })
  }

  fn api_request(&self, endpoint: &str) -> Result<Value> {
    let url = format!("{}/api{}", self.base_url, endpoint);

    let mut req = self.client.get(&url);

    if let Some(token) = &self.token {
      req = req.header("Authorization", format!("token {}", token));
    }

    let rt = tokio::runtime::Runtime::new()?;
    let response =
      rt.block_on(async { req.timeout(std::time::Duration::from_secs(30)).send().await })?;

    if !response.status().is_success() {
      return Err(anyhow::anyhow!(
        "Sourcehut API request failed: {}",
        response.status()
      ));
    }

    let rt = tokio::runtime::Runtime::new()?;
    let data = rt.block_on(async { response.json::<Value>().await })?;

    Ok(data)
  }
}

#[async_trait]
impl Fetcher for SourcehutFetcher {
  async fn get_authenticated_user(&self) -> Result<String> {
    if self.token.is_none() {
      return Err(anyhow::anyhow!(
        "Token required for Sourcehut authentication"
      ));
    }

    let data = self.api_request("/user/profile")?;
    data["username"]
      .as_str()
      .map(String::from)
      .ok_or_else(|| anyhow::anyhow!("Could not get authenticated user"))
  }

  async fn fetch_user_data(&self, username: &str) -> Result<Value> {
    // Sourcehut doesn't have a direct user profile endpoint
    // Return minimal user data
    Ok(serde_json::json!({
      "username": username,
      "name": username,
      "bio": "",
      "website": "",
      "company": "",
    }))
  }

  async fn fetch_user_stats(&self, username: &str, _user_data: Option<&Value>) -> Result<Value> {
    // Fetch user's repositories from git.sr.ht
    let repos_endpoint = format!("/repos?owner={}", username);
    let repos_data = self
      .api_request(&repos_endpoint)
      .unwrap_or_else(|_| serde_json::json!({"results": []}));

    let repos = repos_data["results"]
      .as_array()
      .cloned()
      .unwrap_or_default();

    // Calculate language statistics
    let languages = self.calculate_language_stats(&repos);

    // Sourcehut has minimal stats
    Ok(serde_json::json!({
      "total_stars": 0,
      "total_forks": 0,
      "total_repos": repos.len(),
      "languages": languages,
      "contribution_graph": [],
      "current_streak": 0,
      "longest_streak": 0,
      "total_contributions": 0,
      "pull_requests": {
        "open": 0,
        "awaiting_review": 0,
        "mentions": 0
      },
      "issues": {
        "assigned": 0,
        "created": 0,
        "mentions": 0
      },
    }))
  }
}

impl SourcehutFetcher {
  fn calculate_language_stats(&self, repos: &[Value]) -> Value {
    use std::collections::HashMap;

    let mut language_counts: HashMap<String, i32> = HashMap::new();

    for repo in repos {
      // Sourcehut might not have language field, try to detect from repo data
      if let Some(language) = repo["language"].as_str() {
        if !language.is_empty() {
          let normalized = language.to_lowercase();
          *language_counts.entry(normalized).or_insert(0) += 1;
        }
      }
    }

    let total: i32 = language_counts.values().sum();
    if total == 0 {
      return serde_json::json!({});
    }

    let mut language_percentages: HashMap<String, f64> = HashMap::new();
    for (lang, count) in language_counts {
      let percentage = (count as f64 / total as f64) * 100.0;
      let display_name = lang
        .chars()
        .enumerate()
        .map(|(i, c)| {
          if i == 0 {
            c.to_uppercase().to_string()
          } else {
            c.to_string()
          }
        })
        .collect::<String>();
      language_percentages.insert(display_name, percentage);
    }

    serde_json::to_value(language_percentages).unwrap_or_else(|_| serde_json::json!({}))
  }
}
