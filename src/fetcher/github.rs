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
    // Check if this is the authenticated user
    let auth_user = self.get_authenticated_user().await.ok();
    let is_self = auth_user.as_deref() == Some(username);

    // Use /user/repos for authenticated user to include private repos
    let repos = if is_self {
      self.gh_api("/user/repos?per_page=100&affiliation=owner")?
    } else {
      self.gh_api(&format!("/users/{}/repos?per_page=100", username))?
    };

    let empty_vec = vec![];
    let repos_array = repos.as_array().unwrap_or(&empty_vec);
    let total_stars: i64 = repos_array
      .iter()
      .filter_map(|r| r["stargazers_count"].as_i64())
      .sum();
    let total_forks: i64 = repos_array
      .iter()
      .filter_map(|r| r["forks_count"].as_i64())
      .sum();

    let contrib_graph = match self.fetch_contribution_graph(username) {
      Ok(graph) => graph,
      Err(e) => {
        eprintln!("Warning: Failed to fetch contribution graph: {}", e);
        serde_json::json!([])
      }
    };

    let (current_streak, longest_streak, total_contributions) =
      self.calculate_contribution_stats(&contrib_graph);

    Ok(serde_json::json!({
        "total_stars": total_stars,
        "total_forks": total_forks,
        "total_repos": repos_array.len(),
        "contribution_graph": contrib_graph,
        "current_streak": current_streak,
        "longest_streak": longest_streak,
        "total_contributions": total_contributions,
        "languages": {},
        "pull_requests": {},
        "issues": {},
    }))
  }
}

impl GitHubFetcher {
  fn fetch_contribution_graph(&self, username: &str) -> Result<Value> {
    // Try to get authenticated user (synchronous check via command)
    let auth_output = Command::new("gh").args(&["api", "/user"]).output();

    let is_self = if let Ok(output) = auth_output {
      if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
          if let Ok(data) = serde_json::from_str::<Value>(&stdout) {
            data["login"].as_str() == Some(username)
          } else {
            false
          }
        } else {
          false
        }
      } else {
        false
      }
    } else {
      false
    };

    let query = if is_self {
      // Use viewer for authenticated user to include private contributions
      format!(
        r#"{{
          viewer {{
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
        }}"#
      )
    } else {
      format!(
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
      )
    };

    let data = self.gh_graphql(&query)?;

    let path = if is_self {
      &data["data"]["viewer"]["contributionsCollection"]["contributionCalendar"]["weeks"]
    } else {
      &data["data"]["user"]["contributionsCollection"]["contributionCalendar"]["weeks"]
    };

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
}
