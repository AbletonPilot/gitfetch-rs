use super::Fetcher;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct GiteaFetcher {
  client: reqwest::Client,
  api_base: String,
  token: Option<String>,
}

impl GiteaFetcher {
  pub fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
    let base = base_url.trim_end_matches('/').to_string();
    let api_base = format!("{}/api/v1", base);

    Ok(Self {
      client: reqwest::Client::new(),
      api_base,
      token: token.map(String::from),
    })
  }

  fn api_request(&self, endpoint: &str) -> Result<Value> {
    let url = format!("{}{}", self.api_base, endpoint);

    let mut req = self.client.get(&url);

    if let Some(token) = &self.token {
      req = req.header("Authorization", format!("token {}", token));
    }

    let rt = tokio::runtime::Runtime::new()?;
    let response =
      rt.block_on(async { req.timeout(std::time::Duration::from_secs(30)).send().await })?;

    if !response.status().is_success() {
      return Err(anyhow::anyhow!(
        "Gitea API request failed: {}",
        response.status()
      ));
    }

    let rt = tokio::runtime::Runtime::new()?;
    let data = rt.block_on(async { response.json::<Value>().await })?;

    Ok(data)
  }
}

#[async_trait]
impl Fetcher for GiteaFetcher {
  async fn get_authenticated_user(&self) -> Result<String> {
    if self.token.is_none() {
      return Err(anyhow::anyhow!("Token required for Gitea authentication"));
    }

    let data = self.api_request("/user")?;
    data["login"]
      .as_str()
      .map(String::from)
      .ok_or_else(|| anyhow::anyhow!("Could not get authenticated user"))
  }

  async fn fetch_user_data(&self, username: &str) -> Result<Value> {
    self.api_request(&format!("/users/{}", username))
  }

  async fn fetch_user_stats(&self, username: &str, user_data: Option<&Value>) -> Result<Value> {
    let _user = if let Some(data) = user_data {
      data.clone()
    } else {
      self.fetch_user_data(username).await?
    };

    // Fetch user's repositories
    let mut repos = Vec::new();
    let mut page = 1;
    let per_page = 50;

    loop {
      let endpoint = format!("/users/{}/repos?page={}&limit={}", username, page, per_page);
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
    let total_stars: i64 = repos.iter().filter_map(|r| r["stars_count"].as_i64()).sum();

    let total_forks: i64 = repos.iter().filter_map(|r| r["forks_count"].as_i64()).sum();

    // Calculate language statistics
    let languages = self.calculate_language_stats(&repos);

    // Gitea doesn't have contribution graphs
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

impl GiteaFetcher {
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
