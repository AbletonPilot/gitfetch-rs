use super::Fetcher;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;

pub struct GitHubFetcher {
  _client: reqwest::Client,
}

impl GitHubFetcher {
  pub fn new() -> Result<Self> {
    Self::check_gh_cli()?;

    Ok(Self {
      _client: reqwest::Client::new(),
    })
  }

  fn check_gh_cli() -> Result<()> {
    let output = Command::new("gh").args(&["auth", "status"]).output();

    match output {
      Ok(out) if out.status.success() => Ok(()),
      Ok(_) => Err(anyhow::anyhow!(
        "GitHub CLI not authenticated. Run: gh auth login"
      )),
      Err(_) => Err(anyhow::anyhow!("GitHub CLI not installed")),
    }
  }

  fn gh_api(&self, endpoint: &str) -> Result<Value> {
    let output = Command::new("gh").args(&["api", endpoint]).output()?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("gh api failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let data: Value = serde_json::from_str(&stdout)?;
    Ok(data)
  }

  fn gh_graphql(&self, query: &str) -> Result<Value> {
    let output = Command::new("gh")
      .args(&["api", "graphql", "-f", &format!("query={}", query)])
      .output()?;

    if !output.status.success() {
      return Ok(serde_json::json!({}));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let data: Value = serde_json::from_str(&stdout)?;
    Ok(data)
  }
}

#[async_trait]
impl Fetcher for GitHubFetcher {
  async fn get_authenticated_user(&self) -> Result<String> {
    let data = self.gh_api("/user")?;
    data["login"]
      .as_str()
      .map(String::from)
      .ok_or_else(|| anyhow::anyhow!("Could not get authenticated user"))
  }

  async fn fetch_user_data(&self, username: &str) -> Result<Value> {
    self.gh_api(&format!("/users/{}", username))
  }

  async fn fetch_user_stats(&self, username: &str, _user_data: Option<&Value>) -> Result<Value> {
    // Fetch all public repositories (matching Python behavior)
    let repos = self.fetch_repos(username)?;

    let total_stars: i64 = repos
      .iter()
      .filter_map(|r| r["stargazers_count"].as_i64())
      .sum();
    let total_forks: i64 = repos.iter().filter_map(|r| r["forks_count"].as_i64()).sum();

    // Calculate language statistics
    let languages = self.calculate_language_stats(&repos);

    let contrib_graph = match self.fetch_contribution_graph(username) {
      Ok(graph) => graph,
      Err(e) => {
        eprintln!("Warning: Failed to fetch contribution graph: {}", e);
        serde_json::json!([])
      }
    };

    let (current_streak, longest_streak, total_contributions) =
      self.calculate_contribution_stats(&contrib_graph);

    // Get search username (@me for authenticated user, otherwise username)
    let search_username = self.get_search_username(username);

    // Fetch PR and issue statistics
    let pull_requests = serde_json::json!({
        "awaiting_review": self.search_items(&format!("is:pr state:open review-requested:{}", search_username), 10),
        "open": self.search_items(&format!("is:pr state:open author:{}", search_username), 10),
        "mentions": self.search_items(&format!("is:pr state:open mentions:{}", search_username), 10),
    });

    let issues = serde_json::json!({
        "assigned": self.search_items(&format!("is:issue state:open assignee:{}", search_username), 10),
        "created": self.search_items(&format!("is:issue state:open author:{}", search_username), 10),
        "mentions": self.search_items(&format!("is:issue state:open mentions:{}", search_username), 10),
    });

    Ok(serde_json::json!({
        "total_stars": total_stars,
        "total_forks": total_forks,
        "total_repos": repos.len(),
        "contribution_graph": contrib_graph,
        "current_streak": current_streak,
        "longest_streak": longest_streak,
        "total_contributions": total_contributions,
        "languages": languages,
        "pull_requests": pull_requests,
        "issues": issues,
    }))
  }
}

impl GitHubFetcher {
  fn fetch_contribution_graph(&self, username: &str) -> Result<Value> {
    // GraphQL query for contribution calendar (matching Python behavior)
    // Always use user(login: "...") - does NOT include private contributions
    let query = format!(
      r#"{{
        user(login: "{}") {{
          contributionsCollection {{
            contributionCalendar {{
              weeks {{
                contributionDays {{
                  contributionCount
                  date
                }}
              }}
            }}
          }}
        }}
      }}"#,
      username
    );

    let data = self.gh_graphql(&query)?;
    let path = &data["data"]["user"]["contributionsCollection"]["contributionCalendar"]["weeks"];

    Ok(path.clone())
  }

  fn calculate_contribution_stats(&self, graph: &Value) -> (u32, u32, u32) {
    let weeks = graph.as_array();
    if weeks.is_none() {
      return (0, 0, 0);
    }

    let mut all_contributions: Vec<u32> = weeks
      .unwrap()
      .iter()
      .flat_map(|w| w["contributionDays"].as_array())
      .flatten()
      .filter_map(|d| d["contributionCount"].as_u64())
      .map(|c| c as u32)
      .collect();

    all_contributions.reverse();

    let total: u32 = all_contributions.iter().sum();

    let mut current_streak = 0;
    for &count in &all_contributions {
      if count > 0 {
        current_streak += 1;
      } else {
        break;
      }
    }

    let mut longest_streak = 0;
    let mut temp_streak = 0;
    for &count in &all_contributions {
      if count > 0 {
        temp_streak += 1;
        longest_streak = longest_streak.max(temp_streak);
      } else {
        temp_streak = 0;
      }
    }

    (current_streak, longest_streak, total)
  }

