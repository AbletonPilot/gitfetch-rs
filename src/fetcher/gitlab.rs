use super::Fetcher;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct GitLabFetcher {
  client: reqwest::Client,
  base_url: String,
  token: Option<String>,
}

impl GitLabFetcher {
  pub fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
    Ok(Self {
      client: reqwest::Client::new(),
      base_url: base_url.trim_end_matches('/').to_string(),
      token: token.map(String::from),
    })
  }

  fn api_request(&self, endpoint: &str) -> Result<Value> {
    let url = format!("{}/api/v4{}", self.base_url, endpoint);

    let mut req = self.client.get(&url);

    if let Some(token) = &self.token {
      req = req.header("PRIVATE-TOKEN", token);
    }

    let rt = tokio::runtime::Runtime::new()?;
    let response =
      rt.block_on(async { req.timeout(std::time::Duration::from_secs(30)).send().await })?;

    if !response.status().is_success() {
      return Err(anyhow::anyhow!(
        "GitLab API request failed: {}",
        response.status()
      ));
    }

    let rt = tokio::runtime::Runtime::new()?;
    let data = rt.block_on(async { response.json::<Value>().await })?;

    Ok(data)
  }
}

#[async_trait]
impl Fetcher for GitLabFetcher {
  async fn get_authenticated_user(&self) -> Result<String> {
    if self.token.is_none() {
      return Err(anyhow::anyhow!("Token required for GitLab authentication"));
    }

    let data = self.api_request("/user")?;
    data["username"]
      .as_str()
      .map(String::from)
      .ok_or_else(|| anyhow::anyhow!("Could not get authenticated user"))
  }

  async fn fetch_user_data(&self, username: &str) -> Result<Value> {
    // Search for user by username
    let users = self.api_request(&format!("/users?username={}", username))?;

    if let Some(user_array) = users.as_array() {
      if let Some(user) = user_array.first() {
        return Ok(user.clone());
      }
    }

    Err(anyhow::anyhow!("User not found: {}", username))
  }

  async fn fetch_user_stats(&self, username: &str, user_data: Option<&Value>) -> Result<Value> {
    let user = if let Some(data) = user_data {
      data.clone()
    } else {
      self.fetch_user_data(username).await?
    };

    let user_id = user["id"]
      .as_u64()
      .ok_or_else(|| anyhow::anyhow!("Invalid user ID"))?;

    // Fetch user's projects
    let mut repos = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
      let endpoint = format!(
        "/users/{}/projects?page={}&per_page={}",
        user_id, page, per_page
      );
      let data = self.api_request(&endpoint)?;

      let data_array = match data.as_array() {
        Some(arr) if !arr.is_empty() => arr,
        _ => break,
      };

      repos.extend(data_array.clone());
      page += 1;

      if data_array.len() < per_page {
        break;
      }
    }

    // Calculate statistics
    let total_stars: i64 = repos.iter().filter_map(|r| r["star_count"].as_i64()).sum();

    let total_forks: i64 = repos.iter().filter_map(|r| r["forks_count"].as_i64()).sum();

    // Calculate language statistics
    let languages = self.calculate_language_stats(&repos);

    // GitLab doesn't have contribution graphs like GitHub
    // Return simplified stats
    Ok(serde_json::json!({
      "total_stars": total_stars,
      "total_forks": total_forks,
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

impl GitLabFetcher {
  fn calculate_language_stats(&self, repos: &[Value]) -> Value {
    use std::collections::HashMap;

    let mut language_counts: HashMap<String, i32> = HashMap::new();

    for repo in repos {
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