  fn calculate_language_stats(&self, repos: &[Value]) -> Value {
    use std::collections::HashMap;

    // First pass: collect language occurrences
    let mut language_counts: HashMap<String, i32> = HashMap::new();

    for repo in repos {
      if let Some(language) = repo["language"].as_str() {
        if !language.is_empty() {
          let normalized = language.to_lowercase();
          *language_counts.entry(normalized).or_insert(0) += 1;
        }
      }
    }

    // Calculate percentages
    let total: i32 = language_counts.values().sum();
    if total == 0 {
      return serde_json::json!({});
    }

    let mut language_percentages: HashMap<String, f64> = HashMap::new();
    for (lang, count) in language_counts {
      let percentage = (count as f64 / total as f64) * 100.0;
      // Capitalize first letter
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

  fn fetch_repos(&self, username: &str) -> Result<Vec<Value>> {
    // Always fetch public repositories only (matching Python gitfetch behavior)
    // This uses /users/{username}/repos which only returns public repos
    let mut repos = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
      let endpoint = format!(
        "/users/{}/repos?page={}&per_page={}&type=owner&sort=updated",
        username, page, per_page
      );
      let data = self.gh_api(&endpoint)?;

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

    Ok(repos)
  }

  fn get_search_username(&self, username: &str) -> String {
    // Get the username to use for search queries
    // Uses @me for the authenticated user, otherwise the provided username
    match self.gh_api("/user") {
      Ok(auth_user) => {
        if let Some(login) = auth_user["login"].as_str() {
          if login == username {
            return "@me".to_string();
          }
        }
      }
      Err(_) => {
        // If we can't determine auth user, use provided username
      }
    }
    username.to_string()
  }

  fn search_items(&self, query: &str, per_page: usize) -> Value {
    // Search issues and PRs using GitHub CLI search command
    let search_type = if query.contains("is:pr") {
      "prs"
    } else {
      "issues"
    };

    // Remove is:pr/issue from query as it's implied by search type
    let cleaned_query = query.replace("is:pr ", "").replace("is:issue ", "");

    // Parse query string and convert to command-line flags
    let flags = self.parse_search_query(&cleaned_query);

    // Build command
    let mut cmd = Command::new("gh");
    cmd.arg("search").arg(search_type);

    for flag in flags {
      cmd.arg(flag);
    }

    cmd.args(&[
      "--limit",
      &per_page.to_string(),
      "--json",
      "number,title,repository,url,state",
    ]);

    let output = match cmd.output() {
      Ok(out) if out.status.success() => out,
      _ => {
        return serde_json::json!({
          "total_count": 0,
          "items": []
        });
      }
    };

    let stdout = match String::from_utf8(output.stdout) {
      Ok(s) => s,
      Err(_) => {
        return serde_json::json!({
          "total_count": 0,
          "items": []
        });
      }
    };

    let data: Vec<Value> = match serde_json::from_str(&stdout) {
      Ok(d) => d,
      Err(_) => {
        return serde_json::json!({
          "total_count": 0,
          "items": []
        });
      }
    };

    let items: Vec<Value> = data
      .iter()
      .take(per_page)
      .map(|item| {
        let repo_info = &item["repository"];
        let repo_name = repo_info["nameWithOwner"]
          .as_str()
          .or_else(|| repo_info["name"].as_str())
          .unwrap_or("");

        serde_json::json!({
          "title": item["title"].as_str().unwrap_or(""),
          "repo": repo_name,
          "url": item["url"].as_str().unwrap_or(""),
          "number": item["number"].as_u64()
        })
      })
      .collect();

    serde_json::json!({
      "total_count": items.len(),
      "items": items
    })
  }

  fn parse_search_query(&self, query: &str) -> Vec<String> {
    // Parse search query string into command-line flags
    let mut flags = Vec::new();
    let parts: Vec<&str> = query.split_whitespace().collect();

    for part in parts {
      if let Some((key, value)) = part.split_once(':') {
        match key {
          "assignee" => {
            flags.push("--assignee".to_string());
            flags.push(value.to_string());
          }
          "author" => {
            flags.push("--author".to_string());
            flags.push(value.to_string());
          }
          "mentions" => {
            flags.push("--mentions".to_string());
            flags.push(value.to_string());
          }
          "review-requested" => {
            flags.push("--review-requested".to_string());
            flags.push(value.to_string());
          }
          "state" => {
            flags.push("--state".to_string());
            flags.push(value.to_string());
          }
          "is" => {
            // is:pr and is:issue are handled by search type
            // Skip this
          }
          _ => {
            // For other qualifiers, add as search term
            flags.push(part.to_string());
          }
        }
      } else {
        // Add as general search term
        flags.push(part.to_string());
      }
    }

    flags
  }
}
